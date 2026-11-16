//! The wind: one slowly turning direction and a gusting strength shared by clouds, water and particles.

use bevy::prelude::*;

/// Seconds for the wind direction to turn a full circle.
const TURN_SECONDS: f32 = 900.0;

/// Seconds between two gust peaks.
const GUST_SECONDS: f32 = 23.0;

/// The wind blowing over the world.
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct Wind {
    /// Unit direction the wind blows toward, on the XZ plane.
    pub direction: Vec2,
    /// Strength from 0 (still) to 1 (strong gust).
    pub strength: f32,
}

impl Default for Wind {
    fn default() -> Self {
        wind_at(0.0)
    }
}

impl Wind {
    /// Returns the wind as a world-space velocity in meters per second.
    pub fn velocity(&self) -> Vec3 {
        Vec3::new(self.direction.x, 0.0, self.direction.y) * (1.0 + 5.0 * self.strength)
    }
}

/// Returns the wind at `seconds` of game time.
pub fn wind_at(seconds: f32) -> Wind {
    let angle = seconds / TURN_SECONDS * std::f32::consts::TAU + 0.7;
    let gust = (seconds / GUST_SECONDS * std::f32::consts::TAU).sin();
    let swell = (seconds / (GUST_SECONDS * 3.7) * std::f32::consts::TAU).sin();
    Wind {
        direction: Vec2::from_angle(angle),
        strength: (0.45 + 0.3 * gust + 0.2 * swell).clamp(0.0, 1.0),
    }
}

/// Advances the wind with the game clock.
fn blow(time: Res<Time>, mut wind: ResMut<Wind>) {
    *wind = wind_at(time.elapsed_secs());
}

/// Registers the wind resource and its system.
pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Wind>().add_systems(Update, blow);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The wind direction stays a unit vector and the strength stays in range.
    #[test]
    fn wind_stays_in_range() {
        for step in 0..500 {
            let wind = wind_at(step as f32 * 7.3);
            assert!((wind.direction.length() - 1.0).abs() < 1e-4);
            assert!((0.0..=1.0).contains(&wind.strength));
        }
    }
}
