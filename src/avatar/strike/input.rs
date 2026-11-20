//! The hands of the local robot: the primary button swings or throws, the secondary one guards, the
//! mouse over the last moment picks the side, and the number key of the held hotbar slot draws or
//! stows the tool.

use bevy::ecs::system::SystemParam;
use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::prelude::*;

use super::super::{Avatar, AvatarPose, Mode};
use super::act::{Act, Ask, Strike, Timing, next};
use super::dir::{Dir, Strokes};
use super::held::Held;
use super::path::Path;
use super::stop::Hitstop;
use crate::craft::{Pack, wield::pressed};
use crate::player::Player;
use crate::sim::PlayerId;

/// The button that swings, or draws and throws a spear.
pub const PRIMARY: MouseButton = MouseButton::Left;

/// The button that holds a guard up.
pub const SECONDARY: MouseButton = MouseButton::Right;

/// A spear let go by `body` after a draw of `share` of a full one.
#[derive(Message, Debug, Clone, Copy, PartialEq)]
pub struct Loose {
    /// Root of the body that threw it.
    pub body: Entity,
    /// Share of a full draw, 0 to 1.
    pub share: f32,
}

/// Finds the body of the local robot and its player.
#[derive(SystemParam)]
pub struct Robot<'w, 's> {
    /// Every body, with its player.
    bodies: Query<'w, 's, (Entity, &'static ChildOf), With<Avatar>>,
    /// Every player.
    players: Query<'w, 's, &'static PlayerId, With<Player>>,
}

impl Robot<'_, '_> {
    /// Returns the body of the local robot and its player entity.
    pub fn pair(&self) -> Option<(Entity, Entity)> {
        let local = |(_, of): &(Entity, &ChildOf)| {
            self.players
                .get(of.parent())
                .is_ok_and(|id| *id == PlayerId::LOCAL)
        };
        self.bodies
            .iter()
            .find(local)
            .map(|(body, of)| (body, of.parent()))
    }

    /// Returns the body of the local robot.
    pub fn body(&self) -> Option<Entity> {
        self.pair().map(|(body, _)| body)
    }
}

/// Returns true when a body in `mode` stands on its feet and may use its hands.
fn standing(mode: Mode) -> bool {
    matches!(mode, Mode::Idle | Mode::Walk | Mode::Run | Mode::Land)
}

/// Returns what the buttons ask of hands that are `free` to strike, the mouse asking `dir`.
fn asked(buttons: &ButtonInput<MouseButton>, free: bool, dir: Dir) -> Ask {
    Ask {
        strike: free && buttons.just_pressed(PRIMARY),
        holding: buttons.pressed(PRIMARY),
        guard: free && buttons.pressed(SECONDARY),
        dir,
    }
}

/// Steps `strike` of `held` under `ask` by `dt` seconds and returns the share of a draw let go.
/// A new act takes the side the mouse asks; a new swing has landed nothing yet.
fn step(strike: &mut Strike, held: &Held, ask: &Ask, dt: f32) -> Option<f32> {
    if strike.act == Act::Ready {
        strike.dir = ask.dir;
    }
    let timing = Timing::of(held, &Path::of(held.style, strike.dir)).felt(&strike.hurt);
    let (act, loosed) = next(strike.act, ask, &timing, held.thrown(), dt);
    if matches!(act, Act::Windup(_)) && !matches!(strike.act, Act::Windup(_)) {
        strike.landed = false;
    }
    strike.act = act;
    loosed
}

/// Reads the mouse and the keys into the strike of the local robot. The strike stands still
/// during a hitstop it is in.
pub fn ask(
    (time, buttons, keys): (
        Res<Time>,
        Res<ButtonInput<MouseButton>>,
        Res<ButtonInput<KeyCode>>,
    ),
    (motion, mut strokes, stop): (Res<AccumulatedMouseMotion>, ResMut<Strokes>, Res<Hitstop>),
    (robot, pack): (Robot, Res<Pack>),
    mut bodies: Query<(&mut Strike, &mut Held, &AvatarPose)>,
    mut loose: MessageWriter<Loose>,
) {
    let now = time.elapsed_secs();
    strokes.push(now, motion.delta);
    let dir = strokes.dir(now);
    let Some(body) = robot.body() else { return };
    let Ok((mut strike, mut held, pose)) = bodies.get_mut(body) else {
        return;
    };
    if pressed(&keys) == Some(pack.bar.held) {
        held.out = !held.out;
        strike.act = Act::Ready;
    }
    let ask = asked(&buttons, held.out && standing(pose.mode), dir);
    let dt = if stop.holds(body) {
        0.0
    } else {
        time.delta_secs()
    };
    if let Some(share) = step(&mut strike, &held, &ask, dt) {
        loose.write(Loose { body, share });
    }
}
