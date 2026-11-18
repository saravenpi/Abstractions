//! The rules `starfield.wesl` applies to the light of every star, and `sky.wesl` to the band of
//! the galaxy, written again in Rust so that they can be tested: the shaders run on the GPU,
//! where no test reaches. A test holds the constants of the two sides together.

use bevy::prelude::*;

/// Power of the Doppler factor the brightness of a star follows. Physics has `D^(3-α)` for the
/// light of a point source at one frequency and `D^4` for all of it
/// (https://en.wikipedia.org/wiki/Relativistic_beaming), which at the cap would leave one
/// white blot ahead and nothing behind; this softer power keeps the eye on the stars, and
/// [`BEAM_RANGE`] bounds it.
pub const BEAM: f32 = 1.5;

/// Least and most light a star keeps through the beaming, so the sky behind is dark but not
/// empty and the sky ahead never blows out.
pub const BEAM_RANGE: (f32, f32) = (0.08, 3.0);

/// Doublings of the Doppler factor at which the colour shift is full.
pub const SHIFT_SPAN: f32 = 2.0;

/// Linear tint of a star at the full blue shift and at the full red shift. Both stay near the
/// brightness of white, so the shift reads in black and white as light and dark only.
pub const SHIFTED: (Vec3, Vec3) = (Vec3::new(0.8, 0.92, 1.25), Vec3::new(1.2, 0.8, 0.55));

/// Share of the light of a star spread along its streak: 1 would keep the light of the whole
/// streak that of the point, 0 would make every streak as bright as its star.
pub const SPREAD: f32 = 0.6;

/// Returns the cosine of the angle from the heading at which a point is seen at the speed
/// `beta`, a share of the speed of light, when its true angle has the cosine `cos`.
///
/// At rest it is `cos`. The angle shrinks as the speed rises, and the points straight ahead
/// and straight behind stay where they are. It is `cos θ' = (cos θ + β) / (1 + β cos θ)`
/// (https://en.wikipedia.org/wiki/Relativistic_aberration).
pub fn aberrate(cos: f32, beta: f32) -> f32 {
    ((cos + beta) / (1.0 + beta * cos)).clamp(-1.0, 1.0)
}

/// Returns the share of its light a star keeps at the Doppler factor `factor`.
pub fn beaming(factor: f32) -> f32 {
    factor.max(0.0).powf(BEAM).clamp(BEAM_RANGE.0, BEAM_RANGE.1)
}

/// Returns the linear tint of starlight at the Doppler factor `factor`: white at 1, toward
/// blue above and toward red below.
pub fn tint(factor: f32) -> Vec3 {
    let shift = (factor.max(1.0e-6).log2() / SHIFT_SPAN).clamp(-1.0, 1.0);
    let toward = if shift > 0.0 { SHIFTED.0 } else { SHIFTED.1 };
    Vec3::ONE.lerp(toward, shift.abs())
}

/// Returns the share of its brightness a star keeps when its streak is `half` star widths long
/// on each side: 1 for a point, less as its light spreads along the streak.
pub fn spread(half: f32) -> f32 {
    (1.0 + half.max(0.0) * (4.0 / std::f32::consts::PI)).powf(-SPREAD)
}

/// The shader holds the same constants as the rules tested here.
#[test]
fn the_shader_has_the_constants_of_the_rules() {
    let (shader, sky) = (
        include_str!("../starfield.wesl"),
        include_str!("../sky.wesl"),
    );
    let triple = |v: Vec3| format!("vec3<f32>({:?}, {:?}, {:?})", v.x, v.y, v.z);
    let lines = [
        format!("const BEAM: f32 = {BEAM:?};"),
        format!("const BEAM_LEAST: f32 = {:?};", BEAM_RANGE.0),
        format!("const BEAM_MOST: f32 = {:?};", BEAM_RANGE.1),
        format!("const SHIFT_SPAN: f32 = {SHIFT_SPAN:?};"),
        format!("const BLUE: vec3<f32> = {};", triple(SHIFTED.0)),
        format!("const RED: vec3<f32> = {};", triple(SHIFTED.1)),
        format!("const SPREAD: f32 = {SPREAD:?};"),
    ];
    for line in &lines {
        assert!(shader.contains(line), "starfield.wesl lacks `{line}`");
    }
    for line in &lines[..3] {
        assert!(sky.contains(line), "sky.wesl lacks `{line}`");
    }
}
