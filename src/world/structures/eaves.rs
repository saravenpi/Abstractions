//! Roofs that rest on the walls of a room: the slopes overhang the wall tops and a fill in the wall color
//! closes the gap between the wall tops and the underside of the roof.

use bevy::prelude::*;

use super::brush::{Build, Mat};
use super::palette::SNOW;
use super::roofs::lean_height;
use super::tags::Tag;

/// The outer faces of the walls a roof rests on: low and high corners, both at the height of the wall tops.
pub type Walls = (Vec3, Vec3);

/// A height profile along one wall top: fractions of its length with the height of the roof above each.
type Profile<'a> = &'a [(f32, f32)];

/// How much higher the ridge of the snow layer is than the ridge of the roof it lies on.
const SNOW_RISE: f32 = 0.14;

/// What covers a room.
#[derive(Debug, Clone, Copy)]
pub struct Cap {
    /// Color of the roofing.
    pub roof: [f32; 3],
    /// Color of the walls, used for the fill under the roof.
    pub wall: [f32; 3],
    /// True for a hipped roof, false for a gabled one.
    pub hip: bool,
    /// True when snow lies on the roof.
    pub snow: bool,
}

/// Returns the corners of the wall tops of `walls`, in the order front left, front right, back right and back
/// left: each wall top runs from one to the next.
fn ring(walls: Walls) -> [Vec3; 4] {
    let (lo, hi) = walls;
    let corner = |x: f32, z: f32| Vec3::new(x, lo.y, z);
    [
        corner(lo.x, hi.z),
        corner(hi.x, hi.z),
        corner(hi.x, lo.z),
        corner(lo.x, lo.z),
    ]
}

impl Build<'_> {
    /// Adds a pitched roof resting on `walls`, its eaves `over` meters outside them at the height of the wall
    /// tops and its ridge `rise` above along local x, with the fill that carries it.
    pub fn pitched_over(&mut self, walls: Walls, over: f32, rise: f32, cap: &Cap) {
        let wide = Vec3::new(over, 0.0, over);
        let (lo, hi) = (walls.0 - wide, walls.1 + wide);
        let lift = over * rise / ((hi.z - lo.z) * 0.5);
        self.roofing(lo, hi, rise, cap);
        let eave = [(0.0, lift), (1.0, lift)];
        let gable = [(0.0, lift), (0.5, rise), (1.0, lift)];
        let end: Profile = if cap.hip { &eave } else { &gable };
        self.fill(walls, [&eave, end, &eave, end], cap.wall);
    }

    /// Adds the roofing over the box `lo..hi`: hipped or gabled with open ends, under a snow layer when asked.
    /// Only the top layer is solid: the colliders of a roof under snow are those of the snow.
    fn roofing(&mut self, lo: Vec3, hi: Vec3, rise: f32, cap: &Cap) {
        let layers = [(0.0, cap.roof, true), (SNOW_RISE, SNOW, cap.snow)];
        let solid = self.parts.len();
        for (extra, rgb, _) in layers.into_iter().filter(|l| l.2) {
            self.parts.truncate(solid);
            match cap.hip {
                true => self.hip(lo, hi, rise + extra, Mat::deco(rgb)),
                false => self.slopes(lo, hi, rise + extra, Mat::deco(rgb)),
            }
        }
        if cap.snow && !cap.hip {
            self.snow_ends(lo, hi, rise);
        }
    }

    /// Closes the wedge between a gabled roof and the snow layer on it, at both ends.
    fn snow_ends(&mut self, lo: Vec3, hi: Vec3, rise: f32) {
        let mid = (lo.z + hi.z) * 0.5;
        let color = self.color(SNOW, 0.9);
        for (x, out) in [(lo.x, Vec3::NEG_X), (hi.x, Vec3::X)] {
            let ridge = [rise, rise + SNOW_RISE].map(|y| self.pt(Vec3::new(x, lo.y + y, mid)));
            let out = self.dir(out);
            for z in [lo.z, hi.z] {
                let eave = self.pt(Vec3::new(x, lo.y, z));
                self.sheet_tri([eave, ridge[0], ridge[1]], out, [color, color]);
            }
        }
    }

    /// Adds a single slope over the box `roof`, high at its low z and `rise` lower at its high z, resting on
    /// `walls` with the fill that carries it, solid where it rises out of reach of the walls. `coat` is the
    /// roofing and the wall color.
    pub fn lean_over(
        &mut self,
        walls: Walls,
        roof: (Vec3, Vec3),
        rise: f32,
        coat: (Mat, [f32; 3]),
    ) {
        self.slope(roof.0, roof.1, rise, coat.0);
        let above = |z: f32| lean_height(roof.0, roof.1, rise, z) - walls.0.y;
        let (back, front) = (above(walls.0.z), above(walls.1.z));
        let profiles: [Profile; 4] = [
            &[(0.0, front), (1.0, front)],
            &[(0.0, front), (1.0, back)],
            &[(0.0, back), (1.0, back)],
            &[(0.0, back), (1.0, front)],
        ];
        self.fill(walls, profiles, coat.1);
        let ring = ring(walls);
        for (i, profile) in profiles.into_iter().enumerate() {
            self.end_stack(ring[i], ring[(i + 1) % 4], profile);
        }
    }

    /// Adds the fill above the four wall tops of `walls`, in the order front, right, back and left, each up to
    /// its profile.
    fn fill(&mut self, walls: Walls, profiles: [Profile; 4], rgb: [f32; 3]) {
        let ring = ring(walls);
        self.tagged(Tag::Fill, |s| {
            for (i, profile) in profiles.into_iter().enumerate() {
                s.fill_face(ring[i], ring[(i + 1) % 4], profile, rgb);
            }
        });
    }

    /// Adds an upright sheet standing on the wall top from `a` to `b`, as high as `profile` along it. The
    /// outside is on the right hand when walking from `a` to `b`.
    fn fill_face(&mut self, a: Vec3, b: Vec3, profile: Profile, rgb: [f32; 3]) {
        let top = profile.iter().rev();
        let crest = top.map(|(k, y)| a.lerp(b, *k) + Vec3::Y * *y);
        let outline: Vec<Vec3> = [a, b].into_iter().chain(crest).collect();
        let points: Vec<Vec3> = outline.iter().map(|p| self.pt(*p)).collect();
        let out = self.dir((b - a).cross(Vec3::Y));
        let colors = [self.color(rgb, 0.95), self.color(rgb, 0.8)];
        for pair in points[1..].windows(2) {
            self.sheet_tri([points[0], pair[0], pair[1]], out, colors);
        }
    }
}
