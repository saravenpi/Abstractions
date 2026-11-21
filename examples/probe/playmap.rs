//! The map of ground that was explored: the land around the base as the map opens on it, the
//! region from further out, the system, the card of a planet and the card of a pin.

use abstractions::computer::{Board, Computer, MapView};
use abstractions::map::explored::{Explored, reach};
use abstractions::map::raster::Rasters;
use abstractions::map::{Level, Places, Surface};
use abstractions::menus::Markers;

use super::*;

/// The name of the stage in the log.
const STAGE: &str = "map";

/// Returns the way from the base to the nearest sea within 12 km, with its distance in metres,
/// or the first axis of the base when no sea is that near.
fn shore_way(world: &World, base: &Base) -> (DVec3, Option<f64>) {
    let globe = &world.resource::<Planet>().globe;
    let wet = |at: DVec3| globe.height(at.normalize()) < globe.sea_level();
    let mut best: Option<(f64, DVec3)> = None;
    for way in compass(base.centre, 16) {
        let mut steps = (1..=48).map(|step| f64::from(step) * 250.0);
        let found = steps.find(|m| wet(base.centre + way * *m));
        if let Some(metres) = found
            && best.is_none_or(|(least, _)| metres < least)
        {
            best = Some((metres, way));
        }
    }
    match best {
        Some((metres, way)) => (way, Some(metres)),
        None => (base.east, None),
    }
}

/// Uncovers the ground a flight 200 m up would: a loop 3 km around the base and a leg to the
/// nearest shore, through the function that uncovers the ground under the player. Returns what
/// it did in words, or nothing without a base.
pub fn explore(world: &mut World) -> Option<String> {
    let base = world.get_resource::<Base>().copied()?;
    let radius = world.resource::<Planet>().globe.radius();
    let (way, shore) = shore_way(world, &base);
    let angle = reach(200.0, radius) / radius;
    let leg = shore.map_or(8000.0, |metres| metres + 2000.0);
    let mut explored = world.resource_mut::<Explored>();
    let before = explored.leaves(base.body).map_or(0, |leaves| leaves.len());
    for around in compass(base.centre, 48) {
        let at = (base.centre + around * 3000.0).normalize();
        explored.reveal(base.body, at, angle, radius);
    }
    let mut out = 0.0;
    while out <= leg {
        explored.reveal(
            base.body,
            (base.centre + way * out).normalize(),
            angle,
            radius,
        );
        out += 500.0;
    }
    let after = explored.leaves(base.body).map_or(0, |leaves| leaves.len());
    Some(format!(
        "{before} cells of about 500 m were explored around the base at the start; a flight 200 m \
         up over a loop 3 km out and a leg of {leg:.0} m (sea at {shore:?} m) uncovers {after} cells, \
         on a planet of radius {:.0} km",
        radius / 1000.0
    ))
}

/// Returns true once the open map has had `frames` frames and `seconds` of the wall clock to
/// paint its picture on its workers, or 900 frames whatever the clock says.
pub fn settled(world: &World, play: &Play, frame: u32, (frames, seconds): (u32, f32)) -> bool {
    let open = *world.resource::<State<Computer>>().get() == Computer::Open;
    let held = play.held(frame);
    open && ((held >= frames && play.waited() >= seconds) || held >= 900)
}

/// Returns what the open map shows, in words: its level, its zoom, whether its picture is ready,
/// the explored cells and the markers.
pub fn map_words(world: &World) -> String {
    let level = *world.resource::<Level>();
    let surface = world.resource::<Surface>();
    let body = world.resource::<Location>().body;
    let explored = world.resource::<Explored>();
    let cells = explored.leaves(body).map_or(0, |leaves| leaves.len());
    let picture = world.resource::<Rasters>().image(body).is_some();
    let markers: Vec<String> = world
        .resource::<Markers>()
        .0
        .iter()
        .map(|marker| format!("{:?} {}", marker.kind, marker.label))
        .collect();
    let pins = world.resource::<Places>().pins.len();
    format!(
        "level {level:?}, zoom {:.0} (pointer over the map: {}), picture ready: {picture}, {cells} \
         cells explored, {pins} pins, markers [{}]",
        surface.lens.zoom,
        surface.over,
        markers.join(", ")
    )
}

/// Zooms the map out to see the region: four notches of the wheel, or the same through the lens
/// when the pointer of the probe is not over the map.
fn zoom_out(world: &mut World, held: u32) {
    if held == 1 {
        wheel(world, -4.0);
    }
    let goal = world.resource::<Surface>().goal.zoom;
    if held == 20 && goal > 4000.0 {
        say(
            STAGE,
            "the wheel did not zoom (pointer off the map): zoom set through the lens",
        );
        world.resource_mut::<Surface>().goal.zoom = goal * 0.19;
    }
}

