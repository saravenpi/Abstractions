//! The pack and the workshop on a staged pack, the rest of the HUD hidden under the screen: the
//! empty pack, a pack of stones and a hide, each intent with its proposals, a card focused with its
//! parts lit, and a tool made going into the hotbar: shots `f0` to `f7`.

use abstractions::craft::Pack;
use abstractions::craft::matter::{Class, Grade, Mineral, Stuff, Treatments};
use abstractions::craft::pack::Stack;
use abstractions::craft::ui::{Bench, Workshop};

use super::*;

/// The name of the stage in the log.
const STAGE: &str = "workshop";

/// Returns `mass` kilograms of `mineral`.
fn stone(mineral: Mineral, mass: f32) -> Stack {
    Stack {
        stuff: Stuff::Mineral(mineral),
        material: mineral.material(),
        mass,
    }
}

/// Returns `mass` kilograms of a clean tissue of `class` from the first species of the planet of
/// terrain seed `planet`, so the pack names it after that species.
fn tissue(planet: u64, class: Class, mass: f32) -> Stack {
    Stack {
        stuff: Stuff::Tissue {
            planet,
            species: 0,
            tissue: 0,
            class,
            grade: Grade::Clean,
            treated: Treatments::default(),
        },
        material: class.anchor(),
        mass,
    }
}

/// Puts `stacks` in the pack.
fn stow(world: &mut World, stacks: &[Stack]) {
    let mut pack = world.resource_mut::<Pack>();
    for stack in stacks {
        pack.stow(*stack, f32::INFINITY);
    }
}

/// Says what the screen shows and shoots it as `name`.
fn shoot(world: &mut World, name: &str) {
    say(
        STAGE,
        &format!("{name}: {}; {}", pack_words(world), offer_words(world)),
    );
    shot(world, name);
}

/// Empties the pack, marked kitted so the starting kit does not fill it again, and the bench, and
/// opens the screen on `pack_key`.
fn start(world: &mut World, pack_key: KeyCode, frame: u32) {
    if frame == 1 {
        tidy(world);
        view(world, ViewMode::First, ViewMode::First);
        *world.resource_mut::<Pack>() = Pack {
            kitted: true,
            ..Pack::default()
        };
        *world.resource_mut::<Bench>() = Bench::default();
    }
    tap(world, pack_key, frame, 10);
}

/// Each intent in turn with the Right arrow, from Cut: shots `f2` to `f5`.
fn intents(world: &mut World, frame: u32) {
    let names = [
        "f2-workshop-cut",
        "f3-workshop-pierce",
        "f4-workshop-crush",
        "f5-workshop-throw",
    ];
    let at = ((frame - 120) / 30) as usize;
    if (frame - 120) % 30 == 0 {
        shoot(world, names[at]);
    }
    if at < 3 {
        tap(world, KeyCode::ArrowRight, frame, 121 + 30 * at as u32);
    }
}

/// The staged pack on the screen, step by step: empty, stones and a hide, then bone and wood for
/// shafts, each intent, the most durable card focused, and a tool made.
pub fn play_workshop(world: &mut World, _: &mut Notes, frame: u32) -> Act {
    let pack_key = bound(world, |s| s.controls.keys.pack);
    start(world, pack_key, frame);
    let open = *world.resource::<State<Workshop>>().get() == Workshop::Open;
    let planet = world.resource::<Terrain>().seed();
    match frame {
        50 if !open => return gave_up(world, STAGE, "the pack key did not open the pack"),
        50 => shoot(world, "f0-pack-empty"),
        55 => stow(
            world,
            &[
                stone(Mineral::Flint, 1.5),
                stone(Mineral::Basalt, 2.0),
                tissue(planet, Class::Hide, 1.0),
            ],
        ),
        80 => shoot(world, "f1-pack-stones-hide"),
        85 => stow(
            world,
            &[
                tissue(planet, Class::Bone, 2.0),
                tissue(planet, Class::Wood, 3.0),
            ],
        ),
        90..=91 => tap(world, KeyCode::Tab, frame, 90),
        120..=210 => intents(world, frame),
        211..=216 => [211, 214]
            .into_iter()
            .for_each(|at| tap(world, KeyCode::ArrowDown, frame, at)),
        240 => shoot(world, "f6-card-focused"),
        241..=242 => tap(world, KeyCode::Enter, frame, 241),
        270 => shoot(world, "f7-made"),
        271..=272 => tap(world, pack_key, frame, 271),
        300 => return Act::Next,
        _ => {}
    }
    Act::Wait
}
