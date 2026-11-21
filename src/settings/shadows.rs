//! The two shadow options of the lighting settings: how fine the shadow maps are and how far
//! the sun casts shadows.

use serde::{Deserialize, Serialize};

/// How fine the shadow maps are.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ShadowDetail {
    Low,
    #[default]
    High,
}

impl ShadowDetail {
    /// Returns the side of one shadow map in texels.
    pub fn map_size(self) -> usize {
        match self {
            ShadowDetail::Low => 1024,
            ShadowDetail::High => 2048,
        }
    }

    /// Returns the name shown in the menu.
    pub fn name(self) -> &'static str {
        match self {
            ShadowDetail::Low => "Low",
            ShadowDetail::High => "High",
        }
    }
}

/// How far from the camera the sun casts shadows.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ShadowReach {
    Near,
    #[default]
    Normal,
    Far,
}

impl ShadowReach {
    /// Returns the distance in metres where the last shadow cascade ends.
    pub fn metres(self) -> f32 {
        match self {
            ShadowReach::Near => 55.0,
            ShadowReach::Normal => 110.0,
            ShadowReach::Far => 220.0,
        }
    }

    /// Returns the number of shadow cascades. The first always ends 12 m from the camera, so the
    /// robot and the companion cast sharp shadows; the others carry the reach, each one about three
    /// times as far as the one before: 12 and 55 m near, 12, 36 and 110 m normal, 12, 32, 84 and
    /// 220 m far. The near cascade draws only what stands within its 12 m.
    pub fn cascades(self) -> usize {
        match self {
            ShadowReach::Near => 2,
            ShadowReach::Normal => 3,
            ShadowReach::Far => 4,
        }
    }

    /// Returns the number of shadow cascades for this reach at shadow `detail`. Low detail drops
    /// one cascade (1 on Near, 2 on Normal, 3 on Far) to cut shadow pass cost; High keeps
    /// [`Self::cascades`].
    pub fn cascades_for(self, detail: ShadowDetail) -> usize {
        match detail {
            ShadowDetail::Low => (self.cascades() - 1).max(1),
            ShadowDetail::High => self.cascades(),
        }
    }

    /// Returns the name shown in the menu.
    pub fn name(self) -> &'static str {
        match self {
            ShadowReach::Near => "55 m",
            ShadowReach::Normal => "110 m",
            ShadowReach::Far => "220 m",
        }
    }
}
