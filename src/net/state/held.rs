//! The newest body of a thing that does not move on its own, such as how hurt a robot is.

use std::collections::HashMap;
use std::ops::Deref;

use abstractions_net::wire::{Body, Record};
use bevy::prelude::*;

use super::store::Store;
use super::track::Arrival;
use crate::sim::PlayerId;

/// The newest body of one thing that does not move on its own, and when it came.
#[derive(Debug, Clone, Copy)]
pub struct Held<B> {
    /// The body of the newest record.
    pub body: B,
    /// Local time in seconds the record came at.
    pub heard: f64,
}

impl<B> Deref for Held<B> {
    type Target = B;

    /// Returns the body.
    fn deref(&self) -> &B {
        &self.body
    }
}

impl<B: Body> Store for HashMap<PlayerId, Held<B>> {
    /// Keeps the body of the record as the newest of its player.
    fn take(&mut self, record: &Record, arrival: &mut Arrival) {
        let head = &record.head;
        if head.kind != B::KIND {
            return;
        }
        if let Some(body) = B::decode(&head.frame, record.body) {
            let held = Held {
                body,
                heard: arrival.local,
            };
            self.insert(head.player, held);
        }
    }

    /// Keeps the bodies for which `keep` holds.
    fn sweep(&mut self, _: &mut Commands, keep: &dyn Fn(PlayerId, f64) -> bool) {
        self.retain(|player, held| keep(*player, held.heard));
    }
}
