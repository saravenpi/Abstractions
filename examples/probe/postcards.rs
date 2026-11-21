//! Postcards: clean pictures of the game for the website and the title screen, each composed on
//! purpose. This file holds what every card shares: the camera on its tripod, the switch between
//! foot and seat, the parked ship, the light and the framing.
//!
//! Selected with `PROBE_STAGES=postcards`; `SMOKE_ONLY` keeps a subset, each card sets itself up
//! from any state. The interface is hidden with the game's own switch, `HideHud`.
//! `POSTCARD_SETTLE` is the frames a card waits for the chunks, the grass and the far ground,
//! default 150. `PROBE_SIZE=1920x1080` sets the window size. `SMOKE_SEED` picks the world, 18
//! by default for this group: a home that turns under a Sun-like star, so the star can be low.
//!
//! Most cards are shot in golden light, the star 8 degrees up ([`GOLD`]), from the side or
//! three quarters from behind. On a planet locked to its star the clock cannot bring that light
//! to the base: the cards at the base keep the light they have, and the landscapes go where the
//! star stands at the wanted height, toward the terminator (see [`site`]).

use abstractions::companion::Companion;
use abstractions::screens::HideHud;
use abstractions::ship::Warp;
use abstractions::space::{Clock, Placed};
use abstractions::universe::sun;

use super::*;

/// Sine of the height of the star in golden light: 8 degrees up.
pub const GOLD: f64 = 0.14;

/// Sine of the height of the star when it sits on the horizon.
pub const RIM: f64 = 0.02;

/// Sine of the height of the star in twilight: 5 degrees under the horizon.
pub const DUSK: f64 = -0.08;

/// Sine of the height of the star at noon, or as high as the day brings it.
pub const NOON: f64 = 0.85;

/// Sine of the height of the star in the night.
pub const NIGHT: f64 = -0.5;

/// Simulated seconds per frame while the eye adapts to the hour of a card.
const ADAPT: f64 = 1.0;

/// The frame of a card at which the eye has adapted and the game runs at walking pace again.
const ADAPTED: u32 = 62;

/// Frames a card may take to change between foot and seat before it gives up.
const SWITCH: u32 = 600;

/// The camera of a postcard, put in place after the transforms are propagated.
#[derive(Resource, Default, Clone, Copy)]
pub enum Tripod {
    /// The game camera is left alone.
    #[default]
    Free,
    /// Standing at a planet position and aimed at another.
    Fixed(DVec3, DVec3),
    /// Following the robot, or the ship, without rolling with it: `eye` and `aim` are metres to
    /// the right, up and behind of it, in level axes that look along the planet direction `heading`.
    Follow {
        robot: bool,
        heading: DVec3,
        eye: Vec3,
        aim: Vec3,
    },
}

/// Puts the game camera where the [`Tripod`] says.
pub fn tripod(
    tripod: Res<Tripod>,
    anchor: Option<Res<Anchor>>,
    ships: Query<&GlobalTransform, (With<Ship>, Without<abstractions::render::GameCamera>)>,
    robots: Query<&GlobalTransform, (With<Player>, Without<abstractions::render::GameCamera>)>,
    mut cameras: Query<&mut GlobalTransform, With<abstractions::render::GameCamera>>,
) {
    let Some(anchor) = anchor else {
        return;
    };
    let (eye, target) = match *tripod {
        Tripod::Free => return,
        Tripod::Fixed(eye, target) => (anchor.to_render(eye), anchor.to_render(target)),
        Tripod::Follow {
            robot,
            heading,
            eye,
            aim,
        } => {
            let subject = match robot {
                true => robots.iter().next(),
                false => ships.iter().next(),
            };
            let Some(at) = subject.map(|pose| pose.translation()) else {
                return;
            };
            let up = abstractions::space::up(&anchor, at);
            let ahead = (anchor.rotation * heading).as_vec3();
            let ahead = (ahead - up * ahead.dot(up)).normalize_or(up.any_orthonormal_vector());
            let right = ahead.cross(up);
            let place = |v: Vec3| at + right * v.x + up * v.y - ahead * v.z;
            (place(eye), place(aim))
        }
    };
    let up = abstractions::space::up(&anchor, eye);
    let pose = Transform::from_translation(eye).looking_at(target, up);
    for mut camera in &mut cameras {
        *camera = GlobalTransform::from(pose);
    }
}

/// Returns the frames a card waits before its shot, from `POSTCARD_SETTLE`, at least 90.
pub fn settle() -> u32 {
    let asked = std::env::var("POSTCARD_SETTLE").ok();
    asked.and_then(|s| s.parse().ok()).unwrap_or(150).max(90)
}

/// Sets the vertical field of view of the game camera, in degrees.
pub fn lens(world: &mut World, degrees: u32) {
    if world.resource::<Persistent<Settings>>().graphics.fov != degrees {
        world.resource_mut::<Persistent<Settings>>().graphics.fov = degrees;
    }
}

