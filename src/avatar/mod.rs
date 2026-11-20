//! The robot body the player controls: a humanoid with a screen face.
//!
//! The body is a child of the entity that carries [`crate::player::Player`], so it turns with the
//! stored heading, stays upright along the local up and follows every rebase for free. Its only
//! input is [`AvatarPose`]: one system fills what the player controller already tells
//! ([`AvatarSet::Fill`]), anything ordered after it may overwrite the result, and
//! [`AvatarSet::Animate`] turns it into joint angles. [`Shown`] chooses how much of the body is drawn.

mod air;
mod animate;
mod blend;
pub mod blow;
mod clock;
mod crouch;
mod dress;
mod dust;
mod face;
mod fill;
mod flight;
mod fx;
mod gait;
mod ground;
pub mod hurt;
mod kick;
mod leap;
mod looks;
mod marks;
mod mesh;
mod neck;
mod own;
mod pose;
mod puffs;
mod rig;
mod seated;
mod show;
mod spawn;
pub mod strike;
mod touch;
pub mod tune;
mod warm;
mod water;
mod wing;

#[cfg(test)]
mod tests;

pub use looks::Looks;
pub use marks::{Marked, Marks};
pub use own::Own;
pub use show::Shown;
pub use spawn::{Stock, puppet};

use bevy::prelude::*;

use crate::player::PlayerSet;
use crate::render::Warm;
use crate::screens::shown;
use crate::view::ViewSet;

/// What the body is doing, which chooses its pose.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Mode {
    /// Standing still.
    #[default]
    Idle,
    /// Walking.
    Walk,
    /// Sprinting.
    Run,
    /// Rising after a jump.
    Jump,
    /// Falling with no gear in use.
    Fall,
    /// Touching down. Hold it for a moment: the knees bend while it lasts and recover after.
    Land,
    /// Swimming at the surface.
    Swim,
    /// Swimming under water.
    Dive,
    /// Seated in the ship.
    Seated,
    /// Falling upright on the boot boosters.
    BootFall,
    /// Gliding prone in the wingsuit.
    Glide,
    /// Powered flight: wingsuit and boots together.
    Fly,
    /// Rolling over after a fast touchdown, or tumbling after a crash: the trunk turns whole turns.
    Roll,
    /// Skidding along the ground after a fast shallow touchdown.
    Skid,
}

impl Mode {
    /// Every mode, in declaration order.
    pub const ALL: [Mode; 14] = [
        Mode::Idle,
        Mode::Walk,
        Mode::Run,
        Mode::Jump,
        Mode::Fall,
        Mode::Land,
        Mode::Swim,
        Mode::Dive,
        Mode::Seated,
        Mode::BootFall,
        Mode::Glide,
        Mode::Fly,
        Mode::Roll,
        Mode::Skid,
    ];

    /// Returns whether the mode turns the whole body over or folds it: water, wingsuit, seat,
    /// roll and skid.
    /// A change into or out of such a mode takes the longer blend time.
    pub fn sweeping(self) -> bool {
        self.winged() || self.wet() || self.floored() || self == Mode::Seated
    }

    /// Returns whether the mode lays the whole body on the ground, not only the feet.
    pub fn floored(self) -> bool {
        matches!(self, Mode::Roll | Mode::Skid)
    }

    /// Returns whether the mode lies prone on the wings and turns with the flight path, all the way round
    /// in a loop.
    pub fn winged(self) -> bool {
        matches!(self, Mode::Glide | Mode::Fly)
    }

    /// Returns whether the mode swims and lies along its way through the water.
    pub fn wet(self) -> bool {
        matches!(self, Mode::Swim | Mode::Dive)
    }
}

