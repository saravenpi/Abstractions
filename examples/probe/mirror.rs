//! The water mirror: the robot at a lake shore and the ship hovering low over the water, from
//! behind the robot, from just above the water at a grazing angle, and looking down in first
//! person, each with the water reflections on (Actors) and off. Run it with `SMOKE_SEED=57`.

use abstractions::render::GameCamera;
use abstractions::settings::Reflections;

use super::*;

/// Frames each shot gets: its set-up on the first, the shot on the last.
const EACH: u32 = 40;

/// Frames the stage waits after its set-up before the first shot.
const SETTLE: u32 = 60;

/// Height in metres above the sea of the hovering ship.
const SHIP_HEIGHT: f64 = 5.0;

/// The shots: name, the reflections setting, and how the camera is held.
const SHOTS: [(&str, Reflections, Hold); 8] = [
    ("R1-shore-tp-actors", Reflections::Actors, Hold::Behind),
    ("R2-shore-tp-off", Reflections::Off, Hold::Behind),
    ("R3-graze-ship-actors", Reflections::Actors, Hold::Ship),
    ("R4-graze-ship-off", Reflections::Off, Hold::Ship),
    ("R5-graze-robot-actors", Reflections::Actors, Hold::Robot),
    ("R6-graze-robot-off", Reflections::Off, Hold::Robot),
    ("R7-fp-down-actors", Reflections::Actors, Hold::Down),
    ("R8-fp-down-off", Reflections::Off, Hold::Down),
];

/// Where the camera is for a shot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Hold {
    /// The third person rig behind the robot at the shore, looking out over the water.
    Behind,
    /// Held 0.5 m above the sea at the shore, looking out at the hovering ship.
    Ship,
    /// Held 0.6 m above the sea 12 m out, looking back at the robot.
    Robot,
    /// The first person rig of the robot wading at the waterline, looking down at the water.
    Down,
}

/// The places of the stage on the planet.
#[derive(Resource, Debug, Clone, Copy)]
struct Shore {
    /// Dry ground at the waterline, where the robot stands.
    land: DVec3,
    /// Shallow water just off it, where the robot wades for the first person shots.
    wade: DVec3,
    /// Open water a few metres out.
    sea: DVec3,
    /// The way out to sea, along the ground.
    out: DVec3,
}

/// Returns the first point from `from` to `to` whose ground height `pick` accepts.
fn march(world: &World, (from, to): (DVec3, DVec3), pick: impl Fn(f32) -> bool) -> Option<DVec3> {
    let globe = &world.resource::<Planet>().globe;
    (0..=400)
        .map(|i| from.lerp(to, f64::from(i) / 400.0))
        .find(|p| pick(globe.height(p.normalize())))
}

/// Finds a shore near the base: the open water, the dry ground just inland of it and the shallow
/// water between.
fn find(world: &World) -> Option<Shore> {
    let base = world.get_resource::<Base>()?;
    let radius = world.resource::<Planet>().globe.radius();
    let sea = find_shore(world, base.centre)? * radius;
    let land = march(world, (sea, base.centre), |h| h > 0.3)?.normalize() * radius;
    let wade = march(world, (land, sea), |h| h < -0.3)?.normalize() * radius;
    let up = sea.normalize();
    let flat = (sea - land) - up * (sea - land).dot(up);
    let out = flat.normalize_or(up.any_orthonormal_vector());
    Some(Shore {
        land,
        wade,
        sea,
        out,
    })
}

/// Holds the game camera at planet position `eye` looking at planet position `target`.
fn hold_camera(world: &mut World, eye: DVec3, target: DVec3) {
    let anchor = *world.resource::<Anchor>();
    let body = world
        .query_filtered::<&Transform, With<Player>>()
        .iter(world)
        .next()
        .copied();
    let Some(body) = body else {
        return;
    };
    let up = (anchor.rotation * eye.normalize()).as_vec3();
    let (from, to) = (anchor.to_render(eye), anchor.to_render(target));
    let pose = Transform::from_translation(from).looking_to(to - from, up);
    let back = body.rotation.inverse();
    for mut camera in world
        .query_filtered::<&mut Transform, With<GameCamera>>()
        .iter_mut(world)
    {
        camera.translation = back * (pose.translation - body.translation);
        camera.rotation = back * pose.rotation;
    }
}

