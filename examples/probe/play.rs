//! The `play` stages show the game being played: the map of explored ground, stones and tissue in
//! the pack, a herd stalked, a fight, a kill carved, the field guide. This file holds what they
//! share: the state a stage keeps between frames, the mouse, the facts they log and the way a
//! stage gives up.
//!
//! Every stage logs what it shows as `PROBE play <stage>: <facts>` and never hangs: past its
//! frame limit it logs `PROBE play <stage>: gave up because <why>` and the tour moves on.

use abstractions::avatar::strike::Strike;
use abstractions::computer::Computer;
use abstractions::condition::{Condition, Part};
use abstractions::craft::pack::called;
use abstractions::craft::ui::Workshop;
use abstractions::craft::{Burden, Pack};
use abstractions::fauna::guide::Guide;
use abstractions::fauna::senses::Stance;
use abstractions::fauna::{Attacking, Beast, Dead};
use abstractions::life::herd::Herd;
use bevy::input::mouse::{MouseButtonInput, MouseScrollUnit, MouseWheel};
use bevy::input::touch::TouchPhase;

use super::*;

/// What a play stage keeps between frames.
#[derive(Default)]
pub struct Play {
    /// The step of the stage under way.
    pub step: u32,
    /// The frame that step began on.
    pub since: u32,
    /// When that step began.
    pub clock: Option<Instant>,
    /// The animal the stage is about.
    pub beast: Option<Entity>,
    /// How many times the stage tried again.
    pub tries: u32,
    /// One bit for each thing done once: a shot taken, a key pressed.
    pub done: u32,
    /// A count the stage keeps: presses, blows.
    pub count: u32,
    /// A number the stage keeps: a mass or a health before.
    pub mark: f32,
    /// A name the stage keeps: the species it uses.
    pub name: String,
    /// The index of that species in the life of the planet.
    pub species: u16,
    /// Frames the stage keeps: when it last pressed, moved the robot or was struck.
    pub marks: [u32; 4],
}

impl Play {
    /// Starts a stage on `frame`: everything forgotten, the first step begun.
    pub fn start(&mut self, frame: u32) {
        *self = Play {
            since: frame,
            clock: Some(Instant::now()),
            ..default()
        };
    }

    /// Begins the next step on `frame`.
    pub fn next(&mut self, frame: u32) {
        self.goto(self.step + 1, frame);
    }

    /// Begins step `step` on `frame`.
    pub fn goto(&mut self, step: u32, frame: u32) {
        (self.step, self.since, self.clock) = (step, frame, Some(Instant::now()));
    }

    /// Returns the frames since the step began.
    pub fn held(&self, frame: u32) -> u32 {
        frame.saturating_sub(self.since)
    }

    /// Returns the seconds of the wall clock since the step began.
    pub fn waited(&self) -> f32 {
        self.clock
            .map_or(0.0, |clock| clock.elapsed().as_secs_f32())
    }

    /// Returns true the first time it is asked about `bit`, and false ever after in this stage.
    pub fn once(&mut self, bit: u32) -> bool {
        let fresh = self.done & (1 << bit) == 0;
        self.done |= 1 << bit;
        fresh
    }
}

/// Logs the facts a play stage shows.
pub fn say(stage: &str, facts: &str) {
    eprintln!("PROBE play {stage}: {facts}");
}

/// Returns the window the inputs are sent to.
fn window(world: &mut World) -> Entity {
    world
        .query_filtered::<Entity, With<PrimaryWindow>>()
        .iter(world)
        .next()
        .unwrap_or(Entity::PLACEHOLDER)
}

/// Presses or releases a button of the mouse.
pub fn mouse(world: &mut World, button: MouseButton, pressed: bool) {
    let window = window(world);
    world.write_message(MouseButtonInput {
        button,
        state: [ButtonState::Released, ButtonState::Pressed][usize::from(pressed)],
        window,
    });
}

