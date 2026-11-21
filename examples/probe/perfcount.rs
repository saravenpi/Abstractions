//! The counts the `perf` stages log: what is drawn, what lights cast shadows, what the interface
//! rewrites, and what the render passes cost, from the render diagnostics of Bevy.

use abstractions::world::{BladeMaterial, GroundMaterial, Sited};
use avian3d::prelude::RigidBody;
use bevy::diagnostic::DiagnosticsStore;
use bevy::ecs::change_detection::Tick;
use bevy::light::CascadeShadowConfig;

use super::*;

/// What a `perf` stage gathers between two log lines.
#[derive(Resource, Default)]
pub struct Perf {
    /// Real milliseconds of each frame since the last line.
    pub frames: Vec<f32>,
    /// Text and text span writes since the last line.
    pub texts: usize,
    /// Interface node writes since the last line.
    pub nodes: usize,
    /// The change tick of the last count of writes.
    pub tick: Option<Tick>,
    /// Planet position the low flight starts from.
    pub start: Option<DVec3>,
}

/// Meshes in view of one kind: how many, and their triangles.
#[derive(Default, Debug, Clone, Copy)]
pub struct Drawn {
    pub count: usize,
    pub triangles: usize,
}

/// Adds the mesh entities `meshes` accepts and that are in view to a [`Drawn`].
fn drawn<F: bevy::ecs::query::QueryFilter>(world: &mut World) -> Drawn {
    let mut query = world.query_filtered::<(&Mesh3d, &ViewVisibility), F>();
    let tris = world.resource::<Tris>();
    let mut out = Drawn::default();
    for (mesh, seen) in query.iter(world) {
        if seen.get() {
            out.count += 1;
            out.triangles += tris.0.get(&mesh.0.id()).copied().unwrap_or(0);
        }
    }
    out
}

/// Counts this frame's real time and the interface writes since the last count.
pub fn note_frame(world: &mut World) {
    let millis = world.resource::<Time<Real>>().delta_secs() * 1000.0;
    let now = world.change_tick();
    let last = world.resource::<Perf>().tick.unwrap_or(now);
    let changed = |tick: Tick| tick.is_newer_than(last, now);
    let texts = world
        .query::<Ref<Text>>()
        .iter(world)
        .filter(|t| changed(t.last_changed()))
        .count();
    let spans = world
        .query::<Ref<TextSpan>>()
        .iter(world)
        .filter(|t| changed(t.last_changed()))
        .count();
    let nodes = world
        .query::<Ref<Node>>()
        .iter(world)
        .filter(|n| changed(n.last_changed()))
        .count();
    let mut perf = world.resource_mut::<Perf>();
    perf.frames.push(millis);
    perf.texts += texts + spans;
    perf.nodes += nodes;
    perf.tick = Some(now);
}

/// Returns the point and spot lights in view and how many of each cast shadows, and the
/// directional lights that cast shadows with their cascades.
fn lights(world: &mut World) -> String {
    let points: Vec<bool> = (world.query::<(&PointLight, &ViewVisibility)>().iter(world))
        .filter(|(_, seen)| seen.get())
        .map(|(light, _)| light.shadow_maps_enabled)
        .collect();
    let spots: Vec<bool> = (world.query::<(&SpotLight, &ViewVisibility)>().iter(world))
        .filter(|(_, seen)| seen.get())
        .map(|(light, _)| light.shadow_maps_enabled)
        .collect();
    let suns: Vec<usize> = (world
        .query::<(&DirectionalLight, &CascadeShadowConfig)>()
        .iter(world))
    .filter(|(light, _)| light.shadow_maps_enabled)
    .map(|(_, cascades)| cascades.bounds.len())
    .collect();
    let shadowed = |all: &[bool]| all.iter().filter(|s| **s).count();
    format!(
        "point {} ({} shadowed), spot {} ({} shadowed), directional shadowed {} with cascades {:?}",
        points.len(),
        shadowed(&points),
        spots.len(),
        shadowed(&spots),
        suns.len(),
        suns
    )
}

