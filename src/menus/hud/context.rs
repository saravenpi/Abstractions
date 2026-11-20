//! What the HUD shows where the player is: one pure rule, [`shown`], from a [`Context`] to the
//! set of elements that mean something there. Every element asks it, so a reading that is
//! useless in the place the player stands in never shows.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::MarkerKind;
use crate::gear::{Gear, Mode};
use crate::player::{Motion, Player};
use crate::ship::{ShipTelemetry, WarpState};
use crate::space::{Frame, Location, Travel};

/// Where the player is, as far as the HUD cares.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Context {
    /// Walking on the ground.
    OnFoot,
    /// Swimming or diving.
    Swimming,
    /// In the air on the wingsuit or the boots.
    Gliding,
    /// Seated in the ship, resting on the ground or the sea.
    ShipLanded,
    /// The ship flies in the air, near a surface.
    ShipAir,
    /// The ship flies outside the air, bound to a body by a closed path.
    ShipOrbit,
    /// The ship flies in the open space of a star system, bound to nothing.
    ShipSpace,
    /// The ship flies between the stars.
    ShipBetween,
    /// The warp drive spools or runs.
    Warp,
    /// The map or a menu is open: the HUD is hidden.
    Menu,
}

/// One element of the HUD.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Element {
    /// The compass bar and its markers.
    Compass,
    /// The cardinal points on the compass: north needs a ground.
    Cardinals,
    /// The minimap of the ground.
    Minimap,
    /// The coordinates line.
    Coords,
    /// The place and weather line and the banner of places.
    Place,
    /// The climate line: temperature and weather.
    Climate,
    /// Height above the ground.
    Altitude,
    /// Speed.
    Speed,
    /// Main engine lever.
    Throttle,
    /// Flight assist.
    Assist,
    /// Highest and lowest point of the orbit.
    Apsides,
    /// Landing gear.
    Gear,
    /// The warp spool bar.
    Spool,
    /// The frame integrity row. It hides by itself while the body is whole.
    Integrity,
    /// The one prompt row.
    Prompt,
    /// Markers over the world and at the edge of the screen.
    Markers,
    /// Herd markers, on a living planet near the ground.
    Herds,
}

impl Element {
    /// Every element.
    pub const ALL: [Element; 17] = [
        Element::Compass,
        Element::Cardinals,
        Element::Minimap,
        Element::Coords,
        Element::Place,
        Element::Climate,
        Element::Altitude,
        Element::Speed,
        Element::Throttle,
        Element::Assist,
        Element::Apsides,
        Element::Gear,
        Element::Spool,
        Element::Integrity,
        Element::Prompt,
        Element::Markers,
        Element::Herds,
    ];
}

/// The facts the context is derived from.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Facts {
    /// Whether the player controls the game: no map, no menu.
    pub control: bool,
    /// Whether the player sits in the ship.
    pub seated: bool,
    /// Whether the body swims.
    pub swimming: bool,
    /// Whether the flight gear carries the body through the air.
    pub airborne: bool,
    /// Whether the ship rests on the ground or the sea.
    pub landed: bool,
    /// The frame the ship is simulated in.
    pub frame: Frame,
    /// What the warp drive does.
    pub warp: WarpState,
    /// Whether the ship flies a closed path around a body.
    pub bound: bool,
}

/// Returns the context of `facts`.
pub fn context(facts: &Facts) -> Context {
    if !facts.control {
        return Context::Menu;
    }
    if !facts.seated {
        return match (facts.swimming, facts.airborne) {
            (true, _) => Context::Swimming,
            (_, true) => Context::Gliding,
            _ => Context::OnFoot,
        };
    }
    match (facts.warp, facts.frame) {
        (WarpState::Off, Frame::Surface(_)) if facts.landed => Context::ShipLanded,
        (WarpState::Off, Frame::Surface(_)) => Context::ShipAir,
        (WarpState::Off, Frame::System(_)) if facts.bound => Context::ShipOrbit,
        (WarpState::Off, Frame::System(_)) => Context::ShipSpace,
        (WarpState::Off, Frame::Galaxy(_)) => Context::ShipBetween,
        _ => Context::Warp,
    }
}

/// A set of HUD elements, one bit each.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Set(u32);

impl Set {
    /// Returns the set of `elements`.
    pub const fn of(elements: &[Element]) -> Set {
        let mut bits = 0;
        let mut at = 0;
        while at < elements.len() {
            bits |= 1 << elements[at] as u32;
            at += 1;
        }
        Set(bits)
    }

    /// Returns whether `element` is in the set.
    pub const fn contains(self, element: Element) -> bool {
        self.0 & (1 << element as u32) != 0
    }

