//! Bake tests: a far body is drawn from the texture of its real surface with its cloud shell,
//! shows all of it from where the sky first draws it and a steady dot from afar, and is lit as
//! the ground is.

use std::time::Duration;

use bevy::asset::AssetPlugin;
use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, TaskPool, block_on};
use bevy_persistent::StorageFormat;

use super::Body;
use super::bake::{Surface, Textures, bake, cloudy, shell_radius};
use super::light::star_light;
use super::material::BodyMaterial;
use crate::render::clouds::CELL;
use crate::render::palette::Palette;
use crate::render::vantage::{LocalSystem, Nearby, Vantage};
use crate::render::warp::Lens;
use crate::settings::{Persistent, Settings};
use crate::space::Clock;
use crate::universe::{PlanetClass, PlanetRecord, PlanetSpec, Universe};

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

/// Returns the coarsest level of `body` baked on this thread's pools.
fn baked(body: &PlanetSpec) -> BodyMaterial {
    AsyncComputeTaskPool::get_or_init(TaskPool::new);
    let baked = block_on(bake(body.clone(), None, 0));
    let mut material = BodyMaterial::planet(body);
    let textures = Textures {
        colour: Handle::default(),
        relief: Handle::default(),
    };
    material.baked(body, &baked.ball, textures);
    material
}

/// The home planet has its cloud shell halfway up its cloud band, drawn as a body of its own,
/// and a body whose class has no clouds gets none.
#[test]
fn a_cloudy_body_is_baked_with_its_cloud_shell() {
    let (home, _) = worlds();
    let shell = shell_radius(&home).expect("the home planet has clouds");
    assert_eq!(
        shell,
        home.radius + 0.5 * (home.cloud_base + home.cloud_top)
    );
    let material = baked(&home);
    assert_eq!(material.body.map.w, shell as f32 / CELL);
    assert!(cloudy(&home) && material.surface.is_some() && material.relief.is_some());
    let bare = PlanetSpec {
        record: PlanetRecord {
            class: PlanetClass::Barren,
            ..home.record
        },
        ..home.clone()
    };
    assert_eq!(shell_radius(&bare), None);
    assert!(!cloudy(&bare));
}

/// Once baked, the material shows the real surface: its mean colour for the dot, and on a lava
/// world the light of the sea for the disc and its mean over the planet for the dot.
#[test]
fn the_baked_material_carries_the_real_surface() {
    let (home, lava) = worlds();
    let plain = BodyMaterial::planet(&home);
    assert_eq!((plain.body.map, plain.body.glow), (Vec4::ZERO, Vec4::ZERO));
    let water = baked(&home);
    let body = water.body;
    assert!(
        body.color.z > body.color.x && body.color.w == 0.0,
        "{}",
        body.color
    );
    assert_eq!((body.glow, body.ember), (Vec4::W, Vec4::ZERO));
    assert_eq!(body.map.truncate(), Vec3::ZERO);
    let molten = baked(&lava);
    let look = PlanetClass::Lava.look();
    assert_eq!(molten.body.glow, (look.sea * look.glow).extend(1.0));
    let ember = molten.body.ember.truncate();
    assert!(ember.x > 0.2 && ember.x < molten.body.glow.x, "{ember}");
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

/// Returns default settings stored in a scratch file of their own.
fn settings() -> Persistent<Settings> {
    let dir = format!("abstractions-bodies-test-{}", std::process::id());
    Persistent::<Settings>::builder()
        .name("settings")
        .format(StorageFormat::Toml)
        .path(std::env::temp_dir().join(dir).join("settings.toml"))
        .default(Settings::default())
        .build()
        .unwrap()
}

/// Returns a headless app that draws the bodies of `near`, run until every surface has its first
/// level.
fn settled(near: &Nearby) -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .init_asset::<Mesh>()
        .init_asset::<Image>()
        .insert_resource(Lens::default())
        .insert_resource(Clock::default())
        .insert_resource(Vantage::default())
        .insert_resource(settings())
        .insert_resource(LocalSystem(Some(near.clone())))
        .add_plugins((crate::render::atmosphere::plugin, super::plugin));
    let mut baking = app.world_mut().query::<&Surface>();
    for _ in 0..3000 {
        app.update();
        let done = |surface: &Surface| surface.held[0].is_some();
        if baking.iter(app.world()).count() > 0 && baking.iter(app.world()).all(done) {
            break;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    app
}

/// In a headless app every solid body of the home system gets the texture of its real surface
/// off the main thread, and a gas giant keeps its bands.
#[test]
fn every_solid_body_gets_its_surface() {
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
        let material = materials.get(&handle.0).unwrap();
        let baked = material.body.glow.w == 1.0;
        assert_eq!(baked, !giant, "{id:?}");
        assert_eq!(material.surface.is_some(), !giant, "{id:?}");
        assert_eq!(material.relief.is_some(), !giant, "{id:?}");
        solid += usize::from(!giant);
    }
    assert!(solid >= 1 && solid <= near.bodies().count());
}
