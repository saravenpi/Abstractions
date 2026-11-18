//! Keeping the sea under the focus: its seat on the planet, its floor map and its shader values.

use bevy::math::DVec3;
use bevy::prelude::*;

use super::floor::SeaFloor;
use super::liquid::Liquid;
use super::material::WaterMaterial;
use super::seat::Seat;
use super::uniform::WaterUniform;
use super::{Sea, SeaInputs, depth, mesh};
use crate::wind::Wind;
use crate::world::focus::Focus;
use crate::world::{Planet, Sited};

/// Distance from the camera in meters at which the near sea has faded out completely, leaving
/// the flat sea of the coarse planet mesh. It is also the altitude above which the near sea is
/// not drawn at all.
///
/// The fade starts at 83 percent of it (`HAND_SOFT` in `water.wesl`), just past the farthest
/// detail chunk of any planet at the longest render distance: detail chunks draw the bare sea
/// bed, so the near sea must cover every one of them, while coarse chunks bring their own sea.
/// It ends just inside the rim of the sea mesh.
pub const HANDOVER: f32 = 1550.0;

/// What the follow system reads from the sea entity.
type SeaItem = (
    &'static Transform,
    &'static MeshMaterial3d<WaterMaterial>,
    &'static SeaFloor,
    &'static Seat,
    &'static mut Visibility,
);

/// Returns the wind as the sea sees it: its direction in the chart of a seat whose axes are
/// turned by `pose` in render space.
pub fn wind_in_seat(wind: &Wind, pose: Quat) -> Wind {
    let blown = pose.inverse() * Vec3::new(wind.direction.x, 0.0, wind.direction.y);
    Wind {
        direction: Vec2::new(blown.x, blown.z).normalize_or(wind.direction),
        ..*wind
    }
}

/// Returns `water` with the values that place the sea: the floor map center `map` and the mesh
/// center under sea coordinates `at`, and the planet of `radius` meters and the seat axes read
/// from the render pose of the seat.
pub fn placing(
    water: WaterUniform,
    pose: &Transform,
    radius: f32,
    map: Vec2,
    at: Vec2,
) -> WaterUniform {
    let centre = mesh::snap(Vec3::new(at.x, 0.0, at.y));
    WaterUniform {
        map: map.extend(centre.x).extend(centre.y),
        globe: (pose.translation - pose.rotation * Vec3::Y * radius).extend(radius),
        frame: (pose.rotation * Vec3::X).extend(HANDOVER),
        ..water
    }
}

/// Keeps the seat of the sea on the streamed planet near the focus, and the floor map centered
/// on the focus with the texels that scrolled in sampled.
///
/// A new seat restarts the map. An anchor rebase or a change of cube face changes nothing here,
/// and a planet without ground leaves the map as it is.
pub fn refresh(
    focus: Focus,
    planet: Option<Res<Planet>>,
    mut images: ResMut<Assets<Image>>,
    mut seas: Query<(&mut SeaFloor, &mut Seat, &mut Sited)>,
) {
    let planet = planet.filter(|planet| planet.globe.has_ground());
    let (Some(planet), Some(at)) = (planet, focus.planet()) else {
        return;
    };
    for (mut floor, mut seat, mut sited) in &mut seas {
        if !seat.holds(&planet.globe) || seat.strayed(seat.coords(at)) {
            *seat = Seat::at(&planet.globe, at);
            *sited = seat.sited();
            floor.restart();
        }
        let p = seat.coords(at);
        let centre = depth::centre_texel(Vec3::new(p.x, 0.0, p.y));
        floor.follow(centre, &*seat, &mut images);
    }
}

/// Refreshes the shader values of the sea with the liquid of the planet it sits on, and hides it once the
/// focus is above the hand-over altitude, where the coarse planet mesh draws the sea, and on a planet
/// without a sea, which a planet without ground never has.
pub fn follow_focus(
    focus: Focus,
    inputs: SeaInputs,
    planet: Option<Res<Planet>>,
    mut sea: Query<SeaItem, With<Sea>>,
    mut materials: ResMut<Assets<WaterMaterial>>,
) {
    let at = focus
        .planet()
        .filter(|_| planet.as_ref().is_none_or(|p| p.globe.has_sea()));
    let liquid = planet.map_or(Liquid::WATER, |planet| Liquid::of(&planet.globe));
    for (pose, handle, floor, seat, mut visibility) in &mut sea {
        let radius = seat.radius();
        let low = |at: &DVec3| at.length() < f64::from(radius + HANDOVER);
        let near = at.filter(|at| liquid.shown && low(at));
        visibility.set_if_neq(match near {
            Some(_) => Visibility::Inherited,
            None => Visibility::Hidden,
        });
        let Some(at) = near else {
            continue;
        };
        if let Some(mut material) = materials.get_mut(&handle.0) {
            let map = floor.origin().truncate().truncate();
            let water = inputs.uniform(pose.rotation).of(&liquid);
            material.water = placing(water, pose, radius, map, seat.coords(at));
        }
    }
}
