//! Spawning the body under the player: its segments, materials, screen, fabric, plumes and light.

use bevy::camera::visibility::NoFrustumCulling;
use bevy::ecs::system::SystemParam;
use bevy::light::NotShadowCaster;
use bevy::prelude::*;

use super::animate::{Anim, Bone, Skeleton};
use super::face::Frame;
use super::face::machine::Visor;
use super::face::paint::picture;
use super::fx::plume_color;
use super::kick::Kick;
use super::looks::{Looks, paint};
use super::mesh::{SCREEN_FRONT, fabric};
use super::pose::Pose;
use super::rig::{SEGS, Seg, locals};
use super::tune::{ANKLE_RISE, HEAD, LIGHT_RANGE, NECK, NOZZLE};
use super::wing::Panel;
use super::{Avatar, AvatarPose};
use crate::player::Player;

/// How far in front of the screen the light sits, in meters, so it does not burn a spot on the bezel.
const LAMP_REACH: f32 = 0.16;

/// The picture and the materials that change while the body lives, and what they currently show.
#[derive(Component, Debug, Clone)]
pub struct Kit {
    /// The picture on the screen.
    pub image: Handle<Image>,
    /// Unlit material of the screen, tinted by the face colour.
    pub visor: Handle<StandardMaterial>,
    /// Unlit material of the nozzle glows and the plumes, brightened by the thrust.
    pub glow: Handle<StandardMaterial>,
    /// The frame painted last.
    pub shown: Option<Frame>,
    /// Linear face colour as last applied to the screen and the light.
    pub tint: Vec3,
    /// Thrust as last applied to the glow.
    pub thrust: f32,
    /// Share of its light the screen shows, as last applied: less while the head is hurt.
    pub dim: f32,
}

/// Marks the faint light in front of the screen of the body whose root it names.
#[derive(Component, Debug, Clone, Copy)]
pub struct Lamp(pub Entity);

/// Marks the plume under a boot of the body whose root it names.
#[derive(Component, Debug, Clone, Copy)]
pub struct Plume(pub Entity);

/// The asset stores the parts are created in.
#[derive(SystemParam)]
pub struct Stock<'w> {
    /// Mesh assets.
    pub(super) meshes: ResMut<'w, Assets<Mesh>>,
    /// Material assets.
    pub(super) materials: ResMut<'w, Assets<StandardMaterial>>,
    /// Image assets.
    images: ResMut<'w, Assets<Image>>,
}

/// Creates the picture and the materials that follow the face and the thrust.
pub(super) fn kit(stock: &mut Stock) -> Kit {
    let image = stock.images.add(picture(&Frame::default()));
    Kit {
        visor: stock
            .materials
            .add(paint(Color::BLACK, false, Some(image.clone()))),
        glow: stock.materials.add(paint(plume_color(0.0), false, None)),
        image,
        shown: None,
        tint: Vec3::ZERO,
        thrust: 0.0,
        dim: 1.0,
    }
}

/// Spawns the fifteen segments under `root`, each a child of its parent segment, and returns them.
///
/// Every part states its visibility itself, so the body does not depend on the renderer adding it.
fn bones(commands: &mut Commands, looks: &Looks, root: Entity) -> [Entity; SEGS] {
    let rest = locals(&Pose::default());
    let mut all = [root; SEGS];
    for seg in Seg::ALL {
        let parent = seg.parent().map_or(root, |parent| all[parent as usize]);
        let bone = (
            Bone(seg),
            Mesh3d(looks.segments[seg as usize].clone()),
            MeshMaterial3d(looks.shell.clone()),
            rest[seg as usize],
            Visibility::default(),
        );
        all[seg as usize] = commands.spawn((bone, ChildOf(parent))).id();
    }
    all
}

