//! Bake tests: a far body is drawn from the ball of its real surface with its cloud shell, shows
//! all of it from the hand-over inward and a steady dot from afar, and is lit as the ground is.

use std::time::Duration;

use bevy::asset::AssetPlugin;
use bevy::prelude::*;

use super::Body;
use super::bake::{Baking, bake, shell_radius, shell_triangles};
use super::light::{DETAIL, detail, star_light};
use super::material::BodyMaterial;
use crate::render::clouds::CELL;
use crate::render::palette::Palette;
use crate::render::vantage::{LocalSystem, Nearby, Vantage};
use crate::render::warp::Lens;
use crate::space::Clock;
use crate::universe::{PlanetClass, PlanetRecord, PlanetSpec, Universe};
use crate::world::globe::far::{LIMIT, TRIANGLES, handover};

/// Returns the home planet of seed 7, and the same planet made a lava world with seas.
fn worlds() -> (PlanetSpec, PlanetSpec) {
    let universe = Universe::new(7);
    let home = universe.planet(universe.home());
    let record = PlanetRecord {
        class: PlanetClass::Lava,
        ocean: 0.4,
        ..home.record
    };
    let lava = PlanetSpec {
        record,
        ..home.clone()
    };
    (home, lava)
}

/// The home planet is baked with its cloud shell halfway up its cloud band, 960 triangles marked
/// for the cloud shader around those of the ground, and a body whose class has no clouds
/// gets none.
#[test]
fn a_cloudy_body_is_baked_with_its_cloud_shell() {
    let (home, _) = worlds();
    let shell = shell_radius(&home).expect("the home planet has clouds");
    assert_eq!(
        shell,
        home.radius + 0.5 * (home.cloud_base + home.cloud_top)
    );
    let baked = bake(&home);
    let mut material = BodyMaterial::planet(&home);
    material.baked(&baked);
    assert_eq!(material.body.map.w, shell as f32 / CELL);
    let bare = PlanetSpec {
        record: PlanetRecord {
            class: PlanetClass::Barren,
            ..home.record
        },
        ..home.clone()
    };
    assert_eq!(shell_radius(&bare), None);
    let mesh = baked.ball.mesh;
    assert_eq!(shell_triangles(&mesh), 960);
    assert_eq!(mesh.indices().map(|i| i.len() / 3), Some(TRIANGLES + 960));
}

/// Once baked, the material shows the real surface: its mean colour for the dot, and on a lava
/// world the light of the sea for the disc and its mean over the planet for the dot.
#[test]
fn the_baked_material_carries_the_real_surface() {
    let (home, lava) = worlds();
    let mut water = BodyMaterial::planet(&home);
    assert_eq!((water.body.map, water.body.glow), (Vec4::ZERO, Vec4::ZERO));
    water.baked(&bake(&home));
    let body = water.body;
    assert!(
        body.color.z > body.color.x && body.color.w == 0.0,
        "{}",
        body.color
    );
    assert_eq!((body.glow, body.ember), (Vec4::W, Vec4::ZERO));
    assert_eq!(body.map.truncate(), Vec3::ZERO);
    let mut molten = BodyMaterial::planet(&lava);
    molten.baked(&bake(&lava));
    let look = PlanetClass::Lava.look();
    assert_eq!(molten.body.glow, (look.sea * look.glow).extend(1.0));
    let ember = molten.body.ember.truncate();
    assert!(ember.x > 0.2 && ember.x < molten.body.glow.x, "{ember}");
}

/// A body is a dot in its mean colour up to about a pixel, shows all of its surface from three
/// pixels on, never less as it grows, and all of it wherever the world hands it to the sky.
#[test]
fn a_body_shows_its_surface_from_the_hand_over_inward() {
    assert_eq!((detail(0.0), detail(DETAIL.0)), (0.0, 0.0));
    assert_eq!((detail(DETAIL.1), detail(1.0)), (1.0, 1.0));
    let steps: Vec<f32> = (0..=40).map(|k| detail(k as f32 * 0.0005)).collect();
    assert!(steps.windows(2).all(|pair| pair[1] >= pair[0]));
    for radius in [3000.0, 6000.0, 40_000.0, 150_000.0] {
        let at = handover(radius, None);
        assert_eq!(detail((radius / at) as f32), 1.0, "{radius}");
        assert!(radius / at >= 1.0 / LIMIT);
    }
}

/// A body is lit with the light the ground gets in open space: the lux of the star in its
/// colour, the strongest channel at full strength.
#[test]
fn a_body_gets_the_light_of_the_ground() {
    assert_eq!(star_light(Vec3::ONE), Vec3::splat(Palette::SPACE_LUX));
    let red = star_light(Vec3::new(0.8, 0.4, 0.2));
    assert_eq!(red.x, Palette::SPACE_LUX);
    assert!(red.y < 0.3 * red.x && red.z < red.y, "{red}");
}

/// Returns a headless app that draws the bodies of `near`, run until every ball is baked.
fn settled(near: &Nearby) -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .init_asset::<Mesh>()
        .init_asset::<Image>()
        .insert_resource(Lens::default())
        .insert_resource(Clock::default())
        .insert_resource(Vantage::default())
        .insert_resource(LocalSystem(Some(near.clone())))
        .add_plugins((crate::render::atmosphere::plugin, super::plugin));
    let mut baking = app.world_mut().query::<&Baking>();
    for _ in 0..3000 {
        app.update();
        if baking.iter(app.world()).next().is_none() {
            break;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    app
}

/// In a headless app every solid body of the home system gets the ball of its real surface off
/// the main thread, in place of the plain sphere, and a gas giant keeps the sphere.
#[test]
fn every_solid_body_gets_its_ball() {
    let universe = Universe::new(7);
    let near = Nearby::read(&universe, universe.home().star);
    let mut app = settled(&near);
    let mut bodies = app
        .world_mut()
        .query::<(&Body, &MeshMaterial3d<BodyMaterial>)>();
    let materials = app.world().resource::<Assets<BodyMaterial>>();
    let mut solid = 0;
    for (body, handle) in bodies.iter(app.world()) {
        let Body::Planet(id) = *body else {
            continue;
        };
        let giant = near.find(id).unwrap().record.class == PlanetClass::GasGiant;
        let baked = materials.get(&handle.0).unwrap().body.glow.w == 1.0;
        assert_eq!(baked, !giant, "{id:?}");
        solid += usize::from(!giant);
    }
    assert!(solid >= 1 && solid <= near.bodies().count());
}
