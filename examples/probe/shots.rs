//! Stages that look at one kind of animal.

use super::*;

/// The nearest walker from 8 m, first person.
pub fn animal_8(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    animal_shot(
        world,
        notes,
        frame,
        (15.0, ViewMode::First, "03a-animal-15", big_walker),
    )
}

/// The nearest walker from 8 m, third person.
pub fn animal_8_tp(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    animal_shot(
        world,
        notes,
        frame,
        (15.0, ViewMode::Third, "03b-animal-15-tp", big_walker),
    )
}

/// The nearest walker from 25 m, first person.
pub fn animal_25(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    animal_shot(
        world,
        notes,
        frame,
        (35.0, ViewMode::First, "03c-animal-35", big_walker),
    )
}

/// The nearest flyer from 20 m along the ground, looking up at it.
pub fn sky_flyer(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    animal_shot(
        world,
        notes,
        frame,
        (20.0, ViewMode::First, "03d-sky", flyer),
    )
}

/// The nearest swarm from 5 m.
pub fn swarm_near(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    animal_shot(
        world,
        notes,
        frame,
        (5.0, ViewMode::First, "03e-swarm", swarmer),
    )
}

/// Half a day later, the nearest glowing animal from 8 m.
pub fn night_glow(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    if frame == 1 {
        let day = world.resource::<Planet>().day_seconds;
        world.resource_mut::<abstractions::space::Clock>().seconds += day * 0.5;
    }
    if frame < 90 {
        return Act::Wait;
    }
    animal_shot(
        world,
        notes,
        frame - 89,
        (8.0, ViewMode::First, "03f-night", glowing),
    )
}