    /// Returns the elements of the set, in declaration order.
    pub fn elements(self) -> Vec<Element> {
        Element::ALL
            .into_iter()
            .filter(|element| self.contains(*element))
            .collect()
    }
}

/// Elements of the ground: a compass with north, the minimap, the place and its weather.
const GROUND: Set = Set::of(&[
    Element::Compass,
    Element::Cardinals,
    Element::Minimap,
    Element::Coords,
    Element::Place,
    Element::Climate,
    Element::Prompt,
    Element::Markers,
    Element::Integrity,
]);

/// Elements of the ship anywhere: its engine, and the compass with its markers.
const HELM: Set = Set::of(&[
    Element::Compass,
    Element::Prompt,
    Element::Markers,
    Element::Throttle,
    Element::Assist,
    Element::Integrity,
]);

/// Returns the elements that mean something in `context`. The integrity row is in every
/// context but a menu: it hides by itself while nothing is damaged.
pub const fn shown(context: Context) -> Set {
    let (base, extra): (Set, &[Element]) = match context {
        Context::Menu => return Set(0),
        Context::OnFoot => (GROUND, &[Element::Herds]),
        Context::Swimming => (GROUND, &[]),
        Context::Gliding => (GROUND, &[Element::Altitude, Element::Speed, Element::Herds]),
        Context::ShipLanded => (Set(GROUND.0 | HELM.0), &[Element::Gear]),
        Context::ShipAir => (
            Set(GROUND.0 | HELM.0),
            &[Element::Altitude, Element::Speed, Element::Gear, Element::Herds],
        ),
        Context::ShipOrbit => (HELM, &[Element::Altitude, Element::Speed, Element::Apsides]),
        Context::ShipSpace | Context::ShipBetween => (HELM, &[Element::Speed]),
        Context::Warp => (
            Set::of(&[Element::Compass, Element::Prompt, Element::Markers, Element::Integrity]),
            &[Element::Speed, Element::Spool],
        ),
    };
    Set(base.0 | Set::of(extra).0)
}

impl Context {
    /// Every context.
    pub const ALL: [Context; 10] = [
        Context::OnFoot,
        Context::Swimming,
        Context::Gliding,
        Context::ShipLanded,
        Context::ShipAir,
        Context::ShipOrbit,
        Context::ShipSpace,
        Context::ShipBetween,
        Context::Warp,
        Context::Menu,
    ];

    /// Whether the player sits in the ship.
    pub const fn is_ship(self) -> bool {
        matches!(
            self,
            Context::ShipLanded
                | Context::ShipAir
                | Context::ShipOrbit
                | Context::ShipSpace
                | Context::ShipBetween
                | Context::Warp
        )
    }

    /// Whether the ship is outside the air of a planet.
    pub const fn in_space(self) -> bool {
        matches!(
            self,
            Context::ShipOrbit | Context::ShipSpace | Context::ShipBetween | Context::Warp
        )
    }
}

/// Returns whether a marker of `kind` shows in `context`: in warp only the course, in space no
/// pins, and herds only where the herd element shows.
pub fn marker_shown(context: Context, kind: MarkerKind) -> bool {
    match kind {
        MarkerKind::Herd => shown(context).contains(Element::Herds),
        _ if context == Context::Warp => kind == MarkerKind::Course,
        MarkerKind::Waypoint | MarkerKind::Favourite => !context.in_space(),
        _ => true,
    }
}

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
pub(super) struct Sources<'w, 's> {
    location: Option<Res<'w, Location>>,
    telemetry: Option<Res<'w, ShipTelemetry>>,
    players: Query<'w, 's, (&'static Motion, &'static Gear), With<Player>>,
}

impl Sources<'_, '_> {
    /// Gathers the facts, with `control` for whether the player controls the game.
    fn facts(&self, control: bool) -> Facts {
        let telemetry = self.telemetry.as_deref().copied().unwrap_or_default();
        let (swimming, airborne) = self.players.iter().next().map_or((false, false), |(m, g)| {
            (m.swimming, g.state.mode.airborne() && g.state.mode != Mode::Fall)
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
pub(super) fn update(In(control): In<bool>, sources: Sources, mut now: ResMut<Where>) {
    now.set_if_neq(Where(context(&sources.facts(control))));
}

/// Run condition: the compass shows in the current context.
pub(super) fn shows_compass(now: Res<Where>) -> bool {
    now.has(Element::Compass)
}

/// Run condition: the coordinates line shows in the current context.
pub(super) fn shows_coords(now: Res<Where>) -> bool {
    now.has(Element::Coords)
}

/// Run condition: the climate line shows in the current context.
pub(super) fn shows_climate(now: Res<Where>) -> bool {
    now.has(Element::Climate)
}
