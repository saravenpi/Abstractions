//! The sky light Solari needs: without an environment map every shadow and every sky ray is black.

use bevy::asset::RenderAssetUsages;
use bevy::ecs::schedule::SystemCondition;
use bevy::light::EnvironmentMapLight;
use bevy::prelude::*;
use bevy::render::render_resource::{
    Extent3d, TextureDimension, TextureFormat, TextureViewDescriptor, TextureViewDimension,
};

use super::RayTracing;
use crate::render::Palette;
use crate::render::vantage::Vantage;

/// Share of the ambient color that comes up from the ground.
const GROUND: f32 = 0.6;

/// Converts an sRGB triple to the four bytes of a texel.
fn texel(color: Vec3) -> [u8; 4] {
    let byte = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    [byte(color.x), byte(color.y), byte(color.z), 255]
}

/// Returns the six cube faces in the order +X, -X, +Y, -Y, +Z, -Z: ambient color around and above,
/// dimmer on the faces that look down, away from the local `up`.
pub fn faces(palette: &Palette, up: Vec3) -> Vec<u8> {
    let normals = [
        Vec3::X,
        Vec3::NEG_X,
        Vec3::Y,
        Vec3::NEG_Y,
        Vec3::Z,
        Vec3::NEG_Z,
    ];
    let shade = |normal: Vec3| 1.0 - (1.0 - GROUND) * (-normal.dot(up)).max(0.0);
    normals
        .map(|normal| texel(palette.ambient * shade(normal)))
        .concat()
}

/// Builds the one-texel-per-face sky cube from the palette and the local up.
fn cube(palette: &Palette, up: Vec3) -> Image {
    let mut image = Image::new(
        Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 6,
        },
        TextureDimension::D2,
        faces(palette, up),
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    image.texture_view_descriptor = Some(TextureViewDescriptor {
        dimension: Some(TextureViewDimension::Cube),
        ..default()
    });
    image
}

/// The sky cube while ray tracing is active.
#[derive(Resource, Default)]
struct SkyCube(Option<Handle<Image>>);

/// Returns the sky cube, creating it on first use, with its texels matching the palette and the local up.
fn paint(images: &mut Assets<Image>, sky: &mut SkyCube, look: (&Palette, Vec3)) -> Handle<Image> {
    let (palette, up) = look;
    let handle = sky
        .0
        .get_or_insert_with(|| images.add(cube(palette, up)))
        .clone();
    let bytes = faces(palette, up);
    if let Some(mut image) = images.get_mut(&handle)
        && image.data.as_deref() != Some(bytes.as_slice())
    {
        image.data = Some(bytes);
    }
    handle
}

/// Gives the overlay camera the sky light, or refreshes it.
///
/// Solari reads the environment map of any active camera. The overlay camera draws no 3D, so the game
/// camera and the forward lighting of creatures and particles stay exactly as they are. The palette
/// already carries the real sun direction and the thinning of the light with height.
fn show(
    mut commands: Commands,
    look: (Res<Palette>, Res<Vantage>),
    mut images: ResMut<Assets<Image>>,
    mut sky: ResMut<SkyCube>,
    mut cameras: Query<(Entity, Option<&mut EnvironmentMapLight>), With<Camera2d>>,
) {
    let (palette, vantage) = look;
    let handle = paint(&mut images, &mut sky, (&palette, vantage.up));
    let light = EnvironmentMapLight {
        diffuse_map: handle.clone(),
        specular_map: handle,
        intensity: palette.ambient_brightness,
        ..default()
    };
    for (camera, current) in &mut cameras {
        match current {
            Some(mut current) => *current = light.clone(),
            None => {
                commands.entity(camera).insert(light.clone());
            }
        }
    }
}

/// Takes the sky light off the overlay camera and frees the cube.
fn hide(
    mut commands: Commands,
    mut sky: ResMut<SkyCube>,
    cameras: Query<Entity, (With<Camera2d>, With<EnvironmentMapLight>)>,
) {
    sky.0 = None;
    for camera in &cameras {
        commands.entity(camera).remove::<EnvironmentMapLight>();
    }
}

/// Registers the sky light update.
pub(super) fn plugin(app: &mut App) {
    let changed = resource_changed::<Palette>.or_eager(resource_changed::<RayTracing>);
    app.init_resource::<SkyCube>().add_systems(
        Update,
        (
            show.run_if(super::active.and_then(changed)),
            hide.run_if(resource_changed::<RayTracing>.and_eager(not(super::active))),
        ),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::clock::START_PHASE;

    /// The cube has six texels, with a dimmer one below.
    #[test]
    fn faces_are_dimmer_below() {
        let bytes = faces(&Palette::at(START_PHASE), Vec3::Y);
        assert_eq!(bytes.len(), 24);
        assert!(bytes[12] < bytes[8]);
        assert_eq!(&bytes[8..12], &bytes[0..4]);
    }
}