/// Returns `p` lifted to `height` metres above the sea.
fn above(world: &World, p: DVec3, height: f64) -> DVec3 {
    p.normalize() * (world.resource::<Planet>().globe.radius() + height)
}

/// Sets the robot, the view and the reflections up for a shot.
fn set_up(world: &mut World, shore: &Shore, value: Reflections, hold: Hold) {
    world
        .resource_mut::<Persistent<Settings>>()
        .graphics
        .lighting
        .reflections = value;
    let first = hold == Hold::Down;
    let mode = if first {
        ViewMode::First
    } else {
        ViewMode::Third
    };
    view(world, mode, ViewMode::First);
    let (spot, pitch) = if first {
        (shore.wade, -1.15)
    } else {
        (shore.land, -0.1)
    };
    let feet = ground(world, spot, 0.0);
    stand(world, feet, feet + shore.out * 100.0, pitch);
    world.resource_mut::<abstractions::view::Enabled>().0 =
        matches!(hold, Hold::Behind | Hold::Down);
}

/// Holds the camera of a held shot this frame, and keeps the ship still over the water.
fn keep(world: &mut World, shore: &Shore, hold: Hold) {
    let over = shore.sea + shore.out * 25.0;
    hover(world, over.normalize(), SHIP_HEIGHT, (-shore.out, 0.0));
    let ship = above(world, over, SHIP_HEIGHT * 0.5);
    match hold {
        Hold::Ship => {
            let eye = above(world, shore.land + shore.out * 3.0, 0.5);
            hold_camera(world, eye, ship);
        }
        Hold::Robot => {
            let robot = above(world, shore.land, 1.2);
            let eye = above(world, shore.sea + shore.out * 12.0, 0.6);
            hold_camera(world, eye, robot);
        }
        Hold::Behind | Hold::Down => {}
    }
}

/// Logs whether the mirror camera draws, for the shot `name`.
fn report(world: &mut World, name: &str) {
    let mut cameras = world.query::<(&Name, &Camera)>();
    let drawn = cameras
        .iter(world)
        .find(|(n, _)| n.as_str() == "WaterMirror")
        .map(|(_, c)| c.is_active);
    eprintln!("PROBE reflect {name}: mirror camera active {drawn:?}");
}

/// The water mirror at a lake shore, with the reflections on and off.
pub fn reflect_shore(world: &mut World, _: &mut Notes, frame: u32) -> Act {
    if frame == 1 {
        let Some(shore) = find(world) else {
            return Act::Fail("no shore near the base".into());
        };
        world.insert_resource(shore);
        noon(world);
        set_up(world, &shore, Reflections::Actors, Hold::Behind);
        return Act::Wait;
    }
    let Some(shore) = world.get_resource::<Shore>().copied() else {
        return Act::Fail("the shore was lost".into());
    };
    let Some(step) = frame.checked_sub(SETTLE).map(|f| (f / EACH) as usize) else {
        keep(world, &shore, Hold::Behind);
        return Act::Wait;
    };
    let Some(&(name, value, hold)) = SHOTS.get(step) else {
        world.resource_mut::<abstractions::view::Enabled>().0 = true;
        world
            .resource_mut::<Persistent<Settings>>()
            .graphics
            .lighting
            .reflections = Reflections::Actors;
        view(world, ViewMode::Third, ViewMode::First);
        return Act::Next;
    };
    let at = (frame - SETTLE) % EACH;
    if at == 0 {
        set_up(world, &shore, value, hold);
    }
    keep(world, &shore, hold);
    if at == EACH - 1 {
        report(world, name);
        shot(world, name);
    }
    Act::Wait
}
