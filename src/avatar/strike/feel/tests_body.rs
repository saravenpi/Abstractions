//! Tests of the body behind a strike: the weight goes back in the wind-up and forward through the
//! blow, a heavy tool moves more of the body than a light one, the free arm swings against the tool,
//! the coil runs on without a jump from act to act, and walking keeps the legs to the gait.

use super::super::super::pose::{Core, Limb, Pose};
use super::super::super::{AvatarPose, Mode};
use super::super::act::{Act, Timing};
use super::super::dir::Dir;
use super::super::layer::Layer;
use super::super::path::Path;
use super::super::stage::{club, knife};
use super::super::tune::COAST;
use super::bend::Bend;
use super::body::{counter, phase, stance, still};
use super::tune::{COUNTER, STEP};
use crate::craft::build::Hands;

/// Returns the heft of a tool of `inertia` kg·m², as the body reads it.
fn heft(inertia: f32) -> f32 {
    inertia / (inertia + COAST)
}

/// Cocked, the weight sits over the back foot and the body leans back; followed through, it is over
/// the front foot, leaning into the blow with the front foot stepped in.
#[test]
fn the_weight_goes_back_then_into_the_blow() {
    let level = Bend::default();
    let cocked = stance(-1.0, &level, (0.5, STEP));
    let through = stance(1.0, &level, (0.5, STEP));
    assert!(cocked.sway > 0.0 && through.sway < 0.0);
    assert!(cocked.lean < 0.0 && through.lean > 0.0);
    assert!(through.feet[0].ahead > stance(0.0, &level, (0.5, STEP)).feet[0].ahead);
    assert!(through.feet[0].ahead > 0.0 && through.feet[1].ahead < 0.0);
    assert!(through.lift < cocked.lift);
}

/// A club of great inertia is swung from the legs: it drops and leans the body further than a
/// knife does.
#[test]
fn a_heavy_tool_moves_more_of_the_body() {
    let (light, heavy) = (heft(knife().swing.inertia), heft(club().swing.inertia));
    assert!(heavy > light, "{light} {heavy}");
    let level = Bend::default();
    let (a, b) = (
        stance(1.0, &level, (light, STEP)),
        stance(1.0, &level, (heavy, STEP)),
    );
    assert!(b.lean > a.lean && b.lift < a.lift);
}

/// The free arm reaches ahead while the tool is cocked and tucks back as it follows through.
#[test]
fn the_free_arm_balances_the_blow() {
    assert!(counter(-1.0)[0] > COUNTER[1][0]);
    assert!(counter(1.0)[0] < COUNTER[1][0]);
    assert_eq!(counter(0.0), COUNTER[1]);
}

/// The coil runs from cocked to followed through with no jump between the wind-up, the swing and
/// the recovery, and the strike is fully under way through the swing.
#[test]
fn the_coil_runs_on_from_act_to_act() {
    let held = club();
    let timing = Timing::of(&held, &Path::of(held.style, Dir::Up));
    let end_of_windup = phase(Act::Windup(timing.windup), &timing);
    let start_of_swing = phase(Act::Swing(0.0), &timing);
    assert!((end_of_windup.1 - start_of_swing.1).abs() < 1e-5);
    let end_of_swing = phase(Act::Swing(timing.swing), &timing);
    let start_of_recovery = phase(Act::Recover(0.0), &timing);
    assert!((end_of_swing.1 - start_of_recovery.1).abs() < 1e-5);
    assert_eq!(start_of_swing.0, 1.0);
    assert_eq!(phase(Act::Ready, &timing), (0.0, 0.0));
}

/// A standing body takes the stance and drops into the blow; a body that walks fast keeps its legs
/// to the gait, and the free arm swings only for a tool of one hand.
#[test]
fn the_legs_take_the_stance_only_standing() {
    let standing = AvatarPose::default();
    let running = AvatarPose {
        mode: Mode::Run,
        speed: 5.0,
        ..AvatarPose::default()
    };
    assert_eq!(still(&standing), 1.0);
    assert_eq!(still(&running), 0.0);
    let key = Path::of(super::super::held::Style::Slash, Dir::Right).at(1.0);
    let bend = Bend::of(None, 1.0);
    let base = Pose::default();
    let one = Layer::of(&key, Hands::One).with_body(
        (1.0, 1.0, true),
        &Bend { still: 1.0, ..bend },
        (0.5, STEP),
    );
    let posed = one.apply(base);
    assert!(posed.core(Core::Lift) < base.core(Core::Lift));
    assert_ne!(posed.limb(Limb::Knee, 0), base.limb(Limb::Knee, 0));
    assert_ne!(
        posed.sides[0][Limb::Shoulder as usize],
        base.sides[0][Limb::Shoulder as usize]
    );
    let walking = Layer::of(&key, Hands::One).with_body((1.0, 1.0, true), &bend, (0.5, STEP));
    assert_eq!(walking.apply(base).core(Core::Lift), base.core(Core::Lift));
    let two = Layer::of(&key, Hands::Two).with_body((1.0, 1.0, true), &bend, (0.5, STEP));
    assert!(two.free.is_none());
}
