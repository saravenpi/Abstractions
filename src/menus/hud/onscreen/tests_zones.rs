//! Tests of the room the markers leave the rest of the HUD: whatever the direction of the
//! target, the window and the interface size, a marker covers neither the compass, the place
//! strip, the minimap, the instruments, the prompt nor the hotbar, and stays on the screen.

use bevy::prelude::*;

use super::place::place;
use super::tune::HALF;
use super::zones::{fit, safe};
use crate::craft::ui::hotbar::SIDE;
use crate::menus::hud::compass::HEIGHT;
use crate::menus::hud::minimap::widget;
use crate::menus::hud::needle::WIDTH;
use crate::settings::InterfaceSize;
use crate::ui::{theme, ui_scale};

/// Vertical field of view of the cases, in radians.
const FOV: f32 = 1.3;

/// Returns the rectangles the rest of the HUD draws in on a screen of `screen` interface
/// pixels, measured from the layout itself: the compass, the hotbar, the prompt row, the place
/// strip, the minimap and, while `seated`, the instruments.
fn elements(screen: Vec2, seated: bool) -> Vec<Rect> {
    let (mid, edge) = (screen.x * 0.5, theme::EDGE);
    let bar = (6.0 * SIDE + 5.0 * theme::S2) * 0.5;
    let floor = screen.y - edge;
    let map = (screen.y * widget::SIDE / 1080.0).clamp(widget::LIMITS.0, widget::LIMITS.1);
    let mut all = vec![
        Rect::new(mid - WIDTH * 0.5, edge, mid + WIDTH * 0.5, edge + HEIGHT),
        Rect::new(mid - bar, floor - SIDE, mid + bar, floor),
        Rect::new(
            mid - bar,
            floor - SIDE - 44.0,
            mid + bar,
            floor - SIDE - 8.0,
        ),
        Rect::new(edge, edge, edge + 264.0, edge + 72.0),
        Rect::new(
            screen.x - edge - map,
            edge,
            screen.x - edge,
            edge + map + widget::CAPTION,
        ),
    ];
    if seated {
        all.push(Rect::new(edge, floor - 240.0, edge + 300.0, floor));
    }
    all
}

/// Returns every direction of a grid around the player, in the camera frame.
fn directions() -> Vec<Vec3> {
    let (around, high) = (48, 25);
    let turn = |i: i32| i as f32 / around as f32 * std::f32::consts::TAU;
    let tilt = |j: i32| (j as f32 / (high - 1) as f32 - 0.5) * std::f32::consts::PI;
    let aim = |yaw: f32, pitch: f32| {
        Vec3::new(
            yaw.sin() * pitch.cos(),
            pitch.sin(),
            -yaw.cos() * pitch.cos(),
        )
    };
    (0..around)
        .flat_map(|i| (0..high).map(move |j| aim(turn(i), tilt(j))))
        .collect()
}

/// Returns where a marker for a target in `dir` is drawn on `screen`, with its neighbours of
/// the HUD taken into account.
fn drawn(dir: Vec3, screen: Vec2, seated: bool) -> Vec2 {
    fit(place(dir, FOV, screen, safe(screen)).pos, screen, seated)
}

/// Asserts that on `screen`, `seated` or not, no marker covers another element of the HUD or
/// leaves the screen, whatever the direction of its target.
fn check(screen: Vec2, seated: bool) {
    let half = Vec2::new(HALF.0, HALF.1);
    let taken = elements(screen, seated);
    for dir in directions() {
        let pos = drawn(dir, screen, seated);
        let own = Rect::from_center_half_size(pos, half);
        let on_screen = own.min.cmpge(Vec2::ZERO).all() && own.max.cmple(screen).all();
        assert!(on_screen, "{screen} {dir}: {pos}");
        let clear = taken.iter().all(|other| own.intersect(*other).is_empty());
        assert!(clear, "{screen} seated {seated} {dir}: {pos}");
    }
}

/// At every interface size, in windows from 960 by 540 to 2560 by 1440 and an ultrawide one,
/// on foot and seated, no marker covers another element of the HUD or leaves the screen.
#[test]
fn markers_stay_out_of_the_reserved_zones_at_every_interface_size() {
    let windows = [
        (960.0, 540.0),
        (1280.0, 720.0),
        (1920.0, 1080.0),
        (2560.0, 1440.0),
        (3440.0, 1440.0),
    ];
    for window in windows.map(|(x, y)| Vec2::new(x, y)) {
        for size in InterfaceSize::ALL {
            let screen = window / ui_scale(window, size);
            check(screen, false);
            check(screen, true);
        }
    }
}

/// The safe rectangle keeps the middle of the screen, where the markers of what lies ahead go.
#[test]
fn the_safe_rectangle_keeps_the_middle() {
    for size in InterfaceSize::ALL {
        let window = Vec2::new(960.0, 540.0);
        let screen = window / ui_scale(window, size);
        let room = safe(screen);
        assert!(room.contains(screen * 0.5), "{size:?}: {room:?}");
        assert!(
            room.height() >= 200.0 && room.width() >= 600.0,
            "{size:?}: {room:?}"
        );
        assert_eq!(drawn(Vec3::NEG_Z, screen, false), screen * 0.5);
    }
}
