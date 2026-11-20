import bevy_pbr::render::forward_io::{UncompressedVertex, Vertex, decompress_vertex};
import bevy_pbr::render::mesh_functions;
import bevy_pbr::render::view_transformations::position_world_to_clip;
import package::world::sea::wake::field::{FoamWakes, wake_bit};

struct Foam {
    color: vec4<f32>,
    clock: vec4<f32>,
    wake: FoamWakes,
}

const WAKE_FOOT: f32 = 1.0;

struct FoamOut {
    @builtin(position) position: vec4<f32>,
    @location(0) side: f32,
    @location(1) alpha: f32,
}

@group(constants::MATERIAL_BIND_GROUP) @binding(0) var<uniform> foam: Foam;

fn wake_lift(local: vec3<f32>) -> vec3<f32> {
    var lift = vec3<f32>(0.0);
    let live = u32(foam.wake.count.x);
    for (var i = 0u; i < live; i++) {
        let row = 6u * i;
        let rel = local - foam.wake.rows[row].xyz;
        let way = foam.wake.rows[row + 1u].xyz;
        let up = foam.wake.rows[row + 2u].xyz;
        let at = vec2<f32>(dot(rel, way), dot(rel, cross(up, way)));
        let bit = wake_bit(foam.wake.rows[row + 3u], foam.wake.rows[row + 4u], foam.wake.rows[row + 5u], at, foam.clock.x, WAKE_FOOT);
        lift += up * bit.x;
    }
    return lift;
}

@vertex
fn vertex(vertex_in: Vertex) -> FoamOut {
    let in = decompress_vertex(vertex_in, vertex_in.instance_index);
    var out: FoamOut;
    let age = max(foam.clock.x - in.uv.y, 0.0);
    let life = max(in.uv_b.y, 1e-3);
    let left = 1.0 - age / life;
    let alive = step(0.0, left) * step(1e-3, in.uv_b.y);
    if (alive == 0.0) {
        out.position = vec4<f32>(0.0, 0.0, 2.0, 1.0);
        out.side = 0.0;
        out.alpha = 0.0;
        return out;
    }
    let reach = in.color.x + in.color.y * age + in.uv.x * (in.color.z + in.color.w * age);
    let pushed = in.position + in.normal * reach * alive;
    let local = pushed + wake_lift(pushed) * alive;
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
