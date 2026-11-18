//! The glow of a sea of molten rock on the coarse planet meshes and on the planet baked for the far view.
//!
//! The near sea shines through its own shader. Every other mesh of the world shares one material coloured
//! by a lookup texture, so that material gets a second lookup that is black but for the texel of a glowing
//! sea, and the light of the liquid of the planet below: the far sea then gives off the light of the near one.

use bevy::prelude::*;

use super::liquid::Liquid;
use crate::universe::{PlanetClass, PlanetRecord};
use crate::world::lands::Lands;
use crate::world::props::PropAssets;
use crate::world::shapes::srgb;
use crate::world::{Globe, Planet, colormap};

/// Returns the lands of a world of `class` with a sea.
fn wet(class: PlanetClass) -> Lands {
    Lands::of(&PlanetRecord {
        class,
        ocean: PlanetRecord::BALANCED_OCEAN,
        ..PlanetRecord::TEMPERATE
    })
}

/// Builds the lookup of the glow: black everywhere but white on the texel of the coarse sea of every
/// class whose liquid shines, with the size and the sampling of the colour lookup.
pub fn glow_map() -> Image {
    let mut image = colormap::image();
    let size = image.texture_descriptor.size;
    let glowing = PlanetClass::ALL
        .into_iter()
        .filter(|class| class.look().glow > 0.0);
    let texels: Vec<usize> = glowing
        .map(|class| {
            let [r, g, b] = wet(class).sea_rgb();
            let [u, v] = colormap::uv(srgb(r, g, b));
            let (x, y) = (
                (u * size.width as f32) as usize,
                (v * size.height as f32) as usize,
            );
            y * size.width as usize + x
        })
        .collect();
    let data = image.data.get_or_insert_default();
    for (at, texel) in data.as_chunks_mut::<4>().0.iter_mut().enumerate() {
        let lit = if texels.contains(&at) { 255 } else { 0 };
        *texel = [lit, lit, lit, 255];
    }
    image
}

/// Returns the linear light the coarse sea of `globe` gives off: the light of its liquid where it shines
/// and shows, and none on a planet of water, without a sea or without ground.
pub fn ember(globe: Option<&Globe>) -> LinearRgba {
    let liquid = globe.map(Liquid::of);
    let light = liquid
        .filter(|liquid| liquid.shown)
        .map_or(Vec3::ZERO, |liquid| liquid.ember());
    LinearRgba::rgb(light.x, light.y, light.z)
}

/// Gives the shared material of the world the lookup of the glow, once, and the light of the sea of the
/// streamed planet whenever it changes. The material is only written when one of the two differs.
pub fn kindle(
    planet: Option<Res<Planet>>,
    assets: Option<Res<PropAssets>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let Some(assets) = assets else {
        return;
    };
    let light = ember(planet.as_deref().map(|planet| &*planet.globe));
    let settled = |m: &StandardMaterial| m.emissive_texture.is_some() && m.emissive == light;
    if materials.get(&assets.material).is_none_or(settled) {
        return;
    }
    let Some(mut material) = materials.get_mut(&assets.material) else {
        return;
    };
    if material.emissive_texture.is_none() {
        material.emissive_texture = Some(images.add(glow_map()));
    }
    material.emissive = light;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Returns a globe of `class` whose sea covers the `ocean` share of it.
    fn planet(class: PlanetClass, ocean: f64) -> Globe {
        let record = PlanetRecord {
            class,
            ocean,
            ..PlanetRecord::TEMPERATE
        };
        Globe::new(3, 3000.0, 9.8).with_record(record)
    }

    /// The lookup of the glow lights one texel, the one a coarse sea of molten rock reads, and no other.
    #[test]
    fn the_glow_lookup_lights_the_molten_sea_alone() {
        let image = glow_map();
        let (data, size) = (image.data.as_ref().unwrap(), image.texture_descriptor.size);
        let lit = data.as_chunks::<4>().0.iter().filter(|t| t[0] > 0).count();
        assert_eq!(lit, 1);
        let texel = |class: PlanetClass| {
            let [r, g, b] = wet(class).sea_rgb();
            let [u, v] = colormap::uv(srgb(r, g, b));
            let (x, y) = (
                (u * size.width as f32) as usize,
                (v * size.height as f32) as usize,
            );
            data[(y * size.width as usize + x) * 4]
        };
        for class in PlanetClass::ALL {
            let shines = class.look().glow > 0.0;
            assert_eq!(texel(class) == 255, shines, "{class:?}");
        }
    }

    /// The coarse sea of a lava world gives off the light of the near one, by day and by night; water,
    /// a dry lava world and no world give off none.
    #[test]
    fn the_far_sea_gives_off_the_light_of_the_near_sea() {
        let lava = planet(PlanetClass::Lava, 0.3);
        let near = Liquid::of(&lava);
        let far = ember(Some(&lava));
        let look = PlanetClass::Lava.look();
        let want = look.sea * look.glow;
        assert_eq!((far.red, far.green, far.blue), (want.x, want.y, want.z));
        assert_eq!(near.ember(), near.deep * near.glow);
        assert!(far.red > 1.0 && far.red > 3.0 * far.green);
        for dark in [
            planet(PlanetClass::Island, 0.6),
            planet(PlanetClass::Ice, 0.4),
            planet(PlanetClass::Lava, 0.0),
        ] {
            assert_eq!(ember(Some(&dark)), LinearRgba::BLACK);
        }
        assert_eq!(ember(None), LinearRgba::BLACK);
    }
}
