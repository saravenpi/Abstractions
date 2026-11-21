//! The title screen, the loading screen and the first frames of a new world.

use abstractions::settings::InterfaceSize;

use super::*;

/// The title screen, then a new world.
pub fn title(world: &mut World, _: &mut Notes, frame: u32) -> Act {
    match frame {
        1 => {
            let size = match std::env::var("PROBE_UI_SIZE").as_deref() {
                Ok("small") => Some(InterfaceSize::Small),
                Ok("large") => Some(InterfaceSize::Large),
                _ => None,
            };
            if let Some(size) = size {
                world
                    .resource_mut::<Persistent<Settings>>()
                    .gameplay
                    .interface = size;
            }
            Act::Wait
        }
        20 => {
            shot(world, "00-title");
            Act::Wait
        }
        30 if std::env::var("PROBE_JOIN").is_ok() => join(world),
        30 => {
            let cards = std::env::var("PROBE_STAGES").is_ok_and(|l| l.contains("postcards"));
            let seed = std::env::var("SMOKE_SEED")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(if cards { 18 } else { 7 });
            let dir = world.resource::<SaveDir>().0.clone();
            let entry = new_entry(&dir, "Smoke world", seed);
            let started =
                world.run_system_once(move |mut starter: Starter| starter.start(entry.clone()));
            match started {
                Ok(()) => Act::Next,
                Err(e) => Act::Fail(format!("{e:?}")),
            }
        }
        _ => Act::Wait,
    }
}

/// Joins the server `PROBE_JOIN` names, the way the multiplayer menu does: a `JoinServer` message
/// for the driver, with the password of `PROBE_PASSWORD` and the player name of `PROBE_NAME`.
fn join(world: &mut World) -> Act {
    let address = std::env::var("PROBE_JOIN").unwrap_or_default();
    if let Ok(name) = std::env::var("PROBE_NAME") {
        world.resource_mut::<Persistent<Settings>>().player_name = name;
    }
    let password = std::env::var("PROBE_PASSWORD").ok();
    world.write_message(JoinServer {
        address,
        expected_key: None,
        password,
    });
    Act::Next
}

/// The loading screen, until the world is played.
pub fn loading(world: &mut World, _: &mut Notes, frame: u32) -> Act {
    let screen = *world.resource::<State<Screen>>().get();
    match (screen, frame) {
        (Screen::Loading, 5) => Act::Shoot("01-loading".into()),
        (Screen::Gameplay, _) => Act::Next,
        (_, f) if f > 6000 => Act::Fail("the world never loaded".into()),
        _ => Act::Wait,
    }
}

/// The boot screen, until it lifts.
pub fn boot(world: &mut World, _: &mut Notes, frame: u32) -> Act {
    let warm = *world.resource::<State<WarmUp>>().get();
    if std::env::var("SMOKE_FLASH").is_ok() && frame.is_multiple_of(3) {
        shot(world, &format!("f-boot-{frame:03}"));
    }
    match (warm, frame) {
        (WarmUp::Hidden, 30) => {
            shot(world, "02-boot");
            Act::Wait
        }
        (WarmUp::Idle, f) if f > 40 => Act::Next,
        (_, f) if f > 3000 => Act::Fail("the boot screen never lifted".into()),
        _ => Act::Wait,
    }
}

/// The view the player wakes with.
pub fn wake(world: &mut World, _: &mut Notes, frame: u32) -> Act {
    if std::env::var("SMOKE_FLASH").is_ok() && frame < 40 && frame.is_multiple_of(2) {
        shot(world, &format!("f-wake-{frame:03}"));
    }
    match frame {
        60 => {
            hygiene(world, "home base, wake view");
            Act::Shoot("03-wake".into())
        }
        _ => Act::Wait,
    }
}
