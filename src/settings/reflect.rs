//! What calm water mirrors besides the sky, and how a settings file that predates the option
//! reads it.

use serde::{Deserialize, Serialize};

use super::lighting::Lighting;
use super::shadows::{ShadowDetail, ShadowReach};

/// What calm water mirrors besides the sky. A full mirror of the ground and the clouds is not
/// offered: it would draw the terrain a second time, the biggest cost of a frame.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Reflections {
    /// The sky only, and no extra pass.
    Off,
    /// The robot, the companion, the ship, other players and the animals near the water.
    #[default]
    Actors,
}

impl Reflections {
    /// The options in the order the menu steps through them.
    pub const ALL: [Reflections; 2] = [Reflections::Off, Reflections::Actors];

    /// Returns the name shown in the menu.
    pub fn name(self) -> &'static str {
        match self {
            Reflections::Off => "Off",
            Reflections::Actors => "Actors",
        }
    }

    /// Returns the option a preset with shadow maps of `detail` sets: the actors on High and
    /// Ultra, whose shadows are fine, nothing on Low and Medium.
    pub fn of(detail: ShadowDetail) -> Self {
        match detail {
            ShadowDetail::Low => Reflections::Off,
            ShadowDetail::High => Reflections::Actors,
        }
    }

    /// Returns the next option, back to the first after the last: the menu row cycles.
    pub fn next(self) -> Self {
        let at = Self::ALL.iter().position(|&r| r == self).unwrap_or(0);
        Self::ALL[(at + 1) % Self::ALL.len()]
    }
}

/// The lighting options as a file holds them. A file written before the water reflections
/// existed has no `reflections` key: it takes what its preset sets, read from its shadow detail,
/// so a file on a preset still reads as that preset.
#[derive(Deserialize, Debug, Clone, Copy)]
#[serde(default)]
pub struct Stored {
    /// See [`Lighting::shadow_detail`].
    shadow_detail: ShadowDetail,
    /// See [`Lighting::shadow_reach`].
    shadow_reach: ShadowReach,
    /// See [`Lighting::cloud_shade`].
    cloud_shade: bool,
    /// See [`Lighting::auto_exposure`].
    auto_exposure: bool,
    /// See [`Lighting::white_balance`].
    white_balance: bool,
    /// See [`Lighting::local_shadows`].
    local_shadows: bool,
    /// See [`Lighting::water_glint`].
    water_glint: bool,
    /// See [`Lighting::reflections`]; none in a file that predates it.
    reflections: Option<Reflections>,
}

impl Default for Stored {
    /// The default lighting, with no reflections key.
    fn default() -> Self {
        let own = Lighting::default();
        Self {
            shadow_detail: own.shadow_detail,
            shadow_reach: own.shadow_reach,
            cloud_shade: own.cloud_shade,
            auto_exposure: own.auto_exposure,
            white_balance: own.white_balance,
            local_shadows: own.local_shadows,
            water_glint: own.water_glint,
            reflections: None,
        }
    }
}

impl From<Stored> for Lighting {
    /// Reads the lighting of a file, the reflections of its preset where it has none.
    fn from(stored: Stored) -> Self {
        Lighting {
            shadow_detail: stored.shadow_detail,
            shadow_reach: stored.shadow_reach,
            cloud_shade: stored.cloud_shade,
            auto_exposure: stored.auto_exposure,
            white_balance: stored.white_balance,
            local_shadows: stored.local_shadows,
            water_glint: stored.water_glint,
            reflections: stored
                .reflections
                .unwrap_or_else(|| Reflections::of(stored.shadow_detail)),
        }
    }
}
