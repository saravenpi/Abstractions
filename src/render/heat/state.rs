//! The heat of the craft of the player as the rest of the game reads it: written by the ship and
//! the gear, eased here, read by the view, the sheath, the shake and the roar.

use bevy::prelude::*;

use super::Glow;

/// Where the glowing craft is, in render space.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Craft {
    /// Centre of the craft.
    pub centre: Vec3,
    /// Unit vector along its flight, away from the tail.
    pub heading: Vec3,
    /// Radius in metres of the sphere that holds the craft.
    pub radius: f32,
}

/// The glow of the craft of the player, zero while nothing flies fast in thick air.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Default)]
pub struct Heat {
    /// The glow now, eased so that it never steps.
    pub glow: Glow,
    /// Where the craft was when it last glowed.
    pub craft: Craft,
    offered: Option<(Glow, Craft)>,
}

impl Heat {
    /// Says what the craft makes this frame. A frame without an offer lets the glow die out.
    pub fn offer(&mut self, glow: Glow, craft: Craft) {
        self.offered = Some((glow, craft));
    }

    /// Returns true while the glow shows or an offer waits: the systems that read it run then.
    pub fn live(&self) -> bool {
        self.offered.is_some() || self.glow.heat > 0.0 || self.glow.pressure > 0.0
    }
}

/// Eases the glow toward what was offered this frame, or toward nothing.
fn settle(time: Res<Time>, mut heat: ResMut<Heat>) {
    let (next, craft) = heat.offered.unwrap_or((Glow::NONE, heat.craft));
    let glow = heat.glow.follow(next, time.delta_secs());
    heat.set_if_neq(Heat {
        glow,
        craft,
        offered: None,
    });
}

/// Run condition: the heat has something to ease.
fn live(heat: Res<Heat>) -> bool {
    heat.live()
}

/// Registers the heat, eased after the ship and the gear have offered theirs.
pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Heat>()
        .add_systems(PostUpdate, settle.run_if(live).in_set(HeatSet));
}

/// Where the heat is eased, in `PostUpdate`: the view and the sheath read it after.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HeatSet;