/// Puts the ship back on the landing pad at rest, as a new game parks it. The pose is read from
/// the ship the first time, while it still stands where the game parked it.
pub fn park(world: &mut World, notes: &mut Notes) {
    let (Some(base), Some(hull)) = (world.get_resource::<Base>().copied(), ship(world)) else {
        return;
    };
    let parked = hull.landed && hull.pos.distance(base.pad) < 6.0;
    let fallback = (base.pad + base.pad.normalize() * 1.1, base.pad_rot());
    let (pos, rot) = *notes.berth.get_or_insert(if parked {
        (hull.pos, hull.rot)
    } else {
        fallback
    });
    with_ship(world, |ship| {
        ship.frame = Frame::Surface(base.body);
        (ship.pos, ship.rot) = (pos, rot);
        (ship.vel, ship.spin, ship.throttle) = (DVec3::ZERO, DVec3::ZERO, 0.0);
        (ship.landed, ship.touching, ship.warp) = (true, true, Warp::Off);
        (ship.gear, ship.stowed) = (1.0, false);
    });
    let mut placed = world.query_filtered::<&mut Placed, With<Ship>>();
    for mut pose in placed.iter_mut(world) {
        *pose = Placed { pos, rot };
    }
}

/// Starts a card and brings the robot to its feet or to the seat of the parked ship, with the
/// game's own board key. Returns the frames since the robot is where the card wants it, from 1,
/// or nothing while it changes place. On the first frame the interface is hidden, the camera is
/// freed and the ship is parked.
pub fn card_ready(world: &mut World, notes: &mut Notes, frame: u32, seated: bool) -> Option<u32> {
    if frame == 1 {
        notes.ready = None;
        world.resource_mut::<HideHud>().0 = true;
        *world.resource_mut::<Tripod>() = Tripod::Free;
        world.resource_mut::<Eye>().0 = None;
        pace(world, WALK);
        park(world, notes);
    }
    if let Some(at) = notes.ready {
        return Some(frame - at);
    }
    if (world.resource::<Location>().travel == Travel::InShip) == seated {
        notes.ready = Some(frame - 1);
        return Some(1);
    }
    let board = bound(world, |s| s.controls.keys.ship.board);
    match (frame % 90, ship(world)) {
        (2, Some(hull)) if seated => {
            let beside = hull.pos + hull.rot * DVec3::new(-1.7, -1.2, -1.5);
            stand(world, beside, hull.pos, 0.0);
        }
        (30, _) => key(world, board, true),
        (31, _) => key(world, board, false),
        _ => {}
    }
    None
}

/// What a card answers while the robot changes place: wait, or fail after [`SWITCH`] frames.
pub fn card_wait(frame: u32) -> Act {
    match frame > SWITCH {
        true => Act::Fail("the robot never changed between foot and seat".into()),
        false => Act::Wait,
    }
}

/// The clock of a card that stands still, at card frame `t`: the eye adapts for a minute of the
/// game, the world streams in, and the shot `name` is taken after [`settle`] frames.
pub fn still(world: &mut World, t: u32, name: &str) -> Act {
    match t {
        2 => pace(world, ADAPT),
        ADAPTED => pace(world, WALK),
        _ => {}
    }
    match t >= settle() {
        true => Act::Shoot(name.into()),
        false => Act::Wait,
    }
}

/// Returns the unit direction to the star now, in planet axes.
pub fn star(world: &World) -> DVec3 {
    let body = world.resource::<Planet>().body;
    let spec = world.resource::<Universe>().planet(body);
    sun(&spec, world.resource::<Clock>().seconds)
}

/// Returns the horizontal unit direction toward the star at planet position `at`.
pub fn sunward(world: &World, at: DVec3) -> DVec3 {
    let up = at.normalize();
    star(world)
        .reject_from(up)
        .normalize_or(up.any_orthonormal_vector())
}

/// Moves the clock to a low star, at the sine `height` over the robot, in the morning or in the
/// evening: whichever lights a camera that looks along `ahead` more from the side, for long
/// shadows across the picture. On a planet locked to its star the clock stays.
pub fn side_light(world: &mut World, height: f64, ahead: DVec3) {
    let start = world.resource::<Clock>().seconds;
    let mut best = (f64::MAX, start);
    for rising in [true, false] {
        world.resource_mut::<Clock>().seconds = start;
        clock_at(world, height, rising);
        let here = robot(world);
        let flat = ahead.reject_from(here.normalize()).normalize_or_zero();
        let along = sunward(world, here).dot(flat).abs();
        if along < best.0 {
            best = (along, world.resource::<Clock>().seconds);
        }
    }
    world.resource_mut::<Clock>().seconds = best.1;
}

