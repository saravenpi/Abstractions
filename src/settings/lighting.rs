//! The lighting options of the graphics settings, and the four presets that set them together
//! with the shadows, the ambient occlusion and the bloom of [`Graphics`].
//!
//! Every option only decides how much of the light the game computes. The lights themselves,
//! the sun, the moon, the lamps and the glow of living things, are always there.

use serde::{Deserialize, Serialize};

use super::model::Graphics;
use super::shadows::{ShadowDetail, ShadowReach};

/// A lighting preset. `Custom` is what the options read when they match none of the four.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Quality {
    Low,
    Medium,
    High,
    Ultra,
    Custom,
}

impl Quality {
    /// The four presets, from the lightest to the heaviest.
    pub const PRESETS: [Quality; 4] =
        [Quality::Low, Quality::Medium, Quality::High, Quality::Ultra];

    /// Returns the name shown in the menu.
    pub fn name(self) -> &'static str {
        match self {
            Quality::Low => "Low",
            Quality::Medium => "Medium",
            Quality::High => "High",
            Quality::Ultra => "Ultra",
            Quality::Custom => "Custom",
        }
    }
}

/// The lighting options that [`Graphics`] does not already hold.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(default)]
pub struct Lighting {
    /// Size of the shadow maps.
    pub shadow_detail: ShadowDetail,
    /// How far the sun casts shadows.
    pub shadow_reach: ShadowReach,
    /// Whether the clouds shade the ground, the plants and the props.
    pub cloud_shadows: bool,
    /// Whether the camera exposure follows the light, as an eye adapts. Off keeps one exposure.
    pub auto_exposure: bool,
    /// Whether the camera adapts its white to the colour of the light, as an eye does.
    pub white_balance: bool,
    /// Whether the lamp of the robot casts shadows.
    pub local_shadows: bool,
    /// Whether the sea shows the glint of the sun and the moon, not only the sky.
    pub water_glint: bool,
}

impl Default for Lighting {
    fn default() -> Self {
        Self {
            shadow_detail: ShadowDetail::High,
            shadow_reach: ShadowReach::Normal,
            cloud_shadows: true,
            auto_exposure: true,
            white_balance: true,
            local_shadows: false,
            water_glint: true,
        }
    }
}

impl Graphics {
    /// Returns the options a preset sets, or `self` unchanged for `Custom`. The render distance,
    /// the clouds in the sky, the vsync and the field of view are not lighting and stay.
    pub fn with_quality(&self, quality: Quality) -> Graphics {
        let (shadows, detail, reach) = match quality {
            Quality::Low => (false, ShadowDetail::Low, ShadowReach::Near),
            Quality::Medium => (true, ShadowDetail::Low, ShadowReach::Normal),
            Quality::High => (true, ShadowDetail::High, ShadowReach::Normal),
            Quality::Ultra => (true, ShadowDetail::High, ShadowReach::Far),
            Quality::Custom => return self.clone(),
        };
        let high = matches!(quality, Quality::High | Quality::Ultra);
        Graphics {
            shadows,
            ambient_occlusion: high,
            bloom: quality != Quality::Low,
            lighting: Lighting {
                shadow_detail: detail,
                shadow_reach: reach,
                cloud_shadows: high,
                auto_exposure: quality != Quality::Low,
                white_balance: quality != Quality::Low,
                local_shadows: quality == Quality::Ultra,
                water_glint: quality != Quality::Low,
            },
            ..self.clone()
        }
    }

    /// Returns the preset whose options these are, or `Custom`.
    pub fn quality(&self) -> Quality {
        let same = |quality: Quality| self.with_quality(quality) == *self;
        Quality::PRESETS
            .into_iter()
            .find(|&quality| same(quality))
            .unwrap_or(Quality::Custom)
    }

    /// Moves to the preset `dir` steps away, staying on the first and the last. From `Custom`
    /// a step goes to the nearest end of the way it points.
    pub fn step_quality(&mut self, dir: i32) {
        let last = Quality::PRESETS.len() as i32 - 1;
        let at = Quality::PRESETS.iter().position(|&q| q == self.quality());
        let next = match at {
            Some(at) => (at as i32 + dir).clamp(0, last),
            None if dir < 0 => 0,
            None => last,
        };
        *self = self.with_quality(Quality::PRESETS[next as usize]);
    }
}
