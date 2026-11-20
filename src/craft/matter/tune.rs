//! Tuning constants of matter: thresholds and shares no measurement pins down yet.

/// Share of strength and toughness a tissue keeps per kill grade (clean, good, poor), unitless.
pub const GRADE: [f32; 3] = [1.0, 0.85, 0.6];

/// Starlight a hide needs to sun-dry, the home planet receiving 1, unitless.
pub const SUN_FLUX: f64 = 0.5;

/// Mean ground temperature in kelvin above which the ground chars tissue like a fire: wood starts
/// to char near 570 K.
pub const FIRE_KELVIN: f64 = 600.0;

/// Mean ground temperature in kelvin under which a tissue freezes through and sets.
pub const FROST_KELVIN: f64 = 250.0;

/// Stiffness factor of freeze-setting, unitless: ice in the tissue stiffens it.
pub const FREEZE_STIFFNESS: f32 = 1.5;

/// Toughness factor of freeze-setting, unitless: frozen tissue cracks more easily.
pub const FREEZE_TOUGHNESS: f32 = 0.7;