/// Returns the `top` render diagnostics whose path ends with `field`, largest first.
fn passes(store: &DiagnosticsStore, field: &str, top: usize) -> String {
    let mut rows: Vec<(String, f64)> = store
        .iter()
        .filter(|d| d.path().as_str().starts_with("render/") && d.path().as_str().ends_with(field))
        .filter_map(|d| Some((d.path().as_str().to_string(), d.smoothed()?)))
        .collect();
    rows.sort_by(|a, b| b.1.total_cmp(&a.1));
    let total: f64 = rows.iter().map(|r| r.1).sum();
    let named = rows.iter().take(top).map(|(path, value)| {
        let name = path.trim_start_matches("render/").trim_end_matches(field);
        format!("{}={value:.2}", name.trim_end_matches('/'))
    });
    format!(
        "total {total:.2} [{}]",
        named.collect::<Vec<_>>().join(", ")
    )
}

/// Returns the most coarse triangles in view and the most interface writes per 30 frames that
/// `phase` may show: the far terrain under 250,000 triangles on the ground and 150,000 in orbit,
/// and next to no writes while the player stands or hovers still.
fn limits(phase: &str) -> (usize, Option<usize>) {
    match phase {
        "orbit" => (150_000, Some(30)),
        "base" => (250_000, Some(30)),
        _ => (250_000, None),
    }
}

/// Logs whether `phase` met its [`limits`] with `coarse` triangles in view and `writes` of
/// texts and nodes in the last 30 frames.
fn check(phase: &str, coarse: usize, writes: (usize, usize)) {
    let (most, still) = limits(phase);
    let terrain = ["FAIL", "PASS"][usize::from(coarse <= most)];
    let quiet = still.map_or("PASS", |still| {
        ["FAIL", "PASS"][usize::from(writes.0 + writes.1 <= still)]
    });
    info!(
        "PROBE perf {phase} check: coarse {coarse} triangles of at most {most}: {terrain}; ui {} text and {} node writes \
         of at most {still:?}: {quiet}",
        writes.0, writes.1
    );
}

/// Logs the counts of `phase` at `frame` and starts the next window. On the `last` line of the
/// phase it also logs whether the counts meet [`limits`].
pub fn log_perf(world: &mut World, phase: &str, (frame, last): (u32, bool)) {
    let perf = std::mem::take(&mut world.resource_mut::<Perf>().frames);
    let mean = perf.iter().sum::<f32>() / perf.len().max(1) as f32;
    let max = perf.iter().copied().fold(0.0, f32::max);
    let entities = world.entities().count_spawned();
    let all = drawn::<()>(world);
    let coarse = drawn::<(
        With<MeshMaterial3d<GroundMaterial>>,
        With<Sited>,
        Without<RigidBody>,
    )>(world);
    let chunks = drawn::<(With<MeshMaterial3d<GroundMaterial>>, With<RigidBody>)>(world);
    let grass = drawn::<With<MeshMaterial3d<BladeMaterial>>>(world);
    let meshes = world.query::<&Mesh3d>().iter(world).count();
    let ui = world.query::<&Node>().iter(world).count();
    let texts =
        world.query::<&Text>().iter(world).count() + world.query::<&TextSpan>().iter(world).count();
    let shine = lights(world);
    let (written, moved) = {
        let mut perf = world.resource_mut::<Perf>();
        (
            std::mem::take(&mut perf.texts),
            std::mem::take(&mut perf.nodes),
        )
    };
    info!(
        "PROBE perf {phase} f{frame}: frame mean {mean:.1} ms max {max:.1} ms over {} frames; {entities} entities; \
         {meshes} meshes, {} in view, {} triangles in view; coarse {} ({} tris), chunks {} ({} tris), grass {} ({} tris) \
         in view; lights {shine}; ui {ui} nodes, {texts} texts, {written} text writes and {moved} node writes",
        perf.len(),
        all.count,
        all.triangles,
        coarse.count,
        coarse.triangles,
        chunks.count,
        chunks.triangles,
        grass.count,
        grass.triangles,
    );
    if last {
        check(phase, coarse.triangles, (written, moved));
    }
    if let Some(store) = world.get_resource::<DiagnosticsStore>() {
        info!(
            "PROBE perf {phase} f{frame} passes: cpu ms {}; gpu ms {}; vertices {}; primitives {}; fragments {}",
            passes(store, "elapsed_cpu", 6),
            passes(store, "elapsed_gpu", 6),
            passes(store, "vertex_shader_invocations", 4),
            passes(store, "clipper_primitives_out", 4),
            passes(store, "fragment_shader_invocations", 4),
        );
    }
}
