//! Bringing a world file from an older version up to date.
//!
//! Nothing but the seed stores the ground or the home base, so a version of the game that moves
//! them leaves an old file standing where they were. Version 2 is the ground of 0.9: the base
//! is looked for around a mild northern latitude instead of the pole, and every planet has new
//! relief and rivers.

use bevy::math::DVec3;

use super::model::{Place, SAVE_VERSION, WorldSave};
use crate::player::{BODY_LENGTH, BODY_RADIUS};
use crate::ship::{SavedFrame, ShipState};
use crate::universe::{BodyId, Universe};
use crate::world::Globe;

/// Distance in metres from the hull centre of the ship down to the feet of its gear, as the
/// ship parks itself: its own `GEAR_DROP`.
const GEAR_DROP: f64 = 1.25;

/// Metres a landed ship may stand off its ground before the upgrade seats it again.
const SEATED: f64 = 0.05;

/// The ground of a planet in one direction, as an upgrade needs it.
struct Floor {
    /// Sea level radius of the planet in metres.
    radius: f64,
    /// Height of the ground above sea level in metres.
    ground: f64,
    /// Height of what carries a ship there: the ground, or the sea over it.
    top: f64,
}

impl Floor {
    /// Returns the ground of `body` in direction `dir`, or nothing for a direction without
    /// length or a body without ground.
    fn under(universe: &Universe, body: BodyId, dir: DVec3) -> Option<(DVec3, Self)> {
        let dir = dir.try_normalize()?;
        let globe = Globe::of(&universe.planet(body));
        let ground = f64::from(globe.height(dir));
        let floor = Self {
            radius: globe.radius(),
            ground,
            top: ground.max(f64::from(globe.sea_level())),
        };
        globe.has_ground().then_some((dir, floor))
    }
}

/// Returns `place` with its body standing on the ground of 0.9 when that ground is now higher
/// than its feet.
fn stood(universe: &Universe, place: Place) -> Place {
    let stand = f64::from(BODY_RADIUS + BODY_LENGTH * 0.5);
    let floor = Floor::under(universe, place.body, DVec3::from(place.dir));
    let altitude = floor.map_or(place.altitude, |(_, floor)| {
        place.altitude.max(floor.ground + stand)
    });
    Place { altitude, ..place }
}

/// Returns the record of a ship in the frame of a planet put back on the ground of 0.9: a
/// landed ship on its gear on the ground or the sea, a flying one above them. Nothing when the
/// ship is in space or already sits right.
fn seated(universe: &Universe, ship: &ShipState) -> Option<ShipState> {
    let SavedFrame::Surface(body) = ship.frame else {
        return None;
    };
    let pos = DVec3::from_array(ship.pos);
    let (dir, floor) = Floor::under(universe, body, pos)?;
    let rest = floor.radius + floor.top + GEAR_DROP;
    let height = match ship.landed {
        true => rest,
        false => pos.length().max(rest),
    };
    ((height - pos.length()).abs() > SEATED).then(|| ShipState {
        pos: (dir * height).to_array(),
        ..*ship
    })
}

impl WorldSave {
    /// Brings a file from an older version up to date. A file of this version or of a newer
    /// one is left as it is.
    ///
    /// A file from before v0.6 holds a position on the flat world, which means nothing on a
    /// globe: it is dropped, so the world starts at the home planet spawn. A file from before
    /// 0.9 stands on ground that has changed, see [`WorldSave::resettle`]. The name, the seed,
    /// the play time, the dates, the atlas and the log of changes are kept.
    pub fn upgrade(&mut self) {
        if self.version < 1 {
            self.position = None;
        }
        if self.version < 2 {
            self.resettle();
        }
        self.version = self.version.max(SAVE_VERSION);
    }

    /// Moves a file from before 0.9 onto the ground of 0.9.
    ///
    /// The home base moved, and nothing stores it. A file whose player stands on the home
    /// planet, or whose ship is there, forgets its place and its ship: the world then begins
    /// as a new game does, at the new base with a fresh ship berthed on its pad. A player
    /// seated in a ship that is somewhere else only last stood at home, and keeps flying.
    /// Anywhere else the place and the ship are kept and lifted onto the new ground.
    fn resettle(&mut self) {
        let universe = Universe::new(self.seed);
        let home = universe.home();
        let ship: Option<ShipState> = self.ship.clone().and_then(|ship| ship.try_into().ok());
        let berth = ship.map(|ship| ship.frame);
        let docked = berth == Some(SavedFrame::Surface(home));
        let away = ship.is_some_and(|ship| ship.seated) && !docked;
        if docked || (self.place.is_some_and(|place| place.body == home) && !away) {
            (self.place, self.position, self.ship) = (None, None, None);
            return;
        }
        self.place = self.place.map(|place| stood(&universe, place));
        let moved = ship.and_then(|ship| seated(&universe, &ship));
        if let Some(ship) = moved.and_then(|ship| toml::Value::try_from(ship).ok()) {
            self.ship = Some(ship);
        }
    }
}
