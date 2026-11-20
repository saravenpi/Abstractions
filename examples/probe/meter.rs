//! Frame timing and mesh triangle counts, logged per stage.

use super::*;

/// Triangle count of every mesh asset, read when it is added, before a render-only mesh hands its
/// data to the render world.
#[derive(Resource, Default)]
pub struct Tris(pub std::collections::HashMap<AssetId<Mesh>, usize>);

/// Records the triangle count of new and changed meshes.
pub fn count_tris(
    mut events: MessageReader<AssetEvent<Mesh>>,
    meshes: Res<Assets<Mesh>>,
    mut tris: ResMut<Tris>,
) {
    for event in events.read() {
        let (AssetEvent::Added { id } | AssetEvent::Modified { id }) = event else {
            continue;
        };
        let Some(mesh) = meshes.get(*id) else {
            continue;
        };
        let count = match mesh.try_indices_option() {
            Ok(Some(indices)) => indices.len() / 3,
            Ok(None) => mesh.count_vertices() / 3,
            Err(_) => continue,
        };
        tris.0.insert(*id, count);
    }
}

/// Start of the main-thread part of the frame, and the milliseconds since then at which the
/// pre-update, update and post-update schedules were reached.
#[derive(Resource)]
pub struct FrameStart(pub Instant, pub [f32; 3]);

/// Notes the milliseconds since the frame started at which schedule `AT` was reached.
pub fn mark<const AT: usize>(mut start: ResMut<FrameStart>) {
    start.1[AT] = start.0.elapsed().as_secs_f32() * 1000.0;
}

/// Records the main-thread time of the frame and the streaming step that cost the most in it.
pub fn measure(start: Res<FrameStart>, mut tour: ResMut<Tour>, mut meter: ResMut<Meter>) {
    let millis = start.0.elapsed().as_secs_f32() * 1000.0;
    let (step, cost) = meter.worst();
    let name = STAGES.get(tour.stage).map_or("end", |s| s.0);
    tour.frames.push(millis);
    tour.worst.push((
        millis,
        format!(
            "{name}: reached pre-update {:.0}, update {:.0} (fixed steps between), post-update {:.0} ms; worst stream step {step} {:.1} ms",
            start.1[0], start.1[1], start.1[2], cost.worst
        ),
    ));
    meter.reset();
}
