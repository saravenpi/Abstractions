//! The context of this frame, derived once from the world and read by every element.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::{Context, Element, Facts, context, shown};
use crate::gear::{Gear, Mode};
use crate::player::{Motion, Player};
use crate::ship::ShipTelemetry;
use crate::space::{Location, Travel};

/// The context of this frame, derived once and read by every element.
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct Where(pub Context);

impl Default for Where {
    fn default() -> Self {
        Self(Context::Menu)
    }
}

impl Where {
    /// Returns whether `element` shows in this context.
    pub fn has(&self, element: Element) -> bool {
        shown(self.0).contains(element)
    }
}

/// The state the context is read from.
#[derive(SystemParam)]
pub struct Sources<'w, 's> {
    location: Option<Res<'w, Location>>,
    telemetry: Option<Res<'w, ShipTelemetry>>,
    players: Query<'w, 's, (&'static Motion, &'static Gear), With<Player>>,
}

impl Sources<'_, '_> {
    /// Gathers the facts, with `control` for whether the player controls the game.
    fn facts(&self, control: bool) -> Facts {
        let telemetry = self.telemetry.as_deref().copied().unwrap_or_default();
        let (swimming, airborne) = self.players.iter().next().map_or((false, false), |(m, g)| {
            (
                m.swimming,
                g.state.mode.airborne() && g.state.mode != Mode::Fall,
            )
        });
        Facts {
            control,
            seated: self.location.as_deref().map(|at| at.travel) == Some(Travel::InShip),
            swimming,
            airborne,
            landed: telemetry.landed,
            frame: telemetry.frame,
            warp: telemetry.warp,
            bound: telemetry.apoapsis.is_some() && telemetry.periapsis.is_some(),
        }
    }
}

/// Writes the context of this frame, once.
pub fn update(In(control): In<bool>, sources: Sources, mut now: ResMut<Where>) {
    now.set_if_neq(Where(context(&sources.facts(control))));
}

/// Run condition: the compass shows in the current context.
pub fn shows_compass(now: Res<Where>) -> bool {
    now.has(Element::Compass)
}

/// Run condition: the coordinates line shows in the current context.
pub fn shows_coords(now: Res<Where>) -> bool {
    now.has(Element::Coords)
}

/// Run condition: the climate line shows in the current context.
pub fn shows_climate(now: Res<Where>) -> bool {
    now.has(Element::Climate)
}
