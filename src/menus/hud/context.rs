//! Where the player is, as far as the HUD cares, and what shows there: [`Context`], derived once
//! per frame from the facts, and [`shown`], the one rule from a context to the set of
//! elements that mean something there. Every element asks it, so a reading that is useless in
//! the place the player stands in never shows.

mod elements;
mod here;

#[cfg(test)]
pub use elements::Set;
pub use elements::{Element, marker_shown, shown};
pub use here::{Where, shows_climate, shows_compass, shows_coords, update};

use crate::ship::WarpState;
use crate::space::Frame;

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

impl Context {
    /// Every context.
    #[cfg(test)]
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
