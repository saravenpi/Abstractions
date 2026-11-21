//! The robot hurt and mended: two blows that break an arm and hurt a leg, the body and the
//! frame rows of the HUD that show it, the mend key in the field, and the bay of the base that
//! makes the body whole: shots `e60` to `e63`.
//!
//! The blows are the message an attack of an animal sends when it lands, `avatar::blow::Blow`,
//! written here with a set energy, so the shots do not depend on what a fight happened to do.

use abstractions::avatar::blow::Blow;
use abstractions::condition::Condition;

use super::*;

/// The name of the stage in the log.
const STAGE: &str = "frame";

/// Sends a cutting blow of `joules` on the robot, `side` metres to its right and `rise` metres
/// above its middle, pushing it to its right.
fn strike(world: &mut World, joules: f32, (side, rise): (f32, f32)) {
    let placed = world
        .query_filtered::<&Transform, With<Player>>()
        .iter(world)
        .next()
        .copied();
    let Some(body) = placed else {
        return;
    };
    let (right, up) = (body.right().as_vec3(), body.up().as_vec3());
    world.write_message(Blow {
        at: body.translation + right * side + up * rise,
        joules,
        cut: true,
        dir: right,
        momentum: 60.0,
    });
}

/// Turns the camera around the robot to see it from the front and the left.
fn front(world: &mut World) {
    let heading = *world.resource::<Heading>();
    world.write_message(Cut {
        heading: Some(Heading {
            yaw: heading.yaw + 2.5,
            pitch: -0.15,
        }),
    });
}

/// The blows and the shots of the hurt robot.
fn hurt_steps(world: &mut World, play: &mut Play, frame: u32) {
    let held = play.held(frame);
    let mend = bound(world, |s| s.controls.keys.mend);
    match held {
        2 => {
            view(world, ViewMode::Third, ViewMode::First);
            say(STAGE, &format!("before the blows: {}", frame_words(world)));
        }
        20 => strike(world, 520.0, (-0.3, 0.35)),
        26 => {
            say(
                STAGE,
                &format!(
                    "e60 a cutting blow of 520 J at the left arm: {}",
                    frame_words(world)
                ),
            );
            shot(world, "e60-robot-struck");
        }
        60 => strike(world, 450.0, (0.12, -0.5)),
        70 => front(world),
        130 => {
            say(
                STAGE,
                &format!(
                    "e61 after a second blow of 450 J at the right leg: {}",
                    frame_words(world)
                ),
            );
            shot(world, "e61-broken-part");
        }
        131 | 132 => tap(world, mend, held, 131),
        150 => {
            say(
                STAGE,
                &format!(
                    "e62 after the mend key: {}; {}",
                    frame_words(world),
                    pack_words(world)
                ),
            );
            shot(world, "e62-field-mend");
            play.next(frame);
        }
        _ => {}
    }
}

/// The robot stands in the bay of the base until every part is whole again.
fn bay_steps(world: &mut World, play: &mut Play, frame: u32) -> Option<Act> {
    let held = play.held(frame);
    if held == 1 {
        let Some(base) = world.get_resource::<Base>().copied() else {
            return Some(gave_up(
                world,
                STAGE,
                "this world has no home base to mend at",
            ));
        };
        let feet = ground(world, base.centre + base.east * 6.0, 0.0);
        stand(world, feet, base.pad, -0.1);
    }
    let whole = world.resource::<Condition>().intact();
    if held < 40 || (!whole && held <= 500) {
        return None;
    }
    say(
        STAGE,
        &format!(
            "e63 {held} frames in the bay of the base, whole: {whole}: {}",
            frame_words(world)
        ),
    );
    shot(world, "e63-mended-at-base");
    Some(Act::Next)
}

/// The robot hurt by two blows, its damage on the body and on the HUD, the mend key, and the
/// bay of the base: shots `e60` to `e63`.
pub fn play_frame(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    if frame == 1 {
        notes.play.start(frame);
    }
    let play = &mut notes.play;
    if play.step == 0 {
        hurt_steps(world, play, frame);
        return Act::Wait;
    }
    bay_steps(world, play, frame).unwrap_or(Act::Wait)
}
