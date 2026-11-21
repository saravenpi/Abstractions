//! Postcards of the land alone, for the hero of the website and the title screen: no character,
//! the left third calm for a title, each a different place and light. The stage looks for its
//! subject with the game's own data, then stands the camera so the low star lights it from the
//! side or three quarters from behind. The ship carries the player there and stays behind the
//! camera, out of the picture.

use abstractions::world::Surface;

use super::*;

/// One landscape: its file name, the light, the subject and where the camera stands.
struct Scene {
    /// Name of the picture.
    name: &'static str,
    /// Sine of the height of the star, and whether it rises.
    light: (f64, bool),
    /// Finds the subject within reach of the planet position it is given.
    find: fn(&World, DVec3) -> Option<DVec3>,
    /// Radians from the direction of the star to the way the camera looks: 0 looks at the
    /// star, 1 has it just out of the picture, three quarters from behind.
    off: f64,
    /// Metres from the camera to the subject, and of the lens over the ground.
    stand: (f64, f64),
    /// How much sky: 1 puts the subject on the lower third line, -1 on the upper one.
    sky: f64,
}

/// Returns the ground points on `rings` of metres round `site`, 24 to a ring.
fn round(world: &World, site: DVec3, rings: &[f64]) -> Vec<DVec3> {
    let ways = compass(site, 24);
    let spots = rings
        .iter()
        .flat_map(|r| ways.iter().map(move |d| site + *d * *r));
    spots.map(|p| ground(world, p, 0.0)).collect()
}

/// Returns the highest ground within 8 km: a ridge to look along.
fn ridge(world: &World, site: DVec3) -> Option<DVec3> {
    let rings: Vec<f64> = (1..=16).map(|k| f64::from(k) * 500.0).collect();
    let spots = round(world, site, &rings);
    spots
        .into_iter()
        .max_by(|a, b| a.length().total_cmp(&b.length()))
}

/// Returns the shore within 6 km whose land rises steepest behind it: a coast with cliffs.
fn cliffs(world: &World, site: DVec3) -> Option<DVec3> {
    let globe = &world.resource::<Planet>().globe;
    let rings: Vec<f64> = (1..=60).map(|k| f64::from(k) * 100.0).collect();
    let spots = round(world, site, &rings);
    let wet = |p: &DVec3| globe.height(p.normalize()) < -1.0;
    let shores = spots.windows(25).filter(|w| wet(&w[24]) && !wet(&w[0]));
    let steep = |w: &&[DVec3]| grade(globe, w[0].normalize());
    let shore = shores.max_by(|a, b| steep(a).total_cmp(&steep(b)))?;
    Some(shore[0])
}

/// Returns the nearest shore within 6 km, for the sea under the twilight.
fn strand(world: &World, site: DVec3) -> Option<DVec3> {
    let globe = &world.resource::<Planet>().globe;
    let rings: Vec<f64> = (1..=60).map(|k| f64::from(k) * 100.0).collect();
    let spots = round(world, site, &rings);
    let dry = |p: &&DVec3| (0.5..6.0).contains(&globe.height(p.normalize()));
    spots.iter().find(dry).copied()
}

/// Returns the nearest river within 10 km.
fn river(world: &World, site: DVec3) -> Option<DVec3> {
    let globe = &world.resource::<Planet>().globe;
    let rings: Vec<f64> = (1..=50).map(|k| f64::from(k) * 200.0).collect();
    let spots = round(world, site, &rings);
    let runs = |p: &&DVec3| globe.surface(p.normalize()) == Surface::River;
    spots.iter().find(runs).copied()
}

/// The landscapes: a ridge in golden light, a cliff coast in the evening, the sea in twilight,
/// the land from high up at noon, and a river in the morning.
const SCENES: [Scene; 5] = [
    Scene {
        name: "p16-ridge",
        light: (GOLD, true),
        find: ridge,
        off: 1.0,
        stand: (900.0, 30.0),
        sky: 0.3,
    },
    Scene {
        name: "p18-cliffs",
        light: (GOLD, false),
        find: cliffs,
        off: 0.9,
        stand: (240.0, 22.0),
        sky: -0.4,
    },
    Scene {
        name: "p20-twilight",
        light: (DUSK, false),
        find: strand,
        off: 0.35,
        stand: (160.0, 5.0),
        sky: 1.0,
    },
    Scene {
        name: "p21-noon",
        light: (NOON, true),
        find: ridge,
        off: 1.3,
        stand: (2600.0, 420.0),
        sky: -0.8,
    },
    Scene {
        name: "p22-river",
        light: (GOLD, true),
        find: river,
        off: 1.1,
        stand: (130.0, 7.0),
        sky: -0.3,
    },
];

