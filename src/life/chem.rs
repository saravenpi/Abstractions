//! Biochemistry: one roll per living planet.

/// Returns the linear RGB colour of the light gathering pigment of producers under a star at `kelvin`:
/// green under a yellow star, red to black under a red dwarf, pale under a hot blue star.
///
/// Phase 0 stub: the green of home.
pub fn pigment(kelvin: f32) -> [f32; 3] {
    let _ = kelvin;
    [0.2, 0.5, 0.15]
}
