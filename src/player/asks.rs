//! What each body is asked this step, gathered from the intents of its player.
//!
//! The controller reads these, never the keyboard, so a body moves the same whether its intents
//! come from the keys of this machine or from anywhere else.

use bevy::prelude::*;

use super::Squared;
use super::controller::Motion;
use super::intent::{Intent, wish_direction};
use super::lead::View;
use super::walk::Step;
use crate::sim::{self, IntentOf, PlayerId};

/// Sink in metres per second past which a held run key fires the boots without a new press.
/// It is faster than a jump on flat ground comes down, so a running jump never fires them.
const PLUNGE: f32 = 10.0;

/// Seconds the wings wait, once asked open with a tool in the hands, while the hand slings the
/// tool over the back: the first phase of the opening. The arms spread only after the hand let
/// go, so a tool is never in the hand of spread wings. It is the same in a fast fall: the hand
/// must be free before the arms take the wing, and 0.35 s cost 14 m at 40 m/s.
pub const SLING: f32 = 0.35;

/// The intents of one player for the step in progress, on its body.
#[derive(Component, Debug, Default, Clone, Copy, PartialEq)]
pub struct Intents {
    /// Forward and strafe to the right, each from -1 to 1.
    axes: Vec2,
    /// Heading and pitch of the view, kept until the next look.
    look: Vec2,
    /// The part of the heading the player's own hand turned this step.
    turn: f32,
    /// Whether the player sees the body from behind, kept until the next look.
    behind: bool,
    /// Whether the jump is held.
    jump: bool,
    /// Whether the jump was held on the step before.
    held: bool,
    /// Whether the run, or the boost, is held.
    boost: bool,
    /// Whether the run was held on the step before.
    ran: bool,
    /// Whether the crouch is held.
    crouch: bool,
    /// Whether the wingsuit was asked to spread or fold this step.
    wings: bool,
    /// Flat turn on the wings, from -1 (left) to 1 (right).
    yaw: f32,
    /// Whether the view steers the wings.
    steer: bool,
    /// Whether the body has a tool in its hands, see [`super::Squared`].
    drawn: bool,
}

impl Intents {
    /// Returns the intents at the start of a new step: nothing asked, the view kept.
    fn next(self) -> Self {
        Self {
            look: self.look,
            behind: self.behind,
            held: self.jump,
            ran: self.boost,
            ..Self::default()
        }
    }

    /// Takes one intent into the step.
    fn take(&mut self, intent: sim::Intent) {
        match intent {
            sim::Intent::Move { forward, strafe } => self.axes = Vec2::new(forward, strafe),
            sim::Intent::Look {
                yaw,
                pitch,
                turn,
                behind,
            } => (self.look, self.turn, self.behind) = (Vec2::new(yaw, pitch), turn, behind),
            sim::Intent::Jump => self.jump = true,
            sim::Intent::Boost => self.boost = true,
            sim::Intent::Crouch => self.crouch = true,
            sim::Intent::Wings => self.wings = true,
            sim::Intent::Yaw(right) => self.yaw = right.clamp(-1.0, 1.0),
            sim::Intent::Steer => self.steer = true,
            sim::Intent::Board | sim::Intent::Leave | sim::Intent::Interact => {}
        }
    }

    /// Returns where the player looks.
    pub(super) fn view(&self) -> View {
        View {
            yaw: self.look.x,
            pitch: self.look.y,
            third: self.behind,
        }
    }
}

/// Starts a new step on every body and takes in the intents of its player, and whether the
/// hands of the local body hold a tool.
pub(super) fn gather(
    mut intents: MessageReader<IntentOf>,
    mut bodies: Query<(&PlayerId, &mut Intents)>,
    squared: Option<Res<Squared>>,
) {
    let drawn = squared.is_some_and(|squared| squared.drawn);
    for (id, mut asks) in &mut bodies {
        *asks = Intents {
            drawn: drawn && *id == PlayerId::LOCAL,
            ..asks.next()
        };
    }
    for message in intents.read() {
        for (_, mut asks) in bodies.iter_mut().filter(|(id, _)| **id == message.player) {
            asks.take(message.intent);
        }
    }
}

/// Returns the seconds of the sling left to `motion` after `dt` more: [`SLING`] on the frame the
/// wings were `opened` with a tool `drawn`, then less until none, and none once the wings fold.
fn slung(motion: &Motion, (opened, drawn): (bool, bool), dt: f32) -> f32 {
    match (opened && drawn, motion.wings) {
        (true, _) => SLING,
        (false, true) => (motion.sling - dt).max(0.0),
        (false, false) => 0.0,
    }
}

/// Returns what the player asks of a body this step, against the heading the movement keys
/// hold.
///
/// The gear only answers in the air. There a new press of the jump spreads or folds the
/// wingsuit, as the wings intent does. The run fires the boots once it is pressed in the air,
/// and keeps them on until let go: on the wings it boosts, folded it brakes. A run held since
/// the ground fires them only in a folded fall past [`PLUNGE`], to brake it, and never on the
/// wings: sprinting off a cliff and spreading them glides, and the run pressed again boosts. In
/// the water the run held fires the boots at once: they jet the swimming body. Asked open with
/// a tool in the hands, the wings wait for the sling, see [`SLING`].
pub(super) fn ask(motion: &mut Motion, asks: &Intents, step: &Step) -> Intent {
    let yaw = motion.steer.step(step.view.yaw, asks.turn, asks.axes);
    let aloft = !motion.grounded && !motion.swimming;
    let again = motion.coyote <= 0.0 && asks.jump && !asks.held;
    let opened = aloft && (asks.wings || again) && !motion.wings;
    motion.wings ^= aloft && (asks.wings || again);
    motion.sling = slung(motion, (opened, asks.drawn), step.dt);
    let plunge = !motion.wings && motion.velocity.y < -PLUNGE;
    let fired = motion.boots || !asks.ran || plunge;
    motion.boots = (aloft && fired || motion.swimming) && asks.boost;
    Intent {
        wish: wish_direction(asks.axes.x, asks.axes.y, yaw),
        forward: asks.axes.x,
        right: asks.axes.y,
        sprint: asks.boost,
        crouch: asks.crouch,
        jump: asks.jump,
        yaw: asks.yaw,
        steer: asks.steer,
    }
}
