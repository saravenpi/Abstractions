//! Shared setups of the shedding tests: a plant at a stage of its year and the air it stands in.

use super::rate::Air;
use crate::life::flora::Stage;
use crate::world::canopy::Shed;

/// Returns a plant at `stage` of its year, in leaf, in flower when `flower`.
pub fn plant(deciduous: bool, stage: Stage, flower: bool) -> Shed {
    Shed {
        leaf: Some([0.1, 0.4, 0.05]),
        size: 0.1,
        flower: flower.then_some([1.0, 0.8, 0.8]),
        stage,
        deciduous,
        stress: 0.0,
    }
}

/// Returns home air blowing at `speed` metres per second under `rain`.
pub fn home(speed: f32, rain: f32) -> Air {
    Air {
        speed,
        thickness: 1.0,
        rain,
    }
}
