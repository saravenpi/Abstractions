//! The stars in space, with the interface hidden: the chase view with the star behind the camera
//! and to the side, the cockpit looking away from the star, low orbit over the day side and over
//! the night side, then the chase view with the star in the middle of the frame and at its edge.
//! Every shot must show the star field, and around the star a smooth glow that fades into it with
//! no dark ring. Selected with `PROBE_STAGES=stars`.

use abstractions::screens::HideHud;

use super::*;

/// Frames each view of [`space_stars`] holds: the eye adapts, the world streams in, then the
/// shot is taken two frames before the next view moves the ship.
const HOLD: u32 = 150;

/// The frame of a view at which the eye has adapted and the game runs at walking pace again.
const ADAPTED: u32 = 62;

/// Simulated seconds per frame while the eye adapts to a view.
const ADAPT: f64 = 1.0;

/// The views: the name of the shot and whether the camera is outside the ship.
const VIEWS: [(&str, bool); 6] = [
    ("S1-chase-space", true),
    ("S2-cockpit-space", false),
    ("S3-orbit-day", true),
    ("S4-orbit-night", true),
    ("S5-star-centre", true),
    ("S6-star-edge", true),
];

/// Returns where the view `index` puts the ship for the star along the unit planet direction
/// `light`: the planet direction under it, its height over the sea and where its nose points,
/// as `hover` takes them. The views in space stand two radii up beside the planet, the orbits
/// at 1.6 times the height of the air, the star 53 degrees up on the day side, 53 degrees down
/// on the night side. The last two look at the star, then 40 degrees to its side.
fn pose(world: &World, index: usize, light: DVec3) -> (DVec3, f64, DVec3) {
    let radius = world.resource::<Planet>().globe.radius();
    let body = world.resource::<Location>().body;
    let air = world.resource::<Universe>().planet(body).atmosphere_height;
    let across = light.any_orthonormal_vector();
    let side = across.cross(light);
    match index {
        0 => (across, radius * 2.0, -light * 0.8 + side * 0.6),
        1 => (across, radius * 2.0, -light),
        2 => ((light * 0.8 + across * 0.6).normalize(), air * 1.6, -light),
        3 => ((-light * 0.8 + across * 0.6).normalize(), air * 1.6, side),
        4 => (across, radius * 2.0, light),
        _ => (across, radius * 2.0, light * 0.77 + side * 0.64),
    }
}

/// Seats the pilot, hides the interface and holds the ship in each of the [`VIEWS`] in turn,
/// the engines off, shooting each once the eye has adapted. Gives the interface and the first
/// person view back at the end.
pub fn space_stars(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    let Some(t) = card_ready(world, notes, frame, true) else {
        return card_wait(frame);
    };
    let (index, at) = (((t - 1) / HOLD) as usize, (t - 1) % HOLD + 1);
    let Some(&(name, outside)) = VIEWS.get(index) else {
        world.resource_mut::<HideHud>().0 = false;
        view(world, ViewMode::First, ViewMode::First);
        return Act::Next;
    };
    match at {
        1 => {
            lens(world, 60);
            view(
                world,
                ViewMode::First,
                [ViewMode::First, ViewMode::Third][usize::from(outside)],
            );
            pace(world, ADAPT);
        }
        ADAPTED => pace(world, WALK),
        n if n == HOLD - 2 => shot(world, name),
        _ => {}
    }
    let (dir, height, toward) = pose(world, index, star(world));
    hover(world, dir, height, (toward, 0.0));
    with_ship(world, |ship| ship.throttle = 0.0);
    Act::Wait
}
