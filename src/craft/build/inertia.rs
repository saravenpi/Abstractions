//! Mass properties: a tool as point masses on the axis through the hand.

/// One part as the physics sees it: a mass at a distance from the hand, with its own spread.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Piece {
    /// Mass in kilograms.
    pub mass: f32,
    /// Distance of its centre from the hand along the axis, in metres; negative behind the hand.
    pub at: f32,
    /// Radius of gyration about its own centre, in metres.
    pub gyration: f32,
}

impl Piece {
    /// Returns a uniform rod of `mass` running from `from` to `to` metres along the axis: its centre
    /// halfway, and a radius of gyration of length / √12.
    pub fn rod(mass: f32, from: f32, to: f32) -> Self {
        Self {
            mass,
            at: (from + to) / 2.0,
            gyration: (to - from).abs() / 12f32.sqrt(),
        }
    }
}

/// Returns the moment of inertia about the hand, I = Σ m(r² + k²), in kg·m² (parallel axis theorem).
pub fn inertia(pieces: &[Piece]) -> f32 {
    pieces
        .iter()
        .map(|p| p.mass * (p.at * p.at + p.gyration * p.gyration))
        .sum()
}

/// Returns the first moment Σ m·r about the hand, in kg·m: times gravity, the torque to hold it level.
pub fn moment(pieces: &[Piece]) -> f32 {
    pieces.iter().map(|p| p.mass * p.at).sum()
}

/// Returns the total mass in kilograms.
pub fn mass(pieces: &[Piece]) -> f32 {
    pieces.iter().map(|p| p.mass).sum()
}

/// Returns the balance point, the centre of mass, in metres from the hand.
pub fn balance(pieces: &[Piece]) -> f32 {
    moment(pieces) / mass(pieces).max(f32::EPSILON)
}
