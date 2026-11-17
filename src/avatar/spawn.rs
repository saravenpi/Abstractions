//! Spawning the body under the player: its segments, materials, screen, fabric, plumes and light.

use bevy::camera::visibility::NoFrustumCulling;
use bevy::ecs::system::SystemParam;
use bevy::light::NotShadowCaster;
use bevy::material::OpaqueRendererMethod;
use bevy::prelude::*;

use super::Avatar;
use super::animate::{Anim, Bone};
use super::face::Frame;
use super::face::machine::Visor;
use super::face::paint::picture;
use super::fx::plume_color;
use super::mesh::{SCREEN_FRONT, fabric, glow, screen, segment};
use super::pose::Pose;
use super::rig::{SEGS, Seg, locals};
use super::tune::{ACCENT, ANKLE_RISE, HEAD, LIGHT_RANGE, NECK, NOZZLE};
use super::wing::Panel;
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
}

/// Marks the faint light in front of the screen.
#[derive(Component, Debug, Clone, Copy)]
pub struct Lamp;

/// Marks the plume under a boot.
#[derive(Component, Debug, Clone, Copy)]
pub struct Plume;

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

/// Returns a forward material: `lit` for the matte shell coloured by the vertex colours, or unlit,
/// which shows its base colour as it is and so can glow through bloom.
pub(super) fn paint(base: Color, lit: bool, picture: Option<Handle<Image>>) -> StandardMaterial {
    StandardMaterial {
        base_color: base,
        base_color_texture: picture,
        unlit: !lit,
        perceptual_roughness: 0.85,
        reflectance: 0.2,
        opaque_render_method: OpaqueRendererMethod::Forward,
        ..default()
    }
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
    }
}

/// Spawns the fifteen segments under `root`, each a child of its parent segment, and returns them.
///
/// Every part states its visibility itself, so the body does not depend on the renderer adding it.
fn bones(commands: &mut Commands, stock: &mut Stock, root: Entity) -> [Entity; SEGS] {
    let shell = stock.materials.add(paint(Color::WHITE, true, None));
    let rest = locals(&Pose::default());
    let mut shapes: Vec<Handle<Mesh>> = Vec::with_capacity(SEGS);
    let mut all = [root; SEGS];
    for seg in Seg::ALL {
        let shape = match seg.side() {
            Some(1) => shapes[seg as usize - 1].clone(),
            _ => stock.meshes.add(segment(seg).into_mesh()),
        };
        shapes.push(shape.clone());
        let parent = seg.parent().map_or(root, |parent| all[parent as usize]);
        let bone = (
            Bone(seg),
            Mesh3d(shape),
            MeshMaterial3d(shell.clone()),
            rest[seg as usize],
            Visibility::default(),
        );
        all[seg as usize] = commands.spawn((bone, ChildOf(parent))).id();
    }
    all
}

/// Spawns the glowing parts: the screen on the head, the light in front of it, and the boot burners.
fn glows(commands: &mut Commands, stock: &mut Stock, kit: &Kit, bones: &[Entity; SEGS]) {
    let face = (
        Mesh3d(stock.meshes.add(screen())),
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
    commands.spawn((Lamp, light, ahead, ChildOf(bones[Seg::Torso as usize])));
    burners(commands, stock, kit, bones);
}

/// Spawns a nozzle glow and a hidden plume under each boot.
fn burners(commands: &mut Commands, stock: &mut Stock, kit: &Kit, bones: &[Entity; SEGS]) {
    let (disc, cone) = (stock.meshes.add(glow(0.0)), stock.meshes.add(glow(1.0)));
    let burn = MeshMaterial3d(kit.glow.clone());
    for boot in [bones[Seg::BootL as usize], bones[Seg::BootR as usize]] {
        let under = Transform::from_xyz(0.0, -ANKLE_RISE + 0.004, NOZZLE.1);
        let rim = under.with_scale(Vec3::splat(NOZZLE.0 * 0.85));
        commands.spawn((
            Mesh3d(disc.clone()),
            burn.clone(),
            rim,
            Visibility::default(),
            NotShadowCaster,
            ChildOf(boot),
        ));
        let plume = (
            Plume,
            Mesh3d(cone.clone()),
            burn.clone(),
            under,
            Visibility::Hidden,
        );
        commands.spawn((plume, NotShadowCaster, ChildOf(boot)));
    }
}

/// Spawns the three panels of fabric under `root`, folded and hidden.
fn cloth(commands: &mut Commands, stock: &mut Stock, root: Entity) {
    let accent = Color::srgb(ACCENT[0], ACCENT[1], ACCENT[2]);
    let material = stock.materials.add(super::warm::sheet(accent));
    for panel in Panel::ALL {
        let mesh = Mesh3d(stock.meshes.add(fabric()));
        let sheet = (
            panel,
            mesh,
            MeshMaterial3d(material.clone()),
            Visibility::Hidden,
        );
        commands.spawn((sheet, NoFrustumCulling, ChildOf(root)));
    }
}

/// Spawns the body as a child of the player when the player has none. It is despawned with the player.
pub fn spawn(
    mut commands: Commands,
    mut stock: Stock,
    players: Query<Entity, With<Player>>,
    existing: Query<(), With<Avatar>>,
) {
    let Some(player) = players.iter().next().filter(|_| existing.is_empty()) else {
        return;
    };
    let kit = kit(&mut stock);
    let body = (
        Avatar,
        Anim::default(),
        Visor::default(),
        Transform::default(),
        Visibility::default(),
    );
    let root = commands
        .spawn((Name::new("Avatar"), body, ChildOf(player)))
        .id();
    let bones = bones(&mut commands, &mut stock, root);
    glows(&mut commands, &mut stock, &kit, &bones);
    cloth(&mut commands, &mut stock, root);
    commands.entity(root).insert(kit);
}
