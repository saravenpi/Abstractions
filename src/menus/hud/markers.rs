//! Markers: every point the compass, the minimap and the map show besides the player.
//!
//! Each system that knows of a place to show (home, the ship, friends, waypoints) rewrites its
//! own markers every frame in [`MarkerSet::Fill`]; the compass, the minimap and the map only
//! read them, after that set.

use bevy::math::DVec3;
use bevy::prelude::*;

use crate::universe::BodyId;

/// What a marker stands for, which chooses its icon.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MarkerKind {
    /// The home base.
    Home,
    /// The player's own ship.
    Ship,
    /// Another player.
    Friend,
    /// A waypoint, the player's own or a friend's.
    Waypoint,
    /// A favourite place.
    Favourite,
    /// A herd of animals near the player.
    Herd,
    /// The place the course is set to: a body, a star or a galaxy, at any distance.
    Course,
}

/// Where a marker is: on a body, in the body's own turning frame, or only a direction.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MarkerPlace {
    /// A point fixed to a planet or moon, in metres from its centre, in its turning frame.
    Body(BodyId, DVec3),
    /// A point given in the frame of the anchor at this frame, in metres.
    Anchor(DVec3),
}

/// One marker.
#[derive(Debug, Clone, PartialEq)]
pub struct Marker {
    /// What it stands for.
    pub kind: MarkerKind,
    /// Where it is.
    pub place: MarkerPlace,
    /// Its colour. White for markers that have none of their own.
    pub color: Color,
    /// Its label: a friend's name or a waypoint's name. Empty for none.
    pub label: String,
    /// Who it belongs to, so a filler can rewrite its own: a player id, or 0 for the game.
    pub owner: u32,
}

impl Marker {
    /// Returns whether the player follows this marker on purpose: the course, or their own pin.
    /// Such a marker is never reduced, hides only on arrival and shows at any distance.
    pub fn tracked(&self) -> bool {
        self.kind == MarkerKind::Course || (self.kind == MarkerKind::Waypoint && self.owner == 0)
    }

    /// Returns the place of this marker in the order that settles who is drawn in front when
    /// two meet, over the world and on the compass: the course, the followed pin, friends, home,
    /// the ship, pins, herds.
    pub fn rank(&self) -> u8 {
        match self.kind {
            MarkerKind::Course => 0,
            MarkerKind::Waypoint if self.owner == 0 => 1,
            MarkerKind::Friend => 2,
            MarkerKind::Home => 3,
            MarkerKind::Ship => 4,
            MarkerKind::Waypoint | MarkerKind::Favourite => 5,
            MarkerKind::Herd => 6,
        }
    }
}

/// Every marker of this frame.
#[derive(Resource, Debug, Default, Clone)]
pub struct Markers(pub Vec<Marker>);

impl Markers {
    /// Removes every marker of `kind` that belongs to `owner`, so a filler can write its own anew.
    pub fn clear(&mut self, kind: MarkerKind, owner: u32) {
        self.0
            .retain(|marker| marker.kind != kind || marker.owner != owner);
    }

    /// Removes every marker of `kind`, whoever it belongs to.
    pub fn clear_kind(&mut self, kind: MarkerKind) {
        self.0.retain(|marker| marker.kind != kind);
    }
}

/// When markers are written and read in `Update`.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MarkerSet {
    /// Fillers rewrite their markers.
    Fill,
    /// The compass, the minimap and the map read them.
    Show,
}

/// Registers the markers and orders filling before showing.
pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Markers>()
        .configure_sets(Update, MarkerSet::Fill.before(MarkerSet::Show));
}
