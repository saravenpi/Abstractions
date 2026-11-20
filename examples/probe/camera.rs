//! The camera, the robot and the view mode.

use super::*;

/// Metres the probe lowers the game camera by after the transforms are propagated: a low look at the
/// ground without a crouch in the game.
#[derive(Resource, Default)]
pub struct Lower(pub f32);

/// Lowers the game camera along the local up by the metres of [`Lower`].
pub fn lower(
    low: Res<Lower>,
    anchor: Option<Res<Anchor>>,
    mut camera: Query<&mut GlobalTransform, With<abstractions::render::GameCamera>>,
) {
    let (Some(anchor), true) = (anchor, low.0 > 0.0) else {
        return;
    };
    for mut pose in &mut camera {
        let at = pose.translation();
        let up = abstractions::space::up(&anchor, at);
        let mut t = pose.compute_transform();
        t.translation -= up * low.0;
        *pose = GlobalTransform::from(t);
    }
}

/// Returns the planet position of the player.
pub fn player_at(world: &mut World) -> DVec3 {
    let anchor = *world.resource::<Anchor>();
    let at = world
        .query_filtered::<&Transform, With<Player>>()
        .iter(world)
        .next()
        .map_or(Vec3::ZERO, |t| t.translation);
    anchor.to_frame(at)
}

/// Returns the robot's planet position.
pub fn robot(world: &mut World) -> DVec3 {
    let anchor = *world.resource::<Anchor>();
    let at = world
        .query_filtered::<&GlobalTransform, With<Player>>()
        .iter(world)
        .next()
        .map_or(Vec3::ZERO, |g| g.translation());
    anchor.to_frame(at)
}

/// Sets the view on foot and in the ship.
pub fn view(world: &mut World, on_foot: ViewMode, in_ship: ViewMode) {
    let mut prefs = world.resource_mut::<ViewPrefs>();
    prefs.on_foot = on_foot;
    prefs.ship = in_ship;
}

/// Returns the player's height above the ground under it, in metres.
pub fn clearance(world: &mut World) -> f64 {
    let anchor = *world.resource::<Anchor>();
    let at = world
        .query_filtered::<&Transform, With<Player>>()
        .iter(world)
        .next()
        .map_or(Vec3::ZERO, |t| t.translation);
    let planet = anchor.to_frame(at);
    let globe = &world.resource::<Planet>().globe;
    planet.length() - globe.radius() - f64::from(globe.height(planet.normalize()).max(0.0))
}

/// Opens the wingsuit once the player is in the air.
pub fn spread(world: &mut World) {
    for mut motion in world
        .query_filtered::<&mut Motion, With<Player>>()
        .iter_mut(world)
    {
        if !motion.grounded {
            motion.wings = true;
        }
    }
}
