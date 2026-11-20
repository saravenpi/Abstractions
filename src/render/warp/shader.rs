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

/// Most half length in radians of the streak of a star, however near it passes.
pub const STREAK_MOST: f32 = 0.25;

/// Distance in game light-years up to which a star is drawn, and the last part of it over
/// which a star fades in as the camera closes on it. `starfield.rs` gathers farther than that.
pub const SIGHT: (f32, f32) = (40.0, 2.5);

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
/// on each side: 1 for a point, and less by exactly the area its light is spread over, a disc
/// drawn out into a stadium. The whole light of a streak is the light of its star.
pub fn spread(half: f32) -> f32 {
    1.0 / (1.0 + half.max(0.0) * (4.0 / std::f32::consts::PI))
}

/// Returns the half length in radians of the streak of a star `range` game light-years away
/// whose true bearing from the heading has the sine `sin`, seen at the Doppler factor `factor`,
/// while the ship flies `travel` game light-years during one exposure.
///
/// It is the real apparent motion of the star: its angular speed is the speed of the ship
/// across the line of sight over its distance, `v sin θ / r`. Through the aberration the seen
/// angle moves `1 / D` as fast as the true one. So a near star abeam streaks long, a far one
/// stays a point, and a star dead ahead or dead astern does not streak at all. The streak runs
/// along the great circle through the heading, which is where the motion of the ship moves it.
pub fn streak(travel: f32, sin: f32, range: f32, factor: f32) -> f32 {
    (0.5 * travel * sin / (range * factor)).min(STREAK_MOST)
}

/// Light that arrives from a star, as the star field counts it, from which the star shows
/// through the daylight of a sky, fully one step of ten above it: the star of the system the
/// camera is in, which is the sun of that sky, and no other.
pub const DAY_FLUX: f32 = 1.0;

/// Returns how far a star from which the light `flux` arrives shows through a sky that hides
/// the stars, from 0 to 1: the sun is the same point of the star field as from afar, and the
/// day does not put it out.
pub fn daylit(flux: f32) -> f32 {
    let t = (flux - DAY_FLUX).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Returns the share of its light a star `range` game light-years from the camera keeps: all
/// of it up close, fading to nothing at the edge of sight, so that a star the camera closes on
/// comes into the sky from nothing instead of popping in with the next mesh.
pub fn sight(range: f32) -> f32 {
    let t = ((range - (SIGHT.0 - SIGHT.1)) / SIGHT.1).clamp(0.0, 1.0);
    1.0 - t * t * (3.0 - 2.0 * t)
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
        format!("const STREAK_MOST: f32 = {STREAK_MOST:?};"),
        format!("const SIGHT: f32 = {:?};", SIGHT.0),
        format!("const SIGHT_FADE: f32 = {:?};", SIGHT.1),
        format!("const DAY_FLUX: f32 = {DAY_FLUX:?};"),
        "let day = smoothstep(DAY_FLUX, DAY_FLUX + 1.0, flux);".to_string(),
        format!("const EXPOSURE: f32 = {:?};", super::look::EXPOSURE),
        "return tint(factor) * (beam / (1.0 + half * FOUR_OVER_PI));".to_string(),
        "min(0.5 * stars.params.y * seen.turn / (now * seen.factor), STREAK_MOST);".to_string(),
        "let near = 1.0 - smoothstep(SIGHT - SIGHT_FADE, SIGHT, now);".to_string(),
    ];
    for line in &lines {
        assert!(shader.contains(line), "starfield.wesl lacks `{line}`");
    }
    for line in &lines[..3] {
        assert!(sky.contains(line), "sky.wesl lacks `{line}`");
    }
}
