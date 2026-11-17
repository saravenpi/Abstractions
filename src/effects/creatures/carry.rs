//! Carrying creature state across anchor rebases: the points creatures remember outside transforms.

use bevy::prelude::*;

use super::fireflies::Swarms;
use super::flock::Flocks;
use super::wander::Wander;
use crate::space::{Anchor, Shift};

/// Returns how render space moved between two anchors, when both are surface anchors of one
/// frame and differ. Positions kept across any other change mean nothing afterwards.
pub fn moved(old: Option<Anchor>, new: &Anchor) -> Option<Shift> {
    let old = old.filter(|old| old.frame == new.frame && !new.flat && old != new)?;
    Some(Shift::between(&old, new))
}

/// Moves flock centers, swarm anchors and flight targets with render space when the anchor was
/// rebased. The creature entities themselves are marked `Rebased` and moved with the anchor.
pub fn carry(
    anchor: Res<Anchor>,
    mut seen: Local<Option<Anchor>>,
    mut herds: (ResMut<Flocks>, ResMut<Swarms>),
    mut wanders: Query<&mut Wander>,
) {
    let Some(shift) = moved(seen.replace(*anchor), &anchor) else {
        return;
    };
    for flock in &mut herds.0.0 {
        flock.center = shift.point(flock.center);
    }
    herds.1.carry(&shift);
    for mut wander in &mut wanders {
        wander.target = shift.point(wander.target);
    }
}
