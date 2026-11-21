//! One bot: its connection, its walk, and what it counts.

use std::f64::consts::TAU;
use std::path::Path;
use std::time::Duration;

use abstractions_net::ids::BodyId;
use abstractions_net::io::{Client, ClientEvent, CloseReason, ConnectOpts};
use abstractions_net::log::PlayerId;
use abstractions_net::wire::{
    Body, ClientMsg, Frame, Head, Hello, Kind, PROTOCOL, PlayerRecord, Reason, Record, RobotBody,
    ServerMsg,
};
use abstractions_net::wire::{push, records};
use glam::{DVec3, Vec3};

use super::args::Args;
use super::keep;

/// Radius of the walk in metres.
const RADIUS: f64 = 20.0;

/// Turn rate of the walk in radians per second: 2 m/s on the circle.
const TURN: f64 = 0.1;

/// Wait before connecting again after a connection ended.
const RETRY: Duration = Duration::from_secs(2);

/// What every bot counted over one second.
#[derive(Debug, Default)]
pub struct Tally {
    /// State records received.
    records: usize,
    /// Sum of the round trips QUIC measured, one per bot in, in seconds.
    rtt: f64,
    /// Round trips added up.
    pongs: u32,
}

impl Tally {
    /// Returns the line printed for the second.
    pub fn line(&self, connected: usize, count: usize) -> String {
        let rtt = self.rtt / f64::from(self.pongs.max(1)) * 1000.0;
        format!(
            "connected {connected}/{count}   records {}   rtt {rtt:.2} ms",
            self.records
        )
    }
}

/// One bot.
#[derive(Debug)]
pub struct Player {
    /// The connection, or nothing while waiting to connect again.
    client: Option<Client>,
    /// Where and how the bot connects.
    address: String,
    /// The options the bot connects with.
    opts: ConnectOpts,
    /// Angle where the walk starts, so bots spread around the circle.
    phase: f64,
    /// Universe time at the welcome, the bot's time then, and the bot's id, once in.
    welcome: Option<(f64, f64, PlayerId)>,
    /// Time the connection ended, to connect again after [`RETRY`].
    ended: Option<Duration>,
    /// True once the server sent the bot away for a reason that asking again does not change.
    gave_up: bool,
    /// The record to send once welcomed, taken from the environment.
    keep: Option<PlayerRecord>,
    /// The datagram being written, reused.
    batch: Vec<u8>,
    /// The body being written, reused.
    body: Vec<u8>,
}

impl Player {
    /// Returns bot number `index`, already connecting.
    pub fn new(index: usize, args: &Args, known: &Path) -> Self {
        let key = keep::key(index);
        let hello = Hello {
            protocol: std::env::var("BOT_PROTOCOL")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(PROTOCOL),
            name: format!("Bot {}", index + 1),
            key,
            password: args.password.clone(),
        };
        let address = args.addresses[index % args.addresses.len()].clone();
        let opts = ConnectOpts {
            hello,
            known: known.to_path_buf(),
            expected: None,
        };
        Self {
            client: Some(Client::connect(&address, opts.clone())),
            address,
            opts,
            phase: TAU * index as f64 / args.count as f64,
            welcome: None,
            ended: None,
            gave_up: false,
            keep: None,
            batch: Vec::with_capacity(128),
            body: Vec::with_capacity(64),
        }
    }

    /// Returns true while the bot is in the world.
    pub fn joined(&self) -> bool {
        self.client.is_some() && self.welcome.is_some()
    }

    /// Reads what arrived, then sends a record, and a ping on each new `second`. At `now`.
    pub fn step(&mut self, now: Duration, second: bool, tally: &mut Tally) {
        let Some(mut client) = self.client.take() else {
            if !self.gave_up && self.ended.is_some_and(|ended| now >= ended + RETRY) {
                self.client = Some(Client::connect(&self.address, self.opts.clone()));
                self.ended = None;
            }
            return;
        };
        while let Some(event) = client.poll() {
            if !self.read(event, now, tally) {
                self.ended = Some(now);
                self.welcome = None;
                return;
            }
        }
        if let Some(record) = self.keep.take() {
            client.send(ClientMsg::Record(record));
        }
        if let Some((clock, at, you)) = self.welcome {
            self.walk(&client, you, clock + now.as_secs_f64() - at);
            if second {
                tally.rtt += client.rtt().unwrap_or_default().as_secs_f64();
                tally.pongs += 1;
                client.send(ClientMsg::Ping {
                    id: now.as_secs() as u32,
                    sent: now.as_secs_f64(),
                });
            }
        }
        self.client = Some(client);
    }

    /// Counts one event. Returns false when the connection is over.
    fn read(&mut self, event: ClientEvent, now: Duration, tally: &mut Tally) -> bool {
        match event {
            ClientEvent::Message(ServerMsg::Welcome(welcome)) => {
                println!(
                    "{} {}",
                    self.opts.hello.name,
                    keep::said(welcome.record.as_ref())
                );
                self.welcome = Some((welcome.clock, now.as_secs_f64(), welcome.you));
                self.keep = keep::record();
            }
            ClientEvent::Datagram(bytes) => tally.records += records(&bytes).count(),
            ClientEvent::Closed(reason) => {
                let words = reason.words(&self.address, None);
                eprintln!(
                    "{} ({}) closed: {words} [{reason:?}]",
                    self.opts.hello.name, self.address
                );
                self.gave_up = matches!(
                    reason,
                    CloseReason::Refused(
                        Reason::Replaced | Reason::WrongPassword | Reason::Outdated | Reason::Newer
                    )
                );
                return false;
            }
            _ => {}
        }
        true
    }

    /// Sends the state of the robot of player `you` at universe time `time`, on its circle.
    fn walk(&mut self, client: &Client, you: PlayerId, time: f64) {
        let robot = pose(self.phase + TURN * time, time);
        let frame = Frame::Surface(BodyId::default());
        let head = Head {
            player: you,
            kind: Kind::Robot,
            frame,
            time,
            flags: 0,
        };
        self.body.clear();
        robot.encode(&frame, &mut self.body);
        self.batch.clear();
        push(
            &mut self.batch,
            &Record {
                head,
                body: &self.body,
            },
        );
        client.datagram(&self.batch);
    }

    /// Says goodbye and waits for the connection to end.
    pub fn close(self) {
        if let Some(client) = self.client {
            client.close();
        }
    }
}

/// Returns the robot at `angle` on its circle, `time` seconds into its walk.
fn pose(angle: f64, time: f64) -> RobotBody {
    let (sin, cos) = angle.sin_cos();
    RobotBody {
        position: DVec3::new(RADIUS * cos, 0.0, RADIUS * sin),
        velocity: Vec3::new(
            (-RADIUS * TURN * sin) as f32,
            0.0,
            (RADIUS * TURN * cos) as f32,
        ),
        yaw: (-angle) as f32,
        speed: (RADIUS * TURN) as f32,
        stride_distance: (RADIUS * TURN * time) as f32,
        ..RobotBody::default()
    }
}
