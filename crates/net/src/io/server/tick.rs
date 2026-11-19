//! The server's clock: what wakes the loop, an event or the next tick.

use std::time::{Duration, Instant};

use smol::Timer;
use smol::channel::Receiver;
use smol::future;

use super::run::Event;
use crate::host::TICK_HZ;

/// What woke the loop.
pub(super) enum Wake {
    /// An event from a connection.
    Event(Event),
    /// Time for a tick.
    Tick,
}

/// Ticks at [`TICK_HZ`] while the host is busy, with one timer reused for every tick.
#[derive(Debug)]
pub(super) struct Ticker {
    /// When the next tick is due, while the host is busy.
    due: Option<Instant>,
    /// The timer that waits for it.
    timer: Timer,
}

impl Ticker {
    /// Makes a ticker with no tick due.
    pub(super) fn new() -> Self {
        Self {
            due: None,
            timer: Timer::never(),
        }
    }

    /// Waits for the next event, or for the next tick when `busy`. Without ticks to give, it
    /// waits for an event only, so an empty server sleeps. Returns a stop when every sender
    /// is gone.
    pub(super) async fn wait(&mut self, inbox: &Receiver<Event>, busy: bool) -> Wake {
        let period = Duration::from_secs_f64(1.0 / TICK_HZ);
        if !busy {
            self.due = None;
            return Wake::Event(inbox.recv().await.unwrap_or(Event::Stop));
        }
        let due = *self.due.get_or_insert_with(|| Instant::now() + period);
        self.timer.set_at(due);
        let event = async { Some(inbox.recv().await.unwrap_or(Event::Stop)) };
        let timer = &mut self.timer;
        let tick = async {
            timer.await;
            None
        };
        match future::or(event, tick).await {
            Some(event) => Wake::Event(event),
            None => {
                self.due = Some((due + period).max(Instant::now()));
                Wake::Tick
            }
        }
    }
}
