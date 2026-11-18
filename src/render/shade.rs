//! Fading the shadows of the sun out with height, before their maps run out.
//!
//! The shadow maps reach 110 m from the camera and Bevy stops them dead there. Looking down from
//! higher up, every shadow used to vanish at once. The sun light is now shared with a twin light
//! that casts no shadow: the higher the camera, the more of the light the twin gives, so the
//! shadows thin out and are gone before the maps end, and the ground is lit the same throughout.

use bevy::light::DirectionalLight;
use bevy::prelude::*;

use super::celestial::smoothstep;
use super::fog::Clearance;
use super::lighting::Sun;
use crate::screens::Screen;

/// Heights of the camera above the ground in metres between which the shadows of the sun fade
/// out: gone well before the 110 m the shadow maps reach.
pub const FADE: (f32, f32) = (45.0, 95.0);

/// Marker for the twin of the sun light that casts no shadow.
#[derive(Component, Debug, Default, Clone, Copy)]
pub struct Fill;

/// The sun light, read where it stands and shared out.
type SunLight<'w, 's> =
    Query<'w, 's, (&'static Transform, &'static mut DirectionalLight), (With<Sun>, Without<Fill>)>;

/// The twin of the sun light.
type FillLight<'w, 's> = Query<
    'w,
    's,
    (&'static mut Transform, &'static mut DirectionalLight),
    (With<Fill>, Without<Sun>),
>;

/// Returns the share of the sun light that casts shadows for a camera `clearance` metres above
/// the ground: all of it near the ground, none from the top of [`FADE`] up.
pub fn shadow_share(clearance: f32) -> f32 {
    1.0 - smoothstep(FADE.0, FADE.1, clearance)
}

/// Returns the lux of the sun light and of its twin when the sun gives `lux` in all and casts
/// shadows or not: together always `lux`, and all of it from the sun without shadows to fade.
pub fn split_lux(lux: f32, shadows: bool, clearance: f32) -> (f32, f32) {
    let share = if shadows {
        shadow_share(clearance)
    } else {
        1.0
    };
    (lux * share, lux * (1.0 - share))
}

/// Spawns the twin of the sun when the world starts loading.
fn spawn_fill(mut commands: Commands) {
    let light = DirectionalLight {
        shadow_maps_enabled: false,
        illuminance: 0.0,
        ..default()
    };
    commands.spawn((
        Fill,
        light,
        Transform::default(),
        Visibility::default(),
        DespawnOnEnter(Screen::Title),
    ));
}

/// Shares the light the sun was just given with its twin, by the height of the camera above the
/// ground: the twin stands where the sun does, in its colour, and takes the part that casts no
/// shadow. A sun that casts no shadow keeps all of its light.
pub(super) fn split(aloft: Res<Clearance>, mut suns: SunLight, mut fills: FillLight) {
    let Some((pose, mut sun)) = suns.iter_mut().next() else {
        return;
    };
    let (cast, fill) = split_lux(sun.illuminance, sun.shadow_maps_enabled, aloft.0);
    sun.illuminance = cast;
    if cast <= 0.0 && sun.shadow_maps_enabled {
        sun.shadow_maps_enabled = false;
    }
    for (mut transform, mut light) in &mut fills {
        *transform = *pose;
        light.color = sun.color;
        light.illuminance = fill;
    }
}

/// Registers the twin of the sun. The lights run [`split`] right after they aim the sun.
pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Loading), spawn_fill);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Near the ground every shadow is cast as before, and from 95 m up none is, well before the
    /// 110 m the shadow maps reach; in between the shadows only ever thin out as the camera climbs.
    #[test]
    fn shadows_fade_out_before_their_maps_end() {
        for clearance in [-3.0, 0.0, 1.7, 30.0, FADE.0] {
            assert_eq!(shadow_share(clearance), 1.0, "{clearance}");
        }
        for clearance in [FADE.1, 110.0, 5000.0] {
            assert_eq!(shadow_share(clearance), 0.0, "{clearance}");
        }
        assert!(FADE.1 < 110.0);
        let shares: Vec<f32> = (0..=120).map(|m| shadow_share(m as f32)).collect();
        assert!(shares.windows(2).all(|pair| pair[1] <= pair[0]));
        assert!(shares.windows(2).all(|pair| pair[0] - pair[1] < 0.04));
    }

    /// The sun and its twin always give the light of the sun between them, so the ground is lit the
    /// same at every height, and a sun without shadows keeps all of it.
    #[test]
    fn the_twin_takes_what_casts_no_shadow() {
        for clearance in [0.0, 50.0, 70.0, 94.0, 300.0] {
            let (cast, fill) = split_lux(12_000.0, true, clearance);
            assert!((cast + fill - 12_000.0).abs() < 1.0e-2, "{clearance}");
            assert_eq!(split_lux(12_000.0, false, clearance), (12_000.0, 0.0));
        }
        assert_eq!(split_lux(12_000.0, true, 1.7), (12_000.0, 0.0));
        assert_eq!(split_lux(12_000.0, true, 200.0), (0.0, 12_000.0));
    }
}
