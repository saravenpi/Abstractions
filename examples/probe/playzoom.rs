//! The map of a planet at every zoom: the map opened on the ground explored around the base,
//! looked at from the closest zoom out to the first zoom with tiles, then panned at the closest
//! zoom, with what the tiles cover said at each: shots `e20` to `e28`.

use abstractions::map::Surface;
use abstractions::map::raster::Tiles;

use super::*;

/// The name of the stage in the log.
const STAGE: &str = "map-zoom";

/// The zooms looked at, from the closest out: about 490 m, 2 km, 10 km, 70 km and 700 km
/// across on a planet the size of Earth, and the first zoom with tiles.
const ZOOMS: [f32; 6] = [81_920.0, 20_000.0, 4_000.0, 600.0, 60.0, 3.2];

/// The pans at the closest zoom, in widths of the view.
const PANS: [f64; 2] = [0.6, 1.5];

/// Frames after a pan at which the map is shot while its new tiles are still painted.
const EARLY: u32 = 20;

/// Returns what the tiles of the open map draw, in words: whether they cover the view, and
/// the level of each tile drawn.
fn cover_words(world: &World) -> String {
    let lens = world.resource::<Surface>().lens;
    let (drawn, covered) = world.resource::<Tiles>().ready(lens);
    let levels: Vec<u8> = drawn.iter().map(|(_, id, _)| id.level).collect();
    format!(
        "tiles cover the view: {covered}, {} drawn at levels {levels:?}",
        drawn.len()
    )
}

/// Looks at the zoom of step `index` of [`ZOOMS`], and shots the map once it settled.
fn zoom_step(world: &mut World, play: &mut Play, frame: u32, index: usize) {
    let zoom = ZOOMS[index];
    if play.held(frame) == 1 {
        world.resource_mut::<Surface>().goal.zoom = zoom;
    }
    if settled(world, play, frame, (150, 20.0)) {
        let words = format!("{} {}", map_words(world), cover_words(world));
        say(STAGE, &format!("e2{} zoom {zoom}: {words}", index + 1));
        shot(world, &format!("e2{}-map-zoom-{zoom:.0}", index + 1));
        play.next(frame);
    }
}

/// Pans the closest zoom by pan `index` of [`PANS`], shots the map while its new tiles are
/// painted and once it settled.
fn pan_step(world: &mut World, play: &mut Play, frame: u32, index: usize) {
    let held = play.held(frame);
    let shown = format!("e{}", 27 + index);
    if held == 1 {
        let mut surface = world.resource_mut::<Surface>();
        surface.goal.zoom = ZOOMS[0];
        surface.goal.centre.x += PANS[index] * 2.0 / f64::from(ZOOMS[0]);
    }
    if held == EARLY {
        say(STAGE, &format!("{shown} early: {}", cover_words(world)));
        shot(world, &format!("{shown}a-map-pan-early"));
    }
    if held > EARLY && settled(world, play, frame, (150, 20.0)) {
        say(STAGE, &format!("{shown} baked: {}", cover_words(world)));
        shot(world, &format!("{shown}b-map-pan-baked"));
        play.next(frame);
    }
}

/// Steps through the stage: the ground explored, the map opened, each zoom, each pan, and the
/// map closed.
fn zoom_steps(world: &mut World, play: &mut Play, frame: u32) -> Option<Act> {
    let held = play.held(frame);
    let map_key = bound(world, |s| s.controls.keys.map);
    match play.step {
        0 if held == 40 => {
            let facts = explore(world)?;
            say(STAGE, &facts);
            play.next(frame);
        }
        1 => {
            press(world, map_key, held);
            if settled(world, play, frame, (150, 20.0)) {
                say(
                    STAGE,
                    &format!("e20 the map as it opens: {}", map_words(world)),
                );
                shot(world, "e20-map-open");
                play.next(frame);
            }
        }
        step @ 2..=7 => zoom_step(world, play, frame, step as usize - 2),
        step @ 8..=9 => pan_step(world, play, frame, step as usize - 8),
        10 => {
            press(world, map_key, held);
            if held >= 30 {
                tidy(world);
                return Some(Act::Next);
            }
        }
        _ => {}
    }
    None
}

/// The map of a planet from the closest zoom out to the first zoom with tiles, then panned at
/// the closest: shots `e20` to `e28`. No square of the view is ever empty, while tiles are
/// painted a coarser one stands in, and the coarse cells of the whole picture never show
/// through.
pub fn map_zoom(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    if frame == 1 {
        notes.play.start(frame);
        view(world, ViewMode::First, ViewMode::First);
        if let Some(base) = world.get_resource::<Base>().copied() {
            let feet = ground(world, base.wake, 0.0);
            stand(world, feet, base.pad, 0.0);
        }
    }
    if frame > 6000 {
        return gave_up(
            world,
            STAGE,
            "the map did not get through its zooms in 6000 frames",
        );
    }
    let ended = zoom_steps(world, &mut notes.play, frame);
    match (frame, ended) {
        (_, Some(act)) => act,
        (41.., None) if notes.play.step == 0 && world.get_resource::<Base>().is_none() => {
            gave_up(world, STAGE, "this world has no home base")
        }
        _ => Act::Wait,
    }
}
