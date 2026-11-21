//! The pictures of the three displays of the dashboard: the drive on the left, with the
//! throttle, the warp drive and the gear as bars; the flight in the middle, with the speed, the
//! height and the attitude ball, on which the course and the body nearby are marked; and the
//! status on the right, seven lamps with their legends. Each painter is a pure function of the
//! steps it shows.

use bevy::prelude::*;

use super::paint::{MARGIN, bar, flush, label, write};
use super::pixels::{Ink, Pixels};
use super::readings::{LAMPS, Lamp};
use super::tune::ANGLE_STEP;

/// Size in pixels of the drive display and of the status display.
pub const SIDE: (usize, usize) = (32, 72);

/// Size in pixels of the flight display.
pub const FLIGHT: (usize, usize) = (72, 72);

/// Centre of the attitude ball in the flight display, its radius, and the pixels a radian of
/// pitch moves its horizon: under the two figures, within the margin.
pub const BALL: (IVec2, f32, f32) = (IVec2::new(36, 56), 13.0, 10.0);

/// Radius in pixels of the area of the ball the marks of the course and of the body move in.
pub const REACH: f32 = 11.0;

/// Rows of a figure of the flight display, from its top: the small legend, the value under it,
/// and the gauge under the value.
pub const FIGURE: (i32, i32, i32) = (0, 6, 16);

/// Rows of the tops of the two figures of the flight display.
pub const FIGURES: [i32; 2] = [2, 21];

/// A figure of the flight display: its words and the step of the gauge under them.
pub type Figure<'a> = (Option<&'a str>, Option<u8>);

/// Paints the drive display: for the throttle, the warp drive and the gear a legend and a bar
/// at its step. A bar without a reading is left out, and with none the display is dark.
pub fn drive(bars: [Option<u8>; 3]) -> Pixels {
    let mut pixels = Pixels::dark(SIDE.0, SIDE.1);
    for (i, (name, step)) in ["THR", "WARP", "GEAR"].into_iter().zip(bars).enumerate() {
        let row = 4 + 23 * i as i32;
        if let Some(step) = step {
            write(&mut pixels, name, (MARGIN, row), Ink::Dim);
            bar(&mut pixels, (row + 10, 7), step);
        }
    }
    pixels
}

/// Paints one figure of the flight display from row `y`: the `legend` in the small font, the
/// words under it against the right edge, and the gauge under them. The legend has its own row,
/// so it shows whatever the length of the words.
fn line(pixels: &mut Pixels, y: i32, legend: &str, (words, gauge): Figure) {
    let (top, value, under) = FIGURE;
    if let Some(words) = words {
        let x = flush(pixels, words.chars().count());
        label(pixels, legend, (MARGIN, y + top), Ink::Dim);
        write(pixels, words, (x.max(MARGIN), y + value), Ink::Lit);
    }
    if let Some(step) = gauge {
        bar(pixels, (y + under, 2), step);
    }
}

/// Paints the attitude ball: its rim, the wings of the ship across its middle and, where there
/// is a horizon, the ground as a grey dither under a lit line. `level` is the pitch up and the
/// roll to the right in steps of the display; the horizon turns the other way from the ship.
fn ball(pixels: &mut Pixels, level: Option<IVec2>) {
    let (centre, round, per_radian) = BALL;
    let reach = round.ceil() as i32;
    let tilt = level.map(|at| {
        let (sin, cos) = (at.y as f32 * ANGLE_STEP).sin_cos();
        (
            sin,
            cos,
            (at.x as f32 * ANGLE_STEP).clamp(-1.2, 1.2) * per_radian,
        )
    });
    for (dx, dy) in (-reach..=reach).flat_map(|dy| (-reach..=reach).map(move |dx| (dx, dy))) {
        let (px, py) = (dx as f32, -dy as f32);
        let out = px.hypot(py);
        let above = tilt.map(|(sin, cos, lift)| -px * sin + py * cos + lift);
        let ink = match above {
            _ if out > round => continue,
            _ if out > round - 1.0 => Ink::Lit,
            Some(above) if above.abs() < 0.75 => Ink::Lit,
            Some(above) if above < 0.0 && (dx + dy).rem_euclid(2) == 0 => Ink::Dim,
            _ => continue,
        };
        pixels.put(centre.x + dx, centre.y + dy, ink);
    }
    for dx in [-6, -5, -4, 0, 4, 5, 6] {
        pixels.put(centre.x + dx, centre.y, Ink::Lit);
    }
}

/// Paints the marks on the ball, each at its point from [`project`](super::attitude::project):
/// the course as a plus, the body nearby as a ring.
fn marks(pixels: &mut Pixels, [course, body]: [Option<IVec2>; 2]) {
    let at = |mark: IVec2| BALL.0 + IVec2::new(mark.x, -mark.y);
    if let Some(ring) = body.map(at) {
        for (dx, dy) in [
            (-2, 0),
            (2, 0),
            (0, -2),
            (0, 2),
            (-1, -1),
            (1, -1),
            (-1, 1),
            (1, 1),
        ] {
            pixels.put(ring.x + dx, ring.y + dy, Ink::Lit);
        }
    }
    if let Some(plus) = course.map(at) {
        for d in -2..=2 {
            pixels.put(plus.x + d, plus.y, Ink::Lit);
            pixels.put(plus.x, plus.y + d, Ink::Lit);
        }
    }
}

/// Paints the flight display: the speed and the height as `figures`, and the ball with the
/// attitude `level` and the `marks` of the course and of the body. The ball is drawn when it
/// has a horizon or a mark, and with nothing to show the display is dark.
pub fn flight(figures: [Figure; 2], level: Option<IVec2>, shown: [Option<IVec2>; 2]) -> Pixels {
    let mut pixels = Pixels::dark(FLIGHT.0, FLIGHT.1);
    line(&mut pixels, FIGURES[0], "SPD", figures[0]);
    line(&mut pixels, FIGURES[1], "ALT", figures[1]);
    if level.is_some() || shown.iter().any(Option::is_some) {
        ball(&mut pixels, level);
        marks(&mut pixels, shown);
    }
    pixels
}

/// Rows between the tops of two lamps of the status display, and the row of the first.
pub const LAMP_ROWS: (i32, i32) = (10, 2);

/// Paints the status display: a lamp and its legend for each of `lamps`, in the order of
/// [`Lamp::ALL`]. A lamp that is out is a grey outline, and one that is lit is filled in its
/// colour, with its legend.
pub fn status(lamps: [bool; LAMPS]) -> Pixels {
    let mut pixels = Pixels::dark(SIDE.0, SIDE.1);
    for (i, (lamp, lit)) in Lamp::ALL.into_iter().zip(lamps).enumerate() {
        let row = LAMP_ROWS.1 + LAMP_ROWS.0 * i as i32;
        let ink = if lit { lamp.ink() } else { Ink::Dim };
        for (dx, dy) in (0..7).flat_map(|dy| (0..7).map(move |dx| (dx, dy))) {
            let edge = dx == 0 || dy == 0 || dx == 6 || dy == 6;
            if lit || edge {
                pixels.put(MARGIN + dx, row + dy, ink);
            }
        }
        write(&mut pixels, lamp.legend(), (MARGIN + 10, row), ink);
    }
    pixels
}
