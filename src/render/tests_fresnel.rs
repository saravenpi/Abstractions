//! Tests of the reflection of the sea: Schlick's Fresnel term, redone here on the CPU, is about
//! 2 percent for water seen head on, and all the light at a grazing angle.

/// The shader of the sea.
const SHADER: &str = include_str!("../world/sea/water.wesl");

/// Reflectance of water seen head on: from its index of refraction of 1.33,
/// `((1.33 - 1) / (1.33 + 1))^2`.
const WATER_F0: f32 = 0.02;

/// Returns the share of the light water reflects when the view makes the cosine `cos` with the
/// normal, by Schlick's approximation, as the fragment shader of the sea writes it.
fn schlick(cos: f32) -> f32 {
    WATER_F0 + (1.0 - WATER_F0) * (1.0 - cos.clamp(0.0, 1.0)).powi(5)
}

/// The index of refraction of water gives about 2 percent, and the shader holds the same number.
#[test]
fn water_reflects_two_percent_head_on() {
    let index = 1.33_f32;
    let exact = ((index - 1.0) / (index + 1.0)).powi(2);
    assert!((exact - WATER_F0).abs() < 0.001, "{exact}");
    assert!((schlick(1.0) - 0.02).abs() < 1.0e-6);
    assert!(SHADER.contains("const WATER_F0: f32 = 0.02;"));
}

/// At a grazing angle water is a mirror, and the term rises as the view lowers.
#[test]
fn water_is_a_mirror_at_a_grazing_angle() {
    assert!((schlick(0.0) - 1.0).abs() < 1.0e-6);
    let rising = [1.0, 0.8, 0.5, 0.2, 0.05, 0.0].map(schlick);
    assert!(rising.windows(2).all(|pair| pair[1] > pair[0]));
    assert!(schlick(0.2) < 0.45 && schlick(0.05) > 0.7);
    let line = "let fresnel = (WATER_F0 + (1.0 - WATER_F0) * pow(1.0 - ndv, 5.0)) * water.deep.w;";
    assert!(SHADER.contains(line));
}