/// Selects the last pin the way a click on its row does: it is the selected pin, no row of the
/// list is picked, and the map looks at it on its planet. Returns what it selected, in words.
fn select_pin(world: &mut World) -> String {
    let (index, pin) = {
        let places = world.resource::<Places>();
        let Some(pin) = places.pins.last().cloned() else {
            return "no pin to select".to_string();
        };
        (places.pins.len() - 1, pin)
    };
    world.resource_mut::<Places>().selected = Some(index);
    world.resource_mut::<MapView>().picked = false;
    world.resource_mut::<Surface>().focus = Some(DVec3::from_array(pin.dir));
    *world.resource_mut::<Level>() = Level::Planet(pin.body);
    let radius = world.resource::<Planet>().globe.radius();
    let away = (DVec3::from_array(pin.dir) * radius).distance(robot(world).normalize() * radius);
    format!("selected {} {away:.0} m from the robot", pin.name)
}

/// The first steps of the stage: the ground explored and a pin dropped, the map opened on the
/// ground, then zoomed out over the region.
fn ground_steps(world: &mut World, play: &mut Play, frame: u32) -> Option<Act> {
    let held = play.held(frame);
    let map_key = bound(world, |s| s.controls.keys.map);
    let step = play.step;
    match step {
        0 if held == 40 => {
            let Some(facts) = explore(world) else {
                return Some(gave_up(world, STAGE, "this world has no home base"));
            };
            say(STAGE, &facts);
            let pin = pin_near_base(world, 600.0);
            say(
                STAGE,
                &format!("dropped pin {pin:?} 600 m from the wake point"),
            );
            play.next(frame);
        }
        1 => {
            press(world, map_key, held);
            if settled(world, play, frame, (150, 20.0)) {
                say(
                    STAGE,
                    &format!("e10 the map as it opens: {}", map_words(world)),
                );
                shot(world, "e10-map-ground");
                play.next(frame);
            }
        }
        2 => {
            zoom_out(world, held);
            if settled(world, play, frame, (150, 20.0)) {
                say(STAGE, &format!("e11 zoomed out: {}", map_words(world)));
                shot(world, "e11-map-region");
                play.next(frame);
            }
        }
        _ => {}
    }
    None
}

/// The last steps: the system chart on Backspace, a planet selected for its card, the pin
/// selected for its card, and the map closed.
fn chart_steps(world: &mut World, play: &mut Play, frame: u32) -> Option<Act> {
    let held = play.held(frame);
    let map_key = bound(world, |s| s.controls.keys.map);
    let other = |target: Target, here: BodyId| matches!(target, Target::Body(body) if body != here && body.moon == 0);
    let step = play.step;
    match step {
        3 => {
            press(world, KeyCode::Backspace, held);
            if held == 70 {
                let rows = world.resource::<Board>().lines.len();
                say(
                    STAGE,
                    &format!("e12 the system: {rows} rows, {}", map_words(world)),
                );
                shot(world, "e12-map-system");
                play.next(frame);
            }
        }
        4 if held == 1 => choose_row(world, "play map e13", other),
        4 if held == 60 => {
            shot(world, "e13-map-planet-card");
            play.next(frame);
        }
        5 if held == 1 => {
            let facts = select_pin(world);
            say(STAGE, &format!("e14 the pin card: {facts}"));
        }
        5 if settled(world, play, frame, (150, 20.0)) => {
            say(STAGE, &format!("e14 {}", map_words(world)));
            shot(world, "e14-map-pin-card");
            play.next(frame);
        }
        6 => {
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

/// The map of explored ground, from the land around the base out to the system, with the card
/// of a planet and the card of a pin: shots `e10` to `e14`.
pub fn play_map(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    if frame == 1 {
        notes.play.start(frame);
        view(world, ViewMode::First, ViewMode::First);
        if let Some(base) = world.get_resource::<Base>().copied() {
            let feet = ground(world, base.wake, 0.0);
            stand(world, feet, base.pad, 0.0);
        }
    }
    if frame > 4000 {
        return gave_up(
            world,
            STAGE,
            "the map did not get through its steps in 4000 frames",
        );
    }
    let play = &mut notes.play;
    let step = play.step;
    let ended = match step {
        0..=2 => ground_steps(world, play, frame),
        _ => chart_steps(world, play, frame),
    };
    ended.unwrap_or(Act::Wait)
}
