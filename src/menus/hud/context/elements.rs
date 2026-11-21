//! The elements of the HUD and the rule that maps each context to the ones it shows.

use super::Context;
use crate::menus::hud::MarkerKind;

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
    /// The headlight switch of the ship.
    Headlight,
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
    /// The reticle around the crosshair and the hit mark: on foot only, where the robot swings a
    /// tool.
    Reticle,
    /// The names and health of the animal fought and the one looked at, over their heads: on foot,
    /// with or without a tool.
    Labels,
}

impl Element {
    /// Every element.
    #[cfg(test)]
    pub const ALL: [Element; 20] = [
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
        Element::Headlight,
        Element::Spool,
        Element::Integrity,
        Element::Prompt,
        Element::Markers,
        Element::Herds,
        Element::Reticle,
        Element::Labels,
    ];
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
    #[cfg(test)]
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
        Context::OnFoot => (GROUND, &[Element::Herds, Element::Reticle, Element::Labels]),
        Context::Swimming => (GROUND, &[]),
        Context::Gliding => (GROUND, &[Element::Altitude, Element::Speed, Element::Herds]),
        Context::ShipLanded => (Set(GROUND.0 | HELM.0), &[Element::Gear, Element::Headlight]),
        Context::ShipAir => (
            Set(GROUND.0 | HELM.0),
            &[
                Element::Altitude,
                Element::Speed,
                Element::Gear,
                Element::Headlight,
                Element::Herds,
            ],
        ),
        Context::ShipOrbit => (HELM, &[Element::Altitude, Element::Speed, Element::Apsides]),
        Context::ShipSpace | Context::ShipBetween => (HELM, &[Element::Speed]),
        Context::Warp => (
            Set::of(&[
                Element::Compass,
                Element::Prompt,
                Element::Markers,
                Element::Integrity,
            ]),
            &[Element::Speed, Element::Spool],
        ),
    };
    Set(base.0 | Set::of(extra).0)
}

/// Returns whether a marker of `kind` shows in `context`: in warp only the course, in space no
/// pins, and herds only where the herd element shows.
pub fn marker_shown(context: Context, kind: MarkerKind) -> bool {
    match kind {
        MarkerKind::Herd => shown(context).contains(Element::Herds),
        _ if context == Context::Warp => kind == MarkerKind::Course,
        MarkerKind::Waypoint | MarkerKind::Favourite | MarkerKind::Tool => !context.in_space(),
        _ => true,
    }
}
