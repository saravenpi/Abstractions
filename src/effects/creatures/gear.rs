//! The few meshes and materials every creature shares, and how each firefly light keeps to one firefly.

use std::f32::consts::FRAC_PI_2;

use bevy::material::OpaqueRendererMethod;
use bevy::prelude::*;

/// Butterfly wing colors: white, yellow, orange and blue.
const WING_COLORS: [[f32; 3]; 4] = [
    [0.96, 0.96, 0.92],
    [0.97, 0.85, 0.2],
    [0.95, 0.5, 0.1],
    [0.3, 0.55, 0.95],
];

/// A light leaves its firefly for a free one only when its own is this many times farther, plus `MARGIN`.
const SLACK: f32 = 1.3;

/// Extra distance in meters a free firefly must be closer by before a light leaves its own.
const MARGIN: f32 = 1.5;

/// A light bound to one firefly: which one it follows and how far it has faded in.
#[derive(Component, Debug, Clone, Copy, Default, PartialEq)]
pub struct Bind {
    /// Pool slot of the firefly it follows, if any.
    pub target: Option<usize>,
    /// Fade from 0 (dark, free to take another firefly) to 1 (fully lit).
    pub level: f32,
}

/// Advances the binding of every light by one frame. `dists[slot]` is the distance of a firefly that is out.
///
/// A light keeps its firefly until that one is gone or clearly farther than the nearest free one. It then fades
/// out by `step` per frame, and only at level zero does it take another firefly: a lit light never changes target.
pub fn rebind(binds: &mut [Bind], dists: &[Option<f32>], step: f32) {
    let reach = |bind: &Bind| {
        bind.target
            .and_then(|slot| dists.get(slot).copied().flatten())
    };
    let mut free = dists
        .iter()
        .enumerate()
        .filter_map(|(slot, dist)| Some((slot, (*dist)?)))
        .filter(|(slot, _)| binds.iter().all(|bind| bind.target != Some(*slot)))
        .min_by(|a, b| a.1.total_cmp(&b.1));
    let loser = free.and_then(|(_, near)| {
        binds
            .iter()
            .enumerate()
            .filter_map(|(i, bind)| Some((i, bind.level, reach(bind)?)))
            .filter(|(.., far)| *far > near * SLACK + MARGIN)
            .max_by(|a, b| b.1.total_cmp(&a.1).then(a.2.total_cmp(&b.2)))
            .map(|(i, ..)| i)
    });
    for (i, bind) in binds.iter_mut().enumerate() {
        let keep = reach(bind).is_some() && loser != Some(i);
        bind.level = (bind.level + if keep { step } else { -step }).clamp(0.0, 1.0);
        if !keep && bind.level == 0.0 {
            bind.target = free.take().map(|(slot, _)| slot);
        }
    }
}

/// Shared handles for fireflies, butterflies and birds.
#[derive(Resource, Debug, Clone)]
pub struct Gear {
    /// Low-poly ball for a firefly.
    pub glow_mesh: Handle<Mesh>,
    /// Yellow-green material of fireflies, emissive enough for bloom to give each one a soft halo.
    pub glow: Handle<StandardMaterial>,
    /// Flat wing hinged at its inner edge, spanning `x` from 0 to 1.
    pub wing: Handle<Mesh>,
    /// Thin butterfly body.
    pub moth_body: Handle<Mesh>,
    /// Butterfly wing materials by color.
    pub moth_wings: Vec<Handle<StandardMaterial>>,
    /// Dark butterfly body material.
    pub moth_dark: Handle<StandardMaterial>,
    /// Small bird body.
    pub bird_body: Handle<Mesh>,
    /// Dark feathers.
    pub crow: Handle<StandardMaterial>,
    /// White feathers.
    pub gull: Handle<StandardMaterial>,
}

/// Returns a rough double sided material of one color.
fn sheet(color: Color) -> StandardMaterial {
    StandardMaterial {
        base_color: color,
        perceptual_roughness: 1.0,
        double_sided: true,
        cull_mode: None,
        opaque_render_method: OpaqueRendererMethod::Forward,
        ..default()
    }
}

/// Builds the shared meshes and materials once at startup.
pub fn build(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let wing = Mesh::from(Rectangle::new(1.0, 1.0))
        .rotated_by(Quat::from_rotation_x(-FRAC_PI_2))
        .translated_by(Vec3::X * 0.5);
    let glow = StandardMaterial {
        emissive: LinearRgba::rgb(16.0, 22.0, 3.5),
        ..sheet(Color::srgb(0.6, 0.75, 0.2))
    };
    let moth_wings = WING_COLORS
        .iter()
        .map(|&[r, g, b]| materials.add(sheet(Color::srgb(r, g, b))))
        .collect();
    commands.insert_resource(Gear {
        glow_mesh: meshes.add(Sphere::new(0.5).mesh().uv(6, 4)),
        glow: materials.add(glow),
        wing: meshes.add(wing),
        moth_body: meshes.add(Cuboid::new(0.015, 0.015, 0.09)),
        moth_wings,
        moth_dark: materials.add(sheet(Color::srgb(0.12, 0.1, 0.1))),
        bird_body: meshes.add(Cuboid::new(0.22, 0.2, 0.8)),
        crow: materials.add(sheet(Color::srgb(0.1, 0.1, 0.13))),
        gull: materials.add(sheet(Color::srgb(0.95, 0.96, 0.98))),
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Fade step of a light per frame.
    const STEP: f32 = 0.05;

    /// With fireflies that drift in and out of reach and sometimes hide, a light changes firefly only while dark.
    #[test]
    fn fireflies_lights_never_jump_while_lit() {
        let dist = |t: f32, slot: f32| 12.0 + 10.0 * (t * (0.2 + 0.03 * slot) + slot).sin();
        let out = |t: f32, slot: f32| (t * 0.11 + slot * 0.7).sin() > -0.6;
        let mut binds = [Bind::default(); 6];
        let mut lit = 0;
        for frame in 0..6000 {
            let t = frame as f32 / 60.0;
            let slots = (0..24).map(|slot| slot as f32);
            let dists: Vec<Option<f32>> = slots.map(|s| out(t, s).then(|| dist(t, s))).collect();
            let before = binds;
            rebind(&mut binds, &dists, STEP);
            for (old, new) in before.iter().zip(&binds) {
                let moved = old.target != new.target;
                assert!(
                    !moved || (new.level == 0.0 && old.level <= STEP + 1e-6),
                    "{old:?} -> {new:?}"
                );
            }
            let held = |slot: usize| binds.iter().filter(|b| b.target == Some(slot)).count();
            assert!(
                (0..24).all(|slot| held(slot) <= 1),
                "two lights share a firefly"
            );
            lit += binds.iter().filter(|b| b.level == 1.0).count();
        }
        assert!(lit > 6000, "lights were fully lit only {lit} times");
    }

    /// Fireflies at nearly the same distance that keep swapping rank never make a light let go.
    #[test]
    fn fireflies_lights_ignore_near_ties() {
        let ties = |frame: usize| -> Vec<Option<f32>> {
            let near = |slot: usize| 5.0 + ((frame + slot) % 3) as f32 * 0.4;
            (0..8).map(|slot| Some(near(slot))).collect()
        };
        let mut binds = [Bind::default(); 6];
        (0..60).for_each(|frame| rebind(&mut binds, &ties(frame), STEP));
        let settled = binds;
        (60..600).for_each(|frame| rebind(&mut binds, &ties(frame), STEP));
        assert_eq!(binds, settled);
        assert!(settled.iter().all(|b| b.level == 1.0 && b.target.is_some()));
    }
}