/// Where a photographer stands around a subject, and how the picture is framed.
pub struct Stand {
    /// Radians the photographer walks round the subject from the reference direction,
    /// anticlockwise seen from above.
    pub turn: f64,
    /// Metres from the subject.
    pub back: f64,
    /// Metres of the lens above the subject. The lens never goes under 0.35 m over the ground.
    pub rise: f64,
    /// Where the subject sits across the picture: -1 on the left third line, 1 on the right
    /// one, 0 in the middle.
    pub third: f64,
    /// How much sky: 1 puts the subject on the lower third line, -1 on the upper one.
    pub sky: f64,
}

/// Returns the eye and the target of a camera that stands as `stand` says around the planet
/// position `subject`, the reference direction being `from`. The thirds follow the field of view.
pub fn framed(world: &World, subject: DVec3, from: DVec3, stand: Stand) -> (DVec3, DVec3) {
    let up = subject.normalize();
    let from = from
        .reject_from(up)
        .normalize_or(up.any_orthonormal_vector());
    let out = DQuat::from_axis_angle(up, stand.turn) * from;
    let wanted = subject + out * stand.back + up * stand.rise;
    let floor = ground(world, wanted, 0.35);
    let eye = if wanted.length() < floor.length() {
        floor
    } else {
        wanted
    };
    let fov = world.resource::<Persistent<Settings>>().graphics.fov;
    let half = (f64::from(fov) * 0.5).to_radians().tan();
    let far = eye.distance(subject);
    let right = (subject - eye).cross(up).normalize_or_zero();
    let across = right * (-stand.third * far * half * 16.0 / 9.0 / 3.0);
    (eye, subject + across + up * (stand.sky * far * half / 3.0))
}

/// Returns the planet position of the companion, if it is drawn.
pub fn buddy(world: &mut World) -> Option<DVec3> {
    let anchor = *world.resource::<Anchor>();
    let mut found = world.query_filtered::<&GlobalTransform, With<Companion>>();
    let at = found.iter(world).next()?.translation();
    Some(anchor.to_frame(at))
}

/// Returns the ground `metres` from planet position `at`, on the side along `way` or on the
/// other one: the side whose ground is drier and nearer the height of the ground under `at`.
pub fn beside(world: &World, at: DVec3, way: DVec3, metres: f64) -> DVec3 {
    let level = ground(world, at, 0.0).length();
    let globe = &world.resource::<Planet>().globe;
    let score = |p: &DVec3| {
        let wet = if globe.height(p.normalize()) <= 0.5 {
            100.0
        } else {
            0.0
        };
        (p.length() - level).abs() + wet
    };
    let sides = [way, -way].map(|side| ground(world, at + side * metres, 0.0));
    match score(&sides[0]) <= score(&sides[1]) {
        true => sides[0],
        false => sides[1],
    }
}

/// Applies `PROBE_SIZE`, such as `1920x1080`, to the window: the size of the pictures.
pub fn size_window(mut windows: Query<&mut Window, With<PrimaryWindow>>) {
    let asked = std::env::var("PROBE_SIZE").unwrap_or_default();
    let Some((wide, high)) = asked.split_once('x') else {
        return;
    };
    let (Ok(wide), Ok(high)) = (wide.parse::<f32>(), high.parse::<f32>()) else {
        return;
    };
    for mut window in &mut windows {
        window.resolution.set(wide, high);
    }
}

/// Returns where a card that wants the star at the sine `height` is shot, as a planet position
/// on the ground. Where the day brings the star to that height over the base, the clock moves
/// to it, rising or setting, and the place is the pad. On a planet locked to its star it is the
/// dry ground nearest the base where the star stands at that height: toward the terminator.
pub fn site(world: &mut World, height: f64, rising: bool) -> DVec3 {
    let pad = world.get_resource::<Base>().map_or(DVec3::Y, |b| b.pad);
    let body = world.resource::<Planet>().body;
    let spec = world.resource::<Universe>().planet(body);
    let now = world.resource::<Clock>().seconds;
    let here = pad.normalize();
    let heights =
        (0..720).map(|k| here.dot(sun(&spec, now + spec.day_seconds * f64::from(k) / 720.0)));
    let (low, high) = heights.fold((f64::MAX, f64::MIN), |(l, h), s| (l.min(s), h.max(s)));
    if spec.day_seconds.is_finite() && low <= height && height <= high {
        clock_at(world, height, rising);
        return pad;
    }
    let star = star(world);
    let across = here
        .reject_from(star)
        .normalize_or(star.any_orthonormal_vector());
    let globe = &world.resource::<Planet>().globe;
    let turns = (0..24).map(|k| DQuat::from_axis_angle(star, f64::from(k) * 0.26) * across);
    let places = turns.map(|q| (star * height + q * (1.0 - height * height).sqrt()).normalize());
    let dry = places.clone().find(|p| globe.height(*p) > 3.0);
    let place = dry.or(places.clone().next()).unwrap_or(here);
    eprintln!(
        "PROBE the star never stands at {height} over the base ({low:.2} to {high:.2}): shooting elsewhere"
    );
    ground(world, place * globe.radius(), 0.0)
}
