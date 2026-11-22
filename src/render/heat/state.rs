//! The heat of the craft of the player as the rest of the game reads it: written by the ship and
//! the gear, eased here, read by the sheath, the hull light, the cabin, the shake and the roar.

use bevy::prelude::*;

use super::Glow;
use super::gas::incandescence;
use super::sheath::SheathUniform;
use super::tune::SHEATH_BOUND;
use crate::space::Shift;

/// Where the glowing craft is, in render space.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Craft {
    /// Centre of the craft.
    pub centre: Vec3,
    /// Unit vector along its nose, away from the tail: the glow sits on the hull, and the
    /// airflow turns the nose into the flight.
    pub heading: Vec3,
    /// Radius in metres of the sphere that holds the craft.
    pub radius: f32,
    /// Unit vector out of its roof, which sets where its wings lie across the flight.
    pub roof: Vec3,
}

impl Craft {
    /// Returns the pose of the unit sphere that holds the sheath of this craft: centred behind
    /// it along its flight, from just ahead of the bow shock to the end of the wake.
    pub fn bound(&self) -> Transform {
        let (back, size) = SHEATH_BOUND;
        Transform::from_translation(self.centre - self.heading * (back * self.radius))
            .with_scale(Vec3::splat(size * self.radius))
    }

    /// Returns the values the shader draws this craft with when it glows with `glow`: the
    /// plasma in the colour of the gas, and the skin in the colour of its temperature.
    pub fn sheath(&self, glow: &Glow) -> SheathUniform {
        SheathUniform {
            centre: self.centre.extend(self.radius),
            axis: self.heading.extend(0.0),
            roof: self.roof.extend(glow.pressure),
            glow: glow.color.extend(glow.heat),
            skin: incandescence(glow.kelvin).extend(glow.shine),
        }
    }
}

/// The glow of the craft of the player, zero while nothing flies fast in thick air.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Default)]
pub struct Heat {
    /// The glow now, eased so that it never steps.
    pub glow: Glow,
    /// Where the craft was when it last glowed.
    pub craft: Craft,
    /// What was offered for this frame, before [`settle`] eases toward it.
    pub(crate) offered: Option<(Glow, Craft)>,
}

impl Heat {
    /// Says what the craft makes this frame. A frame without an offer lets the glow die out.
    pub fn offer(&mut self, glow: Glow, craft: Craft) {
        self.offered = Some((glow, craft));
    }

    /// Returns true while the plasma glows, the air presses, the skin is hot or an offer waits:
    /// the systems that read it run then.
    pub fn live(&self) -> bool {
        let glow = self.glow;
        self.offered.is_some() || glow.heat > 0.0 || glow.pressure > 0.0 || glow.kelvin > 0.0
    }

    /// Carries the craft pose and any pending offer across an anchor `shift`.
    pub fn shift(&mut self, shift: &Shift) {
        let turn = shift.turn(Quat::IDENTITY);
        let move_craft = |craft: &mut Craft| {
            craft.centre = shift.point(craft.centre);
            craft.heading = turn * craft.heading;
            craft.roof = turn * craft.roof;
        };
        move_craft(&mut self.craft);
        if let Some((_, craft)) = &mut self.offered {
            move_craft(craft);
        }
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
