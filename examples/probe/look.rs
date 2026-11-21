//! Stages that judge the sky and the light: the same view at several hours of the day.

use abstractions::render::{CloudCover, CloudShell, Palette, SkyDome, Starfield, Vantage};
use abstractions::screens::HideHud;
use abstractions::universe::sun;

use super::*;

/// Simulated seconds per frame while the eye adapts to a new hour.
const ADAPT: f64 = 1.0;

/// Frames the eye is given to adapt after the clock moved.
const SETTLE: u32 = 90;

/// Returns the sine of the height of the star over the robot at `time` seconds.
fn star_height(world: &World, here: DVec3, time: f64) -> f64 {
    let body = world.resource::<Planet>().body;
    let spec = world.resource::<Universe>().planet(body);
    here.dot(sun(&spec, time))
}

/// Moves the clock to the next moment in the day the star stands at the sine `height` over the
/// robot, rising or setting, and logs it.
pub fn clock_at(world: &mut World, height: f64, rising: bool) {
    let body = world.resource::<Planet>().body;
    let day = world.resource::<Universe>().planet(body).day_seconds;
    let here = robot(world).normalize();
    let now = world.resource::<abstractions::space::Clock>().seconds;
    let step = day / 720.0;
    let (low, high) = (0..720)
        .map(|k| star_height(world, here, now + step * f64::from(k)))
        .fold((f64::MAX, f64::MIN), |(low, high), h| {
            (low.min(h), high.max(h))
        });
    eprintln!("PROBE day {day:.0} s, star height over a day {low:.2} to {high:.2}");
    let found = (0..720)
        .map(|k| now + step * f64::from(k))
        .find(|t| {
            let (a, b) = (
                star_height(world, here, *t),
                star_height(world, here, t + step),
            );
            (a - height) * (b - height) <= 0.0 && (b > a) == rising
        })
        .unwrap_or(now);
    eprintln!("PROBE clock moved {:.0} s to height {height}", found - now);
    world.resource_mut::<abstractions::space::Clock>().seconds = found;
}

/// Turns the robot to look at the horizon, `pitch` radians above it, toward the star at its
/// azimuth when `sunward`, away from it otherwise.
pub fn gaze(world: &mut World, pitch: f32, sunward: bool) {
    let here = robot(world);
    let feet = ground(world, here, 0.0);
    let up = feet.normalize();
    let east = up.cross(DVec3::Y).normalize_or(DVec3::X);
    let side = if sunward { east } else { -east };
    stand(world, feet, feet + side * 100.0, pitch);
}

/// A stage that sets the clock to `height` and looks at the sky: wait for the eye, then shoot.
fn hour(
    world: &mut World,
    frame: u32,
    (height, rising, pitch, name): (f64, bool, f32, &str),
) -> Act {
    match frame {
        1 => {
            clock_at(world, height, rising);
            gaze(world, pitch, true);
            pace(world, ADAPT);
            Act::Wait
        }
        n if n == SETTLE => {
            pace(world, WALK);
            Act::Wait
        }
        n if n == SETTLE + 4 => Act::Shoot(name.into()),
        _ => Act::Wait,
    }
}

/// Noon, the horizon ahead.
pub fn look_noon(world: &mut World, _: &mut Notes, frame: u32) -> Act {
    hour(world, frame, (0.8, true, 0.15, "L2-noon"))
}

/// The star low in the afternoon, the horizon ahead.
pub fn look_sunset(world: &mut World, _: &mut Notes, frame: u32) -> Act {
    hour(world, frame, (0.05, false, 0.1, "L3-sunset"))
}

/// Dusk, the star just under the horizon.
pub fn look_dusk(world: &mut World, _: &mut Notes, frame: u32) -> Act {
    hour(world, frame, (-0.12, false, 0.1, "L4-dusk"))
}

/// Deep night, looking at the horizon.
pub fn look_night(world: &mut World, _: &mut Notes, frame: u32) -> Act {
    hour(world, frame, (-0.8, false, 0.1, "L5-night"))
}

/// Deep night, looking high at the sky for the band of the galaxy and the stars.
pub fn look_stars(world: &mut World, _: &mut Notes, frame: u32) -> Act {
    hour(world, frame, (-0.8, false, 1.0, "L6-night-sky"))
}

/// Frames each switch of [`look_grain`] gets: the switch is thrown, the picture settles, a shot is
/// taken and the switch is put back.
const TURN: u32 = 20;

/// Hides or shows every entity with a visibility that is not the sky dome, the camera or a light.
fn bare(world: &mut World, hidden: bool) {
    let mut q = world.query_filtered::<&mut Visibility, (Without<SkyDome>, Without<Camera>)>();
    q.iter_mut(world)
        .for_each(|mut v| *v = [Visibility::Inherited, Visibility::Hidden][usize::from(hidden)]);
}