/// Everything a body needs to pose itself, on its root. Angles are radians, speeds meters per
/// second. The filler and the dresser write the one of the player at this machine; whatever
/// writes the one of another body makes that body move the same way.
#[derive(Component, Debug, Clone, Copy, PartialEq, Default)]
pub struct AvatarPose {
    /// What the body is doing.
    pub mode: Mode,
    /// Horizontal speed.
    pub speed: f32,
    /// Speed along the local up, negative when falling.
    pub vertical_speed: f32,
    /// Distance walked so far in meters, ever growing. The leg cycle follows it, so feet do not slide.
    pub stride_distance: f32,
    /// Bank into a turn while flying: positive rolls the right side down.
    pub bank: f32,
    /// Pitch of the flight path: positive climbs.
    pub pitch: f32,
    /// Thrust of the boot boosters, from 0 to 1.
    pub thrust: f32,
    /// How far the wingsuit is asked to deploy, from 0 (folded) to 1.
    pub wings: f32,
    /// Speed through the air, which makes the fabric flutter.
    pub airspeed: f32,
    /// Where the head looks relative to the body: `x` is the yaw to the left, `y` the pitch up.
    pub look: Vec2,
    /// How hard the last landing was, from 0 (soft) to 1 (hard). Read when the mode turns to `Land`.
    pub landed_hard: f32,
    /// Speed along the local up the body lost in its last touch-down, in meters per second.
    pub impact: f32,
    /// Velocity the body had when the gear last hit a surface, in meters per second in the frame
    /// of the body: +X right, +Y up, -Z ahead.
    pub struck: Vec3,
    /// Speed the body lost into that surface, in meters per second.
    pub lost: f32,
    /// True for a moment after any touch-down, at a run too: the knees take the landing in.
    pub absorb: bool,
    /// How fast the ground speed grows, eased, in meters per second squared. Negative brakes.
    pub gain: f32,
    /// How fast the heading turns to the left on the ground, eased, in radians per second.
    pub turn: f32,
    /// Free height under the body down to the ground, in meters. Zero on the ground.
    pub clearance: f32,
    /// How prone the flight gear lays the body, from 0 (upright) to 1 (flat along its nose).
    pub prone: f32,
    /// How far the body is crouched, from 0 (standing) to 1 (fully down).
    pub crouch: f32,
    /// Felt load in g while the gear carries the body: 1 in steady flight, more in a pull-up.
    pub load: f32,
}

/// The two stages of the body, in order, in `Update`.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AvatarSet {
    /// [`AvatarPose`] is filled from the player controller. Order after it to overwrite fields.
    Fill,
    /// The body is spawned, posed, painted and shown or hidden.
    Animate,
}

/// Marker for the root of the body, a child of the player entity. There is only ever one.
#[derive(Component, Debug, Default, Clone, Copy)]
pub struct Avatar;

/// Adds the resources and the systems of the body, with no run condition.
fn systems(app: &mut App) {
    let rendered = resource_exists::<Warm>.and_then(resource_exists::<Assets<Image>>);
    app.add_systems(Startup, warm::warm.run_if(rendered))
        .add_message::<blow::Blow>()
        .init_resource::<Shown>()
        .init_resource::<Marks>()
        .configure_sets(Update, AvatarSet::Animate.after(AvatarSet::Fill))
        .add_systems(Update, fill::fill.in_set(AvatarSet::Fill))
        .add_systems(
            Update,
            (
                (spawn::spawn, puffs::spawn),
                animate::animate,
                marks::mark,
                (kick::kick, kick::fly).chain().run_if(puffs::ready),
                fx::wings,
                fx::boots,
                fx::face,
                show::show,
            )
                .chain()
                .in_set(AvatarSet::Animate),
        );
    seated::systems(app);
    strike::plugin(app);
    hurt::plugin(app);
}

/// Registers the avatar: its input, its visibility setting and the systems that run it while
/// the world is shown, behind the boot screen too. The pose is filled once the controller moved
/// the body, then dressed with the gear and the view once the camera rig ran.
pub(super) fn plugin(app: &mut App) {
    systems(app);
    app.add_message::<blow::Blow>();
    app.configure_sets(Update, (AvatarSet::Fill, AvatarSet::Animate).run_if(shown))
        .configure_sets(Update, AvatarSet::Fill.after(PlayerSet::Move))
        .add_systems(
            Update,
            dress::wear
                .after(AvatarSet::Fill)
                .after(ViewSet::Rig)
                .before(AvatarSet::Animate)
                .run_if(shown),
        );
}