/// Turns the wheel of the mouse by `notches` lines, closer when positive.
pub fn wheel(world: &mut World, notches: f32) {
    let window = window(world);
    world.write_message(MouseWheel {
        unit: MouseScrollUnit::Line,
        x: 0.0,
        y: notches,
        window,
        phase: TouchPhase::Moved,
    });
}

/// Presses `code` on the first frame of a step and lets go of it on the second: `held` is the
/// frames since the step began.
pub fn press(world: &mut World, code: KeyCode, held: u32) {
    match held {
        1 => key(world, code, true),
        2 => key(world, code, false),
        _ => {}
    }
}

/// Lets go of every key and button a play stage holds, and closes the map, the pack and the guide.
pub fn tidy(world: &mut World) {
    let keys = world.resource::<Persistent<Settings>>().controls.keys;
    for code in [keys.forward, keys.back, keys.crouch, keys.sprint] {
        key(world, code, false);
    }
    mouse(world, MouseButton::Left, false);
    mouse(world, MouseButton::Right, false);
    if *world.resource::<State<Computer>>().get() != Computer::Closed {
        world
            .resource_mut::<NextState<Computer>>()
            .set(Computer::Closed);
    }
    if *world.resource::<State<Workshop>>().get() != Workshop::Closed {
        world
            .resource_mut::<NextState<Workshop>>()
            .set(Workshop::Closed);
    }
    if *world.resource::<State<Guide>>().get() != Guide::Closed {
        world.resource_mut::<NextState<Guide>>().set(Guide::Closed);
    }
}

/// Ends a stage that could not show what it is for: everything let go, the reason logged.
pub fn gave_up(world: &mut World, stage: &str, why: &str) -> Act {
    tidy(world);
    eprintln!("PROBE play {stage}: gave up because {why}");
    Act::Next
}

/// Returns the health of the frame and of every part of the robot, in words.
pub fn frame_words(world: &World) -> String {
    let condition = *world.resource::<Condition>();
    let parts: Vec<String> = Part::ALL
        .iter()
        .map(|part| format!("{} {:.0}%", part.name(), condition.of(*part) * 100.0))
        .collect();
    format!(
        "frame {:.0}% ({})",
        condition.integrity() * 100.0,
        parts.join(", ")
    )
}

/// Returns what the robot carries, in words: each tool, each material, the load and the held tool.
pub fn pack_words(world: &World) -> String {
    let pack = world.resource::<Pack>();
    let burden = *world.resource::<Burden>();
    let tools: Vec<String> = pack
        .items
        .iter()
        .map(|item| format!("{} {:.2} kg", item.name(), item.mass()))
        .collect();
    let stuff: Vec<String> = pack
        .stacks
        .iter()
        .map(|stack| format!("{} {:.2} kg", called(&stack.stuff), stack.mass))
        .collect();
    format!(
        "{} tools [{}], {} materials [{}], {:.1} of {:.1} kg carried, in hand: {}",
        tools.len(),
        tools.join(", "),
        stuff.len(),
        stuff.join(", "),
        burden.load,
        burden.capacity,
        pack.held()
            .map_or("nothing".to_string(), |item| item.name())
    )
}

/// Returns the strike of the local robot: what its hands do with the tool.
pub fn strike_of(world: &mut World) -> Option<Strike> {
    world.query::<&Strike>().iter(world).next().copied()
}

/// One animal as a play stage reads it.
#[derive(Clone, Copy)]
pub struct Animal {
    /// Planet position of its root, on the ground under it.
    pub at: DVec3,
    /// Unit direction it faces, along the ground, in planet axes.
    pub facing: DVec3,
    /// Index of its species.
    pub species: u16,
    /// What it does about the robot.
    pub stance: Stance,
    /// Share of its health left.
    pub health: f32,
    /// True once it was killed.
    pub dead: bool,
    /// The attack it makes now.
    pub attack: Option<Attacking>,
}

