//! What each body is asked this step, gathered from the intents of its player.
//!
//! The controller reads these, never the keyboard, so a body moves the same whether its intents
//! come from the keys of this machine or from anywhere else.

use bevy::prelude::*;

use super::controller::Motion;
use super::intent::{Intent, wish_direction};
use super::lead::View;
use super::walk::Step;
use crate::sim::{self, IntentOf, PlayerId};

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
    /// Whether the wingsuit was asked to spread or fold this step.
    wings: bool,
}

impl Intents {
    /// Returns the intents at the start of a new step: nothing asked, the view kept.
    fn next(self) -> Self {
        Self {
            look: self.look,
            behind: self.behind,
            held: self.jump,
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
            sim::Intent::Wings => self.wings = true,
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

/// Starts a new step on every body and takes in the intents of its player.
pub(super) fn gather(
    mut intents: MessageReader<IntentOf>,
    mut bodies: Query<(&PlayerId, &mut Intents)>,
) {
    for (_, mut asks) in &mut bodies {
        *asks = asks.next();
    }
    for message in intents.read() {
        for (_, mut asks) in bodies.iter_mut().filter(|(id, _)| **id == message.player) {
            asks.take(message.intent);
        }
    }
}

/// Returns what the player asks of a body this step, against the heading the movement keys
/// hold. The wingsuit only answers in the air, and so does the second jump that fires the boots.
pub(super) fn ask(motion: &mut Motion, asks: &Intents, step: &Step) -> Intent {
    let yaw = motion.steer.step(step.view.yaw, asks.turn, asks.axes);
    let aloft = !motion.grounded && !motion.swimming;
    motion.wings ^= aloft && asks.wings;
    let again = aloft && motion.coyote <= 0.0 && asks.jump && !asks.held;
    motion.boots = asks.jump && (motion.boots || again);
    Intent {
        wish: wish_direction(asks.axes.x, asks.axes.y, yaw),
        forward: asks.axes.x,
        right: asks.axes.y,
        sprint: asks.boost,
        jump: asks.jump,
    }
}
