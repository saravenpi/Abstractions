import bevy_pbr::render::forward_io::{UncompressedVertex, Vertex, decompress_vertex};
import bevy_pbr::render::mesh_functions;
import bevy_pbr::render::view_transformations::position_world_to_clip;

struct Foam {
    color: vec4<f32>,
    clock: vec4<f32>,
}

struct FoamOut {
    @builtin(position) position: vec4<f32>,
    @location(0) side: f32,
    @location(1) alpha: f32,
}

@group(constants::MATERIAL_BIND_GROUP) @binding(0) var<uniform> foam: Foam;

@vertex
fn vertex(vertex_in: Vertex) -> FoamOut {
    let in = decompress_vertex(vertex_in, vertex_in.instance_index);
    var out: FoamOut;
    let age = max(foam.clock.x - in.uv.y, 0.0);
    let life = max(in.uv_b.y, 1e-3);
    let left = 1.0 - age / life;
    let alive = step(0.0, left) * step(1e-3, in.uv_b.y);
    let reach = in.color.x + in.color.y * age + in.uv.x * (in.color.z + in.color.w * age);
    let local = in.position + in.normal * reach * alive;
    let world_from_local = mesh_functions::get_world_from_local(in.instance_index);
    let world = mesh_functions::mesh_position_local_to_world(world_from_local, vec4<f32>(local, 1.0));
    out.position = position_world_to_clip(world.xyz);
    out.side = in.uv.x;
    out.alpha = in.uv_b.x * clamp(left, 0.0, 1.0) * clamp(left, 0.0, 1.0) * alive;
    return out;
}

@fragment
fn fragment(in: FoamOut) -> @location(0) vec4<f32> {
    let profile = 1.0 - in.side * in.side;
    return vec4<f32>(foam.color.rgb, foam.color.w * in.alpha * profile);
}