/// Returns where the camera of `scene` stands for `subject` and what it aims at: of the two
/// sides of the star, the one whose ground is dry and higher, the subject on the right third
/// so the left one stays calm.
fn camera(world: &World, scene: &Scene, subject: DVec3) -> (DVec3, DVec3) {
    let up = subject.normalize();
    let light = sunward(world, subject);
    let globe = &world.resource::<Planet>().globe;
    let place = |turn: f64| {
        let look = DQuat::from_axis_angle(up, turn) * light;
        ground(world, subject - look * scene.stand.0, scene.stand.1)
    };
    let rank = |eye: &DVec3| {
        let wet = if globe.height(eye.normalize()) < 0.5 {
            1.0e6
        } else {
            0.0
        };
        wet - eye.length()
    };
    let (one, other) = (place(scene.off), place(-scene.off));
    let eye = if rank(&one) <= rank(&other) {
        one
    } else {
        other
    };
    let framing = Stand {
        turn: 0.0,
        back: eye.distance(subject),
        rise: eye.length() - subject.length(),
        third: 1.0,
        sky: scene.sky,
    };
    (eye, framed(world, subject, eye - subject, framing).1)
}

/// Holds the ship 150 m behind a camera at `eye` that looks at `target`, 25 m over the ground:
/// out of the picture, with its shadow behind the camera too.
fn carry(world: &mut World, eye: DVec3, target: DVec3) {
    let up = eye.normalize();
    let back = (eye - target)
        .reject_from(up)
        .normalize_or(up.any_orthonormal_vector());
    let (over, height) = above_ground(world, eye + back * 150.0, 25.0);
    hover(world, over, height, (-back, 0.0));
}

/// Shoots `scene`: finds the place and the light, stands the camera, holds the ship behind it
/// while the world streams in, and takes the picture.
fn landscape(world: &mut World, notes: &mut Notes, frame: u32, scene: &Scene) -> Act {
    let Some(t) = card_ready(world, notes, frame, true) else {
        return card_wait(frame);
    };
    if t == 1 {
        lens(world, 60);
        view(world, ViewMode::First, ViewMode::First);
        let place = site(world, scene.light.0, scene.light.1);
        let Some(subject) = (scene.find)(world, place) else {
            return Act::Fail(format!("nothing to look at for {}", scene.name));
        };
        let (eye, target) = camera(world, scene, subject);
        notes.spots = vec![eye, target];
        *world.resource_mut::<Tripod>() = Tripod::Fixed(eye, target);
    }
    carry(world, notes.spots[0], notes.spots[1]);
    still(world, t, scene.name)
}

/// The highest ridge within 8 km in the golden light of the morning.
pub fn card_ridge(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    landscape(world, notes, frame, &SCENES[0])
}

/// A coast with cliffs in the golden light of the evening.
pub fn card_cliffs(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    landscape(world, notes, frame, &SCENES[1])
}

/// The sea from the shore in twilight, looking toward the glow.
pub fn card_twilight(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    landscape(world, notes, frame, &SCENES[2])
}

/// The land from 400 m up at noon.
pub fn card_noon(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    landscape(world, notes, frame, &SCENES[3])
}

/// A river in the golden light of the morning.
pub fn card_river(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    landscape(world, notes, frame, &SCENES[4])
}

/// The valley under the highest ridge within 8 km, looked down into from the ridge with the
/// star on the horizon: the lowest dry ground within 3 km on the right third.
pub fn card_valley(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    let Some(t) = card_ready(world, notes, frame, true) else {
        return card_wait(frame);
    };
    if t == 1 {
        lens(world, 60);
        view(world, ViewMode::First, ViewMode::First);
        let place = site(world, RIM, false);
        let Some(top) = ridge(world, place) else {
            return Act::Fail("no ridge".into());
        };
        let rings: Vec<f64> = (2..=10).map(|k| f64::from(k) * 300.0).collect();
        let globe = world.resource::<Planet>().globe.clone();
        let spots = round(world, top, &rings);
        let dry = spots.iter().filter(|p| globe.height(p.normalize()) > 1.0);
        let floor = dry.min_by(|a, b| a.length().total_cmp(&b.length()));
        let floor = floor.copied().unwrap_or(place);
        let down = Stand {
            turn: 0.0,
            back: top.distance(floor),
            rise: top.length() - floor.length() + 4.0,
            third: 1.0,
            sky: 0.3,
        };
        let (eye, target) = framed(world, floor, top - floor, down);
        notes.spots = vec![eye, target];
        *world.resource_mut::<Tripod>() = Tripod::Fixed(eye, target);
    }
    carry(world, notes.spots[0], notes.spots[1]);
    still(world, t, "p17-valley")
}