/// Spawns the glowing parts of the body `root`: the screen on the head, the light in front of
/// it, and the boot burners.
fn glows(
    commands: &mut Commands,
    looks: &Looks,
    kit: &Kit,
    (root, bones): (Entity, &[Entity; SEGS]),
) {
    let face = (
        Mesh3d(looks.screen.clone()),
        MeshMaterial3d(kit.visor.clone()),
        Visibility::default(),
    );
    commands.spawn((face, NotShadowCaster, ChildOf(bones[Seg::Head as usize])));
    let light = PointLight {
        intensity: 0.0,
        range: LIGHT_RANGE,
        radius: 0.05,
        shadow_maps_enabled: false,
        ..default()
    };
    let ahead = Transform::from_xyz(NECK[0], NECK[1] + HEAD.0, -(SCREEN_FRONT + LAMP_REACH));
    let torso = bones[Seg::Torso as usize];
    commands.spawn((Lamp(root), light, ahead, ChildOf(torso)));
    let burn = MeshMaterial3d(kit.glow.clone());
    for boot in [bones[Seg::BootL as usize], bones[Seg::BootR as usize]] {
        let under = Transform::from_xyz(0.0, -ANKLE_RISE + 0.004, NOZZLE.1);
        let rim = under.with_scale(Vec3::splat(NOZZLE.0 * 0.85));
        let disc = (Mesh3d(looks.disc.clone()), burn.clone(), rim);
        commands.spawn((disc, Visibility::default(), NotShadowCaster, ChildOf(boot)));
        let plume = (Plume(root), Mesh3d(looks.cone.clone()), burn.clone(), under);
        commands.spawn((plume, Visibility::Hidden, NotShadowCaster, ChildOf(boot)));
    }
}

/// Spawns the three panels of fabric under `root`, folded and hidden. Each body drapes its own
/// meshes; the material is shared.
fn cloth(commands: &mut Commands, stock: &mut Stock, looks: &Looks, root: Entity) {
    for panel in Panel::ALL {
        let sheet = (
            panel,
            Mesh3d(stock.meshes.add(fabric())),
            MeshMaterial3d(looks.cloth.clone()),
            Visibility::Hidden,
        );
        commands.spawn((sheet, NoFrustumCulling, ChildOf(root)));
    }
}

/// Spawns the look of a robot, with no player and no controller, and returns its root: the
/// segments from the shared `looks`, the screen, the fabric, the plumes and the light, posed
/// every frame from the [`AvatarPose`] on the root. Whatever writes that pose drives the body:
/// the controller of the player at this machine, or a pose received from somewhere else.
///
/// The root is made a child of `parent` when one is given, and is despawned with it.
pub fn puppet(
    commands: &mut Commands,
    stock: &mut Stock,
    looks: &Looks,
    parent: Option<Entity>,
) -> Entity {
    let kit = kit(stock);
    let body = (
        Avatar,
        AvatarPose::default(),
        Anim::default(),
        Kick::default(),
        Visor::default(),
        Transform::default(),
        Visibility::default(),
    );
    let root = commands.spawn((Name::new("Avatar"), body)).id();
    if let Some(parent) = parent {
        commands.entity(root).insert(ChildOf(parent));
    }
    let bones = bones(commands, looks, root);
    glows(commands, looks, &kit, (root, &bones));
    cloth(commands, stock, looks, root);
    commands.entity(root).insert((kit, Skeleton(bones)));
    root
}

/// Gives every player that has no body one, as its child, from the shared look, which is
/// built on the first call. A body is despawned with its player.
pub fn spawn(
    mut commands: Commands,
    mut stock: Stock,
    looks: Option<Res<Looks>>,
    players: Query<Entity, With<Player>>,
    existing: Query<&ChildOf, With<Avatar>>,
) {
    let bare = |player: &Entity| !existing.iter().any(|body| body.parent() == *player);
    if !players.iter().any(|player| bare(&player)) {
        return;
    }
    let looks = match looks {
        Some(looks) => looks.clone(),
        None => Looks::new(&mut stock.meshes, &mut stock.materials),
    };
    for player in players.iter().filter(bare) {
        puppet(&mut commands, &mut stock, &looks, Some(player));
    }
    commands.insert_resource(looks);
}
