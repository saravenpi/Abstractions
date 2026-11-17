//! The modes of the flight gear.

/// What the gear is doing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Mode {
    /// The gear does nothing: the controller walks, jumps, falls and swims as usual.
    #[default]
    Off,
    /// Boots alone: they brake the fall and give a little drift.
    BootFall,
    /// Wingsuit alone: an unpowered glide.
    Glide,
    /// Wingsuit and boots: powered flight.
    Fly,
    /// Flight stopped in the air: the velocity carries on ballistically.
    Fall,
    /// Touched down shallow and fast: sliding along the ground into a run.
    Slide,
    /// Touched down steep: a soft landing roll that sheds speed.
    Roll,
}

impl Mode {
    /// Returns true while the gear owns the velocity of the body.
    pub fn drives(self) -> bool {
        self != Mode::Off
    }

    /// Returns true while the body is carried by the air and not by the ground.
    pub fn airborne(self) -> bool {
        matches!(self, Mode::BootFall | Mode::Glide | Mode::Fly | Mode::Fall)
    }

    /// Returns true while the wingsuit is meant to be open.
    pub fn winged(self) -> bool {
        matches!(self, Mode::Glide | Mode::Fly)
    }
}
