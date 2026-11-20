//! The systems of the displays: each one gets its image and its unlit material when it is
//! fitted, and is repainted only when the face it shows changes.

use bevy::asset::RenderAssetUsages;
use bevy::image::ImageSampler;
use bevy::material::OpaqueRendererMethod;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use super::fit::Lit;
use super::screens::{Display, Face, Screen};
use super::state::Cockpit;

/// Builds the image of `screen`, dark: nearest sampling and no mipmaps, so the pixels stay sharp.
pub fn picture(screen: Screen) -> Image {
    let (width, height) = screen.size();
    let size = Extent3d {
        width: width as u32,
        height: height as u32,
        depth_or_array_layers: 1,
    };
    let mut image = Image::new(
        size,
        TextureDimension::D2,
        screen.painted(Face::Dark).rgba(),
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    image.sampler = ImageSampler::nearest();
    image
}

/// Run condition: a display was just fitted.
pub fn fitted(screens: Query<(), Added<Screen>>) -> bool {
    !screens.is_empty()
}

/// Gives every display that was just fitted its own image and its own unlit material, which
/// shows the image. Ships share every mesh, but never a picture.
pub fn equip(
    mut commands: Commands,
    screens: Query<(Entity, &Screen), Added<Screen>>,
    (mut images, mut materials): (ResMut<Assets<Image>>, ResMut<Assets<StandardMaterial>>),
) {
    for (entity, screen) in &screens {
        let image = images.add(picture(*screen));
        let material = StandardMaterial {
            base_color_texture: Some(image.clone()),
            unlit: true,
            opaque_render_method: OpaqueRendererMethod::Forward,
            ..default()
        };
        commands.entity(entity).insert((
            MeshMaterial3d(materials.add(material)),
            Display { image, shown: None },
            Lit(Vec3::ONE),
        ));
    }
}

/// Repaints the display of every cockpit whose face changed: a value moves the picture only
/// when it moved by a step of the display, so a steady reading paints nothing.
pub fn repaint(
    cockpits: Query<&Cockpit>,
    mut screens: Query<(&Screen, &ChildOf, &mut Display)>,
    mut images: ResMut<Assets<Image>>,
) {
    for (screen, parent, mut display) in &mut screens {
        let Ok(cockpit) = cockpits.get(parent.parent()) else {
            continue;
        };
        let face = screen.face(cockpit);
        if display.shown == Some(face) {
            continue;
        }
        if let Some(mut image) = images.get_mut(&display.image) {
            image.data = Some(screen.painted(face).rgba());
            display.shown = Some(face);
        }
    }
}