/// Throws switch `which` off (`off`) or back on, and says what it is.
fn flip(world: &mut World, which: usize, off: bool) {
    match which {
        0 => {
            let levels =
                [abstractions::render::Pixelate::default().levels, 4096.0][usize::from(off)];
            let mut q = world.query::<&mut abstractions::render::Pixelate>();
            q.iter_mut(world).for_each(|mut p| p.levels = levels);
        }
        1 => {
            let mut settings = world.resource_mut::<Persistent<Settings>>();
            settings.graphics.lighting.white_balance = !off;
        }
        2 => {
            let mut q = world.query::<(&Starfield, &mut Visibility)>();
            q.iter_mut(world).for_each(|(_, mut v)| {
                *v = [Visibility::Inherited, Visibility::Hidden][usize::from(off)]
            });
        }
        3 => {
            let mut q = world.query_filtered::<Entity, With<CloudShell>>();
            let shells: Vec<Entity> = q.iter(world).collect();
            shells.into_iter().for_each(|e| {
                if off {
                    world.despawn(e);
                }
            });
        }
        _ => bare(world, off),
    }
}

/// The switches of [`look_grain`], in the order they are thrown: the picture is shot with each one
/// off alone, then with the clouds gone for good, then with everything but the dome hidden.
const SWITCHES: [&str; 5] = [
    "L1-no-dither",
    "L1-no-balance",
    "L1-no-stars",
    "L1-no-clouds",
    "L1-dome-only",
];

/// Noon at the shot's own hour: the baseline, then each suspect of the sky grain off alone, one
/// shot each. The pixelate dither, the white balance, the starfield, the cloud shell (which the
/// layer shows or hides itself every frame, so it is despawned) and last everything but the dome.
/// The cloud shell and the dome-only shots are not put back, so they come last.
pub fn look_grain(world: &mut World, _: &mut Notes, frame: u32) -> Act {
    if frame == 1 {
        gaze(world, 0.3, true);
        let palette = world.resource::<Palette>();
        eprintln!(
            "PROBE sky stars {:.2} zenith {:?} horizon {:?} air {:.2}",
            palette.stars,
            palette.zenith,
            palette.horizon,
            world.resource::<Vantage>().air
        );
    }
    if frame == 30 {
        shot(world, "L1a-all");
        return Act::Wait;
    }
    let step = frame.saturating_sub(31) / (2 * TURN);
    let at = frame.saturating_sub(31) % (2 * TURN);
    if frame < 31 || step as usize >= SWITCHES.len() {
        return Act::Wait;
    }
    let last = step as usize == SWITCHES.len() - 1;
    match at {
        0 => flip(world, step as usize, true),
        n if n == TURN - 1 => shot(world, SWITCHES[step as usize]),
        n if n == TURN => {
            if step < 3 {
                flip(world, step as usize, false)
            }
        }
        n if n == 2 * TURN - 1 && last => {
            return Act::Shoot(format!("{}-end", SWITCHES[step as usize]));
        }
        _ => {}
    }
    Act::Wait
}

/// Forces the cover the cloud shell is drawn with, or gives it back to the weather.
fn cover(world: &mut World, value: Option<f32>) {
    world.resource_mut::<CloudCover>().0 = value;
}

/// A clear, a broken and an overcast sky by day from the ground, then a clear night: the cover
/// is forced, since the weather of the seed cannot be asked for. Shots `L7-clear-day`,
/// `L7-broken`, `L7-overcast` and `L7-clear-night`.
pub fn look_cover(world: &mut World, _: &mut Notes, frame: u32) -> Act {
    match frame {
        1 => {
            gaze(world, 0.25, true);
            cover(world, Some(0.0));
        }
        30 => shot(world, "L7-clear-day"),
        31 => cover(world, Some(0.45)),
        60 => shot(world, "L7-broken"),
        61 => cover(world, Some(0.9)),
        90 => shot(world, "L7-overcast"),
        91 => {
            cover(world, Some(0.0));
            clock_at(world, -0.8, false);
            pace(world, ADAPT);
        }
        n if n == 91 + SETTLE => pace(world, WALK),
        n if n == 95 + SETTLE => {
            cover(world, None);
            return Act::Shoot("L7-clear-night".into());
        }
        _ => {}
    }
    Act::Wait
}

/// Frames each shot of a vista gets: the clock and the place are set, the eye adapts, then it shoots.
const VISTA: u32 = 320;

/// One shot of a vista: its name, the height of the star to move the clock to (a sine, rising or
/// not) when the clock is to move, the cloud cover to force (`None` is no shell at all), and the
/// metres over the ground the view is taken from, 0 for the robot on foot.
struct View {
    /// Name of the shot.
    name: &'static str,
    /// Star height to move the clock to.
    star: Option<(f64, bool)>,
    /// Forced cover, or no clouds.
    cover: Option<f32>,
    /// Metres over the ground.
    high: f64,
    /// Radians the view is turned from the star, about the vertical: 0 looks at the star.
    turn: f64,
}

