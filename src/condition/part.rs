//! The parts of the robot that carry health.

/// Number of parts that carry health.
pub const PARTS: usize = 10;

/// A part of the robot that carries health of its own.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Part {
    /// The head with its screen.
    Head,
    /// The torso and the hips.
    Torso,
    /// The left arm.
    ArmL,
    /// The right arm.
    ArmR,
    /// The left leg.
    LegL,
    /// The right leg.
    LegR,
    /// The left wing of the wingsuit on the back.
    WingL,
    /// The right wing of the wingsuit on the back.
    WingR,
    /// The left boot with its booster.
    BootL,
    /// The right boot with its booster.
    BootR,
}

impl Part {
    /// Every part, in declaration order.
    pub const ALL: [Part; PARTS] = [
        Part::Head,
        Part::Torso,
        Part::ArmL,
        Part::ArmR,
        Part::LegL,
        Part::LegR,
        Part::WingL,
        Part::WingR,
        Part::BootL,
        Part::BootR,
    ];

    /// Returns the name the HUD writes.
    pub fn name(self) -> &'static str {
        const NAMES: [&str; PARTS] = [
            "Head",
            "Torso",
            "Left arm",
            "Right arm",
            "Left leg",
            "Right leg",
            "Left wing",
            "Right wing",
            "Left boot",
            "Right boot",
        ];
        NAMES[self as usize]
    }

    /// Returns true for a part that moves on a joint, which a field repair binds: a limb or a wing.
    /// The others are plates, which a field repair patches with something hard.
    pub fn joint(self) -> bool {
        !matches!(self, Part::Head | Part::Torso | Part::BootL | Part::BootR)
    }
}
