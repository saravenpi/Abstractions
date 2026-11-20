//! Tests of the plants the particles read: leaves only near leafy plants, in the colour of their species.

use bevy::ecs::system::RunSystemOnce;

use super::*;
use crate::effects::rate::Air;
use crate::effects::sense::Sensed;
use crate::life::flora::Stage;
use crate::world::canopy::Crown;

/// A breezy day at home.
const WINDY: Air = Air {
    speed: 5.0,
    thickness: 1.0,
    rain: 0.0,
};

/// Returns a crown `away` metres from the eye along x, shedding `leaf` and `flower`.
fn crown(away: f32, leaf: Option<[f32; 3]>, flower: Option<[f32; 3]>) -> Crown {
    Crown {
        at: Vec3::new(away, 0.0, 0.0),
        height: 6.0,
        reach: 2.0,
        foot: 0.3,
        shed: Some(Shed {
            leaf,
            size: 0.1,
            flower,
            stage: Stage::Fruit,
            deciduous: true,
            stress: 0.0,
        }),
    }
}

/// Returns the world after one gather around an eye at the origin over a chunk holding `crowns`.
fn gathered(crowns: Vec<Crown>) -> World {
    let mut world = World::new();
    world.init_resource::<Assets<Mesh>>();
    world.init_resource::<Assets<StandardMaterial>>();
    world.init_resource::<Eye>();
    world.init_resource::<Sensed>();
    world
        .run_system_once(super::super::pool::build)
        .expect("the pool");
    world.spawn((Canopy(crowns), GlobalTransform::IDENTITY));
    world.run_system_once(gather).expect("the gather");
    world
}

/// Under bare trees nothing sheds a leaf; a leafy tree nearby gives leaves of its own colour.
#[test]
fn leaves_come_only_from_leafy_plants() {
    let mut bare = gathered(vec![crown(3.0, None, None)]);
    let shed = &mut bare.resource_mut::<Sensed>().shed;
    shed.weigh(&WINDY);
    assert_eq!((shed.leaves(), shed.choose(Kind::Leaf, 0.5)), (0.0, None));
    let green = [0.1, 0.4, 0.05];
    let mut leafy = gathered(vec![
        crown(3.0, Some(green), None),
        crown(90.0, Some([1.0; 3]), None),
    ]);
    leafy.resource_mut::<Sensed>().shed.weigh(&WINDY);
    let shed = &leafy.resource::<Sensed>().shed;
    assert_eq!(shed.sources.len(), 1, "the far tree is out of reach");
    assert!(shed.leaves() > 0.0);
    let slot = shed.choose(Kind::Leaf, 0.5).expect("a leaf source");
    let pool = leafy.resource::<Pool>();
    let material = leafy
        .resource::<Assets<StandardMaterial>>()
        .get(&pool.leaves[slot]);
    let colour = material.map(|m| m.base_color.to_linear().to_f32_array_no_alpha());
    assert_eq!(colour, Some(green));
    assert_eq!(shed.choose(Kind::Petal, 0.5), None);
}

/// A leaf starts in the crown of its plant and inside the particle box.
#[test]
fn a_leaf_starts_in_its_crown() {
    let mut rng = Rng::default();
    let source = Source {
        at: Vec3::new(4.0, 0.0, 1.0),
        height: 6.0,
        reach: 2.0,
        shed: crown(0.0, Some([0.2; 3]), None)
            .shed
            .expect("a leafy crown"),
        leaf: 1.0,
        petal: 0.0,
        pollen: 0.0,
    };
    for _ in 0..50 {
        let at = start(&source, Kind::Leaf, &mut rng);
        assert!(at.xz().distance(source.at.xz()) <= source.reach + 1e-4);
        assert!((2.7..=6.0).contains(&at.y), "{at}");
    }
}
