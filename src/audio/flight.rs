//! The values of the flight gear the game hands to the audio thread.

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
}
