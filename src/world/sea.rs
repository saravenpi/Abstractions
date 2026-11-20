//! The sea: one wavy, foamy water mesh wrapped on the planet around the focus, laid out in a
//! chart fixed to the planet, and handed over to the coarse planet mesh from high up.

mod bed;
mod blend;
mod depth;
mod ember;
mod floor;
mod follow;
mod height;
mod liquid;
mod material;
mod mesh;
mod order;
mod seat;
mod swell;
mod uniform;
pub mod wake;
mod waves;

#[cfg(test)]
mod tests_curve;
#[cfg(test)]
mod tests_globe;
#[cfg(test)]
mod tests_height;
#[cfg(test)]
mod tests_light;
#[cfg(test)]
mod tests_shelf;
#[cfg(test)]
mod tests_sort;
#[cfg(test)]
mod tests_swell;

pub use self::ember::ember as sea_glow;
pub use self::follow::follow_focus;
pub use self::liquid::Liquid;

use bevy::asset::AssetEventSystems;
use bevy::camera::visibility::NoFrustumCulling;
use bevy::ecs::system::SystemParam;
use bevy::light::{NotShadowCaster, NotShadowReceiver};
use bevy::math::{DQuat, DVec3};
use bevy::pbr::DistanceFog;
use bevy::prelude::*;

use self::floor::SeaFloor;
use self::material::WaterMaterial;
use self::seat::Seat;
use self::uniform::{FogLook, WaterUniform};
use self::wake::{SeaWakes, Wakes};
use super::focus::Focus;
use super::{Planet, SEA_LEVEL};
use crate::render::{GameCamera, Murk, Palette, PlaceLit};
use crate::screens::Screen;
use crate::space::Placed;
use crate::wind::Wind;

/// Marks the sea mesh.
#[derive(Component, Debug, Clone, Copy)]
pub struct Sea;

/// Everything the water shader values are computed from.
#[derive(SystemParam)]
pub struct SeaInputs<'w, 's> {
    palette: Res<'w, Palette>,
    wind: Res<'w, Wind>,
    time: Res<'w, Time>,
    wakes: Option<Res<'w, Wakes>>,
    fog: Query<'w, 's, (&'static DistanceFog, Option<&'static Murk>), With<GameCamera>>,
}

impl SeaInputs<'_, '_> {
    /// Returns the shader values for the current sky, wind, time and camera fog, for a seat
    /// whose axes are turned by `pose` in render space. Under the surface the fog the water
    /// reads is the colour of the liquid, which its underside takes.
    fn uniform(&self, pose: Quat) -> WaterUniform {
        let camera = self.fog.iter().next();
        let mut fog = FogLook::from_fog(camera.map(|(fog, _)| fog), self.palette.horizon);
        if let Some(murk) = camera.and_then(|(_, murk)| murk.filter(|m| m.submerged())) {
            fog.color = murk.colour.extend(1.0);
        }
        let wind = follow::wind_in_seat(&self.wind, pose);
        WaterUniform::new(&self.palette, &wind, self.time.elapsed_secs(), &fog)
    }
}

impl SeaInputs<'_, '_> {
    /// Returns the live wakes in the sea coordinates of `seat`, none when nothing stirs the sea.
    fn stirred(&self, seat: &Seat) -> SeaWakes {
        let now = self.time.elapsed_secs();
        match self.wakes.as_deref() {
            Some(wakes) if wakes.stirred(now) => SeaWakes::pack(wakes, now, |at| seat.coords(at)),
            _ => SeaWakes::default(),
        }
    }
}

/// Spawns the sea when the world starts loading, seated under the focus of the streamed planet.
///
/// The mesh and the material are built once. From then on only the pose and the shader values
/// change, also when the anchor is rebased.
pub fn spawn_sea(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<WaterMaterial>>,
    mut images: ResMut<Assets<Image>>,
    world: (SeaInputs, Focus, Option<Res<Planet>>),
) {
    let (inputs, focus, planet) = world;
    let Some(planet) = planet else {
        return;
    };
    let seat = Seat::at(&planet.globe, focus.planet().unwrap_or(DVec3::Y));
    let floor = SeaFloor::new(&mut images);
    let water = MeshMaterial3d(materials.add(WaterMaterial {
        water: inputs.uniform(Quat::IDENTITY),
        floor: floor.image(),
        place: default(),
    }));
    let placed = Placed {
        pos: DVec3::ZERO,
        rot: DQuat::IDENTITY,
    };
    commands.spawn((
        (Sea, Mesh3d(meshes.add(mesh::sea_mesh())), water, floor),
        (seat.sited(), placed, seat),
        Transform::from_xyz(0.0, SEA_LEVEL, 0.0),
        Visibility::Hidden,
        (NotShadowCaster, NotShadowReceiver, NoFrustumCulling),
        DespawnOnEnter(Screen::Title),
    ));
}

/// Registers the water material and the system that keeps the seat and the sea floor map under
/// the focus, before the planet-fixed poses are written so that a new seat is placed at once,
/// and hands the sea the light of the planet once the camera is tracked.
pub(super) fn plugin(app: &mut App) {
    app.add_plugins((material::plugin, wake::plugin))
        .add_systems(Update, follow::refresh.before(super::chunks::place))
        .add_systems(Update, ember::kindle)
        .add_systems(
            PostUpdate,
            swell::swell
                .after(bevy::transform::TransformSystems::Propagate)
                .run_if(resource_exists::<Planet>),
        )
        .add_systems(
            PostUpdate,
            follow::light.after(PlaceLit).before(AssetEventSystems),
        );
    crate::render::warm_material::<WaterMaterial>(app);
}