/// Runs the views one after the other, `VISTA` frames each, and ends on the last shot.
fn vista(world: &mut World, frame: u32, views: &[View]) -> Act {
    let index = (frame.saturating_sub(1) / VISTA) as usize;
    let at = frame.saturating_sub(1) % VISTA;
    let Some(view) = views.get(index) else {
        return Act::Wait;
    };
    match at {
        0 => {
            world.resource_mut::<HideHud>().0 = true;
            let feet = match view.star {
                Some((height, rising)) => site(world, height, rising),
                None => player_at(world),
            };
            world.resource_mut::<Persistent<Settings>>().graphics.clouds = view.cover.is_some();
            cover(world, view.cover);
            let up = feet.normalize();
            let toward = DQuat::from_axis_angle(up, view.turn) * sunward(world, feet);
            let height = up.dot(star(world));
            eprintln!(
                "PROBE vista {}: at {:?} km, star height {height:.2}",
                view.name,
                (feet / 1000.0).round()
            );
            match view.high > 0.0 {
                true => {
                    let radius = world.resource::<Planet>().globe.radius();
                    hover(
                        world,
                        up,
                        feet.length() - radius + view.high,
                        (toward, 0.12),
                    );
                }
                false => stand(world, feet, feet + toward * 100.0, 0.08),
            }
            pace(world, ADAPT);
        }
        100 => pace(world, WALK),
        n if n == VISTA - 5 => {
            return match index + 1 == views.len() {
                true => Act::Shoot(view.name.into()),
                false => {
                    shot(world, view.name);
                    Act::Wait
                }
            };
        }
        _ => {}
    }
    Act::Wait
}

/// Radians a side-lit view is turned from the star: the star stands to the side and a little behind
/// the camera, so slopes show a lit and a shaded face and shadows run across the picture.
const SIDE: f64 = 2.0;

/// Returns the five views of a day from the ground under a broken sky, turned `turn` radians from
/// the star: noon, the star 8 degrees up, on the horizon, twilight and night. Their names start
/// with `prefix`.
fn day(prefix: &str, turn: f64) -> [View; 5] {
    let names: &'static [[&'static str; 5]; 2] = &[
        [
            "V1-noon",
            "V1-low-star",
            "V1-horizon",
            "V1-twilight",
            "V1-night",
        ],
        [
            "V3-noon",
            "V3-low-star",
            "V3-horizon",
            "V3-twilight",
            "V3-night",
        ],
    ];
    let names = names[usize::from(prefix == "V3")];
    let stars = [
        (0.8, true),
        (0.14, false),
        (0.0, false),
        (-0.12, false),
        (-0.8, false),
    ];
    std::array::from_fn(|k| View {
        name: names[k],
        star: Some(stars[k]),
        cover: Some(0.3),
        high: 0.0,
        turn,
    })
}

/// The same five hours as [`look_vista_light`] with the star to the side and behind: the lit
/// landscape, where the key, the fill and the long shadows show. Shots `V3-noon` to `V3-night`.
pub fn look_vista_side(world: &mut World, _: &mut Notes, frame: u32) -> Act {
    vista(world, frame, &day("V3", SIDE))
}

/// The light of a day from the ground, with broken cloud: noon, a low star near 8 degrees, the
/// star on the horizon, twilight and night, looking at the star. On a turning home the clock moves to
/// each height. On a locked home the view goes where the star stands at that height, toward the
/// terminator and past it. Shots `V1-noon` to `V1-night`.
pub fn look_vista_light(world: &mut World, _: &mut Notes, frame: u32) -> Act {
    vista(world, frame, &day("V1", 0.0))
}

/// The same place from the ground and from 300 m, 3 km and 30 km up, at the hour it is, each
/// without clouds and with a broken sky. Shots `V2-<height>-clear` and `V2-<height>-broken`.
pub fn look_vista_height(world: &mut World, _: &mut Notes, frame: u32) -> Act {
    let view = |name, cover, high| View {
        name,
        star: None,
        cover,
        high,
        turn: SIDE,
    };
    let views = [
        view("V2-ground-clear", Some(0.0), 0.0),
        view("V2-ground-broken", Some(0.45), 0.0),
        view("V2-300m-clear", Some(0.0), 300.0),
        view("V2-300m-broken", Some(0.45), 300.0),
        view("V2-3km-clear", Some(0.0), 3.0e3),
        view("V2-3km-broken", Some(0.45), 3.0e3),
        view("V2-30km-clear", Some(0.0), 3.0e4),
        view("V2-30km-broken", Some(0.45), 3.0e4),
    ];
    vista(world, frame, &views)
}