/// Returns the animal `entity`, or nothing once it is gone.
pub fn animal(world: &World, entity: Entity) -> Option<Animal> {
    let beast = world.get_entity(entity).ok()?.get::<Beast>()?;
    let root = &beast.motion.root;
    let at = beast.spot.planet(root.at);
    let ahead = beast.spot.planet(root.at + root.turn * Vec3::NEG_Z) - at;
    Some(Animal {
        at,
        facing: ahead.normalize_or(at.normalize().any_orthonormal_vector()),
        species: beast.index,
        stance: beast.wits.stance,
        health: beast.wits.health.share(),
        dead: world.get::<Dead>(entity).is_some(),
        attack: world.get::<Attacking>(entity).copied(),
    })
}

/// Returns every drawn animal of species index `species` that is alive, nearest the robot first:
/// its entity and what a stage reads of it.
pub fn living(world: &mut World, species: u16) -> Vec<(Entity, Animal)> {
    let me = robot(world);
    let all: Vec<Entity> = world
        .query_filtered::<Entity, With<Beast>>()
        .iter(world)
        .collect();
    let mut found: Vec<(Entity, Animal)> = all
        .into_iter()
        .filter_map(|entity| Some((entity, animal(world, entity)?)))
        .filter(|(_, a)| a.species == species && !a.dead)
        .collect();
    found.sort_by(|a, b| a.1.at.distance(me).total_cmp(&b.1.at.distance(me)));
    found
}

/// Returns how many drawn animals have a part the camera sees this frame.
pub fn in_view(world: &mut World) -> usize {
    let roots: Vec<Vec<Entity>> = world
        .query_filtered::<&Children, With<Beast>>()
        .iter(world)
        .map(|children| children.iter().collect())
        .collect();
    let seen = |part: &Entity| world.get::<ViewVisibility>(*part).is_some_and(|v| v.get());
    roots.iter().filter(|parts| parts.iter().any(seen)).count()
}

/// Returns the herds within `radius` metres of the robot at the time of the game, wherever the
/// scan of the game has not reached yet: the same search the game rolls its watch from.
pub fn herds_around(world: &mut World, radius: f64) -> Vec<Herd> {
    let globe = world.resource::<Terrain>().globe().clone();
    let Some(life) = globe.life() else {
        return Vec::new();
    };
    let time = world.resource::<abstractions::space::Clock>().seconds;
    let me = robot(world).normalize();
    let sample = |dir: DVec3| site_of(&globe, dir);
    let light = |_: DVec3, _: f64| 1.0;
    let height = |dir: DVec3| globe.height(dir);
    let ground = abstractions::life::herd::Ground {
        sample: &sample,
        height: &height,
        light: &light,
    };
    let search = abstractions::life::herd::Search {
        at: me,
        radius,
        planet: globe.radius(),
        time: time + 2.0,
    };
    let mut herds = abstractions::life::herd::near(life, &ground, &search);
    herds.sort_by(|a, b| a.centre.dot(me).total_cmp(&b.centre.dot(me)).reverse());
    herds
}

/// Returns the wind at the robot as a unit direction along the ground in planet axes, the way it
/// blows towards, or nothing in still air.
pub fn wind_way(world: &mut World) -> Option<DVec3> {
    let anchor = *world.resource::<Anchor>();
    let blowing = world.get_resource::<abstractions::wind::Wind>()?.velocity();
    let here = anchor.to_render(robot(world));
    let way = anchor.to_frame(here + blowing) - anchor.to_frame(here);
    way.try_normalize()
}

/// Returns the pitch that looks from the eyes of a robot standing at planet position `feet` to
/// planet position `at`.
pub fn pitch_to(feet: DVec3, at: DVec3) -> f32 {
    let up = feet.normalize();
    let rel = (at - (feet + up * 1.6)).normalize_or(up);
    (rel.dot(up).asin() as f32).clamp(-1.3, 1.3)
}
