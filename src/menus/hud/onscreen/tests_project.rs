//! Tests of where a marker on its target lands: on the point the game camera really projects
//! its target to, within a pixel, for several camera poses, fields of view and windows, a Retina
//! one and ones the game image is cropped on, and without trailing the view as it turns.

use bevy::camera::CameraProjection;
use bevy::prelude::*;
use bevy::window::WindowResolution;

use super::pick::State;
use super::place::{Frustum, place};
use crate::menus::MarkerKind;
use crate::render::Fit;
use crate::settings::InterfaceSize;
use crate::ui::ui_scale;

/// Windows of the tests: physical size and scale factor. A plain 720p, a Retina 16:10 laptop,
/// a 16:10 desktop at a fractional scale, an odd laptop height the game image is cropped on,
/// a square window and a very wide one.
const WINDOWS: [(u32, u32, f32); 6] = [
    (1280, 720, 1.0),
    (2560, 1600, 2.0),
    (1920, 1200, 1.25),
    (3456, 2234, 2.0),
    (1000, 1000, 1.0),
    (2234, 900, 1.0),
];

/// Returns a window of `physical` pixels at the scale factor `factor`.
fn window((width, height, factor): (u32, u32, f32)) -> Window {
    let resolution = WindowResolution::new(width, height).with_scale_factor_override(factor);
    Window {
        resolution,
        ..default()
    }
}

/// Returns where Bevy's own projection of a camera with `projection`, rendering the full-window
/// game image that `window` shows, puts the camera frame point `point`, in interface pixels under
/// the interface scale `ui`.
fn projected(projection: &PerspectiveProjection, window: &Window, ui: f32, point: Vec3) -> Vec2 {
    let clip = projection.get_clip_from_view() * point.extend(1.0);
    let ndc = clip.truncate().truncate() / clip.w;
    let physical = Vec2::new(ndc.x + 1.0, 1.0 - ndc.y) * 0.5 * window.physical_size().as_vec2();
    physical / (window.scale_factor() * ui)
}

/// A camera seeing `fov` radians up and down into the game image of `window`, as Bevy sizes
/// it, with the interface scale of that window.
struct Rig {
    /// The window.
    window: Window,
    /// The interface scale.
    ui: f32,
    /// The projection of the camera.
    projection: PerspectiveProjection,
}

impl Rig {
    /// Returns the rig of a window of `size` and a field of view of `fov` radians.
    fn new(size: (u32, u32, f32), fov: f32) -> Self {
        let window = window(size);
        let ui = ui_scale(
            Vec2::new(window.width(), window.height()),
            InterfaceSize::Normal,
        );
        let fit = Fit::of(window.physical_size());
        let aspect_ratio = fit.size.x as f32 / fit.size.y as f32;
        let projection = PerspectiveProjection {
            fov,
            aspect_ratio,
            ..default()
        };
        Self {
            window,
            ui,
            projection,
        }
    }

    /// Returns how far in interface pixels the icon of a target at the camera frame point
    /// `point` lands from where the camera projects it, or nothing when it is not on its place.
    fn off(&self, point: Vec3) -> Option<f32> {
        let projection = Projection::Perspective(self.projection.clone());
        let lens = Frustum::of(Some(&projection), 1.0, &self.window, self.ui);
        let at = place(point.normalize(), lens, Rect::new(-1e6, -1e6, 1e6, 1e6));
        let wanted = projected(&self.projection, &self.window, self.ui, point);
        at.arrow.is_none().then(|| at.pos.distance(wanted))
    }
}

/// For every window, every field of view the view can give, from the setting to the widest
/// it reaches at speed, and points all over the view, the icon of a target sits on the point
/// the camera projects it to, within one interface pixel.
#[test]
fn a_marker_lands_on_the_projected_point_on_every_window() {
    let points = [
        Vec3::new(0.0, 0.0, -10.0),
        Vec3::new(3.0, 1.0, -8.0),
        Vec3::new(-4.0, -2.0, -9.0),
        Vec3::new(5.0, 4.0, -12.0),
        Vec3::new(-25.0, 18.0, -60.0),
    ];
    for size in WINDOWS {
        for fov in [60.0_f32, 75.0, 95.0, 120.0].map(f32::to_radians) {
            let rig = Rig::new(size, fov);
            for point in points {
                let off = rig.off(point).expect("on its place");
                assert!(off <= 1.0, "{size:?} fov {fov}: {point} {off} px off");
            }
        }
    }
}

/// Camera poses turned all ways: the direction of a world point in the camera frame, as the
/// markers take it, lands where the camera projects that world point.
#[test]
fn a_marker_follows_every_camera_pose() {
    let rig = Rig::new((2560, 1600, 2.0), 80.0_f32.to_radians());
    let target = Vec3::new(12.0, 3.0, -40.0);
    let poses = [
        Transform::from_xyz(0.0, 1.6, 0.0).looking_at(Vec3::new(10.0, 2.0, -40.0), Vec3::Y),
        Transform::from_xyz(5.0, 30.0, 10.0).looking_at(Vec3::new(14.0, 0.0, -38.0), Vec3::Y),
        Transform::from_xyz(-20.0, 2.0, -30.0).looking_at(Vec3::new(10.0, 6.0, -42.0), Vec3::Z),
        Transform::from_xyz(12.0, 3.0, 20.0).looking_at(Vec3::new(15.0, 3.0, -40.0), Vec3::X),
    ];
    for pose in poses {
        let camera = GlobalTransform::from(pose);
        let point = camera.affine().inverse().transform_point3(target);
        let off = rig.off(point).expect("on its place");
        assert!(off <= 1.0, "{pose:?}: {off} px off");
    }
}

/// An icon on its target sits on the place of this frame, however little it is asked to
/// follow: it never trails the view while the camera turns. On the border, with its arrow, it
/// still glides there.
#[test]
fn an_icon_on_its_target_never_trails_the_view() {
    use super::place::Placed;
    let mut state = State::default();
    let who = (MarkerKind::Home, 0);
    let first = Placed {
        pos: Vec2::new(400.0, 300.0),
        arrow: None,
    };
    let turned = Placed {
        pos: Vec2::new(460.0, 290.0),
        arrow: None,
    };
    state.settle(0, who, first, 0.1);
    assert_eq!(state.settle(0, who, turned, 0.1), turned.pos);
    let edge = Placed {
        pos: Vec2::new(68.0, 290.0),
        arrow: Some(-1.5),
    };
    let glided = state.settle(0, who, edge, 0.1);
    assert!(glided.distance(edge.pos) > 1.0, "{glided}");
}
