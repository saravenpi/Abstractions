//! Feeding a trail at a chosen spacing: the wingsuit lays a point every 1.75 m, a ship, which
//! flies ten times as fast, every few metres.

use bevy::math::DVec3;

use super::trail::{Sample, Trail};

impl Trail {
    /// Moves the emitter to `at` after `dt` seconds, glowing at `heat`.
    ///
    /// Every point ages. New points are laid every `spacing` metres along the straight line
    /// from the newest point to the emitter, however long the frame was, so the spacing does
    /// not depend on the frame rate. A jump longer than `jump` metres starts the trail over.
    /// A faster emitter, such as a ship, takes a longer spacing and a longer jump.
    pub fn feed_by(&mut self, at: DVec3, heat: f32, dt: f32, (spacing, jump): (f64, f64)) {
        for sample in &mut self.ring {
            sample.age += dt;
        }
        let from = self.point(0);
        self.tip = Sample { at, age: 0.0, heat };
        let gap = from.at.distance(at);
        if self.is_empty() || gap > jump {
            self.len = 0;
            self.push(self.tip);
            return;
        }
        for step in 1..=(gap / spacing) as usize {
            let along = (step as f64 * spacing / gap) as f32;
            self.push(Sample {
                at: from.at.lerp(at, f64::from(along)),
                age: dt * (1.0 - along),
                heat: from.heat + (heat - from.heat) * along,
            });
        }
    }
}
