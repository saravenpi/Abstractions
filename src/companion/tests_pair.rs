//! Two companions on screen: each drives its own hands, light and face from its own pose.

use std::time::Duration;

use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;

use super::face::Expression;
use super::face::machine::Face;
use super::limbs::{Hand, limbs};
use super::motion::Float;
use super::pose::Pose;
use super::puppet::Parts;
use super::screen::{Display, repaint};
use super::sense::CompanionSense;
use super::shine::{Lamp, shine};
use super::spawn::{Stock, look};

/// Builds a headless app that runs the hands, the screen and the glow of every companion.
fn app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default(), TransformPlugin))
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
            16,
        )))
        .init_asset::<Mesh>()
        .init_asset::<Image>()
        .init_asset::<StandardMaterial>()
        .init_resource::<CompanionSense>()
        .add_systems(Update, (limbs, repaint, shine).chain());
    app
}

/// Spawns a companion with `companion::look` at `x` metres along, posed by `pose`, flying
/// `flight` of the way and showing `expr`. Returns its root.
fn puppet(app: &mut App, x: f32, (pose, flight): (Pose, f32), expr: Expression) -> Entity {
    let at = Transform::from_xyz(x, 0.0, 0.0);
    let spawn = move |mut commands: Commands, mut stock: Stock| look(&mut commands, &mut stock, at);
    let root = app.world_mut().run_system_once(spawn).unwrap();
    let mut face = Face::new(x.to_bits().into());
    for _ in 0..30 {
        face.advance(expr, 0.016);
    }
    let float = Float {
        pose,
        flight,
        ..default()
    };
    app.world_mut().entity_mut(root).insert((face, float));
    root
}

/// Returns how far apart the two hands of the companion `root` float, the colour of its light
/// and the expression painted on its screen.
fn parts(app: &App, root: Entity) -> (f32, Color, Option<Expression>) {
    let world = app.world();
    let own = world.get::<Parts>(root).unwrap();
    let rig = world.get::<Children>(own.rig).unwrap();
    let spread = rig
        .iter()
        .filter_map(|child| Some((world.get::<Hand>(child)?, world.get::<Transform>(child)?)))
        .map(|(hand, at)| at.translation.x * hand.side)
        .sum();
    let head = world.get::<Children>(own.head).unwrap();
    let light = head.iter().find_map(|child| {
        let lit = world.get::<Lamp>(child).and(world.get::<PointLight>(child));
        lit.map(|light| (light.color, light.intensity))
    });
    let (color, intensity) = light.unwrap();
    assert!(intensity > 0.0, "the light of {root} is off");
    let shown = world
        .get::<Display>(root)
        .unwrap()
        .shown
        .map(|frame| frame.expr);
    (spread, color, shown)
}

/// Two companions spawned with `companion::look`, one stretching its hands out and happy, the
/// other at rest and curious, each get their own hands, light and face: with one `Single` per
/// part, the second companion used to stop the systems for both.
#[test]
fn two_companions_each_drive_their_own_parts() {
    let mut app = app();
    let wide = Pose {
        spread: 1.0,
        ..default()
    };
    let happy = puppet(&mut app, 0.0, (wide, 1.0), Expression::Happy);
    let curious = puppet(&mut app, 3.0, (Pose::default(), 0.0), Expression::Curious);
    for _ in 0..90 {
        app.update();
    }
    let (open, warm, shown) = parts(&app, happy);
    let (shut, cool, seen) = parts(&app, curious);
    assert!(open > shut + 0.1, "hands {open} m apart against {shut} m");
    assert_eq!(shown, Some(Expression::Happy));
    assert_eq!(seen, Some(Expression::Curious));
    assert!(
        warm.to_linear().red > cool.to_linear().red + 0.2,
        "{warm:?} against {cool:?}"
    );
    let glows = |root| app.world().get::<Display>(root).unwrap().thrust;
    assert!(
        glows(happy) > glows(curious) + 0.5,
        "the flying one glows no brighter"
    );
}
