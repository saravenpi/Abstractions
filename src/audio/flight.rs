//! The values of the flight gear and of the air the game hands to the audio thread.

use super::scene::AudioScene;
use super::shared::{Param, Shared, unit};

impl Shared {
    /// Stores what the flight gear sounds follow. The rush, the cloth and the buffeting of the air
    /// are not heard in space, where there is none: `outside` is 0 there and 1 anywhere else.
    pub(super) fn publish_flight(&self, scene: &AudioScene, outside: f32) {
        self.set(Param::Boost, unit(scene.boost));
        let aloft = [
            (Param::Rush, scene.rush),
            (Param::Flutter, scene.flutter),
            (Param::Speed, scene.speed),
            (Param::Buffet, scene.buffet),
        ];
        for (param, level) in aloft {
            self.set(param, unit(level * outside));
        }
    }

    /// Stores what the air carries: the wind, the shore, the rain and the dust, as loud as the air
    /// of the planet carries them and none of the rain and dust in space, where `outside` is 0,
    /// and how the air around the listener takes the high end with distance.
    pub(super) fn publish_air(&self, scene: &AudioScene, outside: f32) {
        let air = scene.air;
        self.set(Param::Wind, unit(scene.wind * scene.altitude_fade * air));
        self.set(Param::Shore, unit(scene.shore * air));
        self.set(Param::ShoreSide, unit(0.5 + 0.5 * scene.shore_side));
        for (param, level) in [(Param::Rain, scene.rain), (Param::Dust, scene.dust)] {
            self.set(param, unit(level * scene.altitude_fade * air * outside));
        }
        self.set(Param::AirFar, scene.absorb.far);
        self.set(Param::AirSlope, scene.absorb.slope);
    }
}
