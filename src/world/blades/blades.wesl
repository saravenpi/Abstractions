import bevy_pbr::render::forward_io::{Vertex, VertexOutput, FragmentOutput, decompress_vertex};
import bevy_pbr::render::mesh_functions::get_world_from_local;
import bevy_pbr::render::mesh_view_bindings::{view, globals};
import bevy_pbr::render::view_transformations::position_world_to_clip;
import bevy_pbr::render::pbr_fragment::pbr_input_from_standard_material;
import bevy_pbr::render::pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing};
import package::render::daylight::{place_from, relit};
import package::world::turf::{Turf, carpet, cover, spot};

@group(constants::MATERIAL_BIND_GROUP) @binding(100) var place_map: texture_2d<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(101) var<uniform> turf: Turf;
@group(constants::MATERIAL_BIND_GROUP) @binding(102) var wind_map: texture_2d<f32>;

// The inner ring radius in metres, inside which every blade stands, the outer ring radius past which
// none does, and the share of the blades kept between them.
const NEAR: f32 = 15.0;
const FAR: f32 = 40.0;
const EDGE: f32 = 0.16;

fn kept(distance: f32) -> f32 {
    let inner = mix(1.0, EDGE, smoothstep(NEAR - 3.0, NEAR + 2.0, distance));
    return inner * (1.0 - smoothstep(FAR - 8.0, FAR, distance));
}

fn pixel(distance: f32) -> f32 {
    return 2.0 * distance / (view.clip_from_view[1][1] * max(view.viewport.w, 1.0));
}

fn bend(root: vec3<f32>, up: vec3<f32>, seed: f32) -> vec3<f32> {
    let wind = textureLoad(wind_map, vec2<i32>(0, 0), 0);
    let flat = wind.xyz - up * dot(wind.xyz, up);
    let speed = length(flat);
    let along = select(vec3<f32>(0.0), flat / max(speed, 1e-4), speed > 1e-4);
    let wave = sin(globals.time * (1.6 + 0.8 * seed) + dot(root.xz, along.xz) * 0.35 + seed * 6.283);
    let push = (0.04 * speed + 0.35 * wind.w) * (0.65 + 0.35 * wave);
    let limp = 1.0 - turf.high.a * 0.7;
    return along * min(push * limp, 0.9);
}

@vertex
fn vertex(packed: Vertex) -> VertexOutput {
    var out: VertexOutput;
    let v = decompress_vertex(packed, packed.instance_index);
    let model = get_world_from_local(packed.instance_index);
    let root = (model * vec4<f32>(v.position, 1.0)).xyz;
    let up = normalize((model * vec4<f32>(0.0, 1.0, 0.0, 0.0)).xyz);
    let rank = v.color.x;
    let seed = v.color.y;
    let to_eye = view.world_position - root;
    let distance = length(to_eye);
    let thick = cover(turf, v.uv_b);
    if (rank >= kept(distance) * thick) {
        out.position = vec4<f32>(0.0, 0.0, 0.0, 1.0);
        return out;
    }
    let height = turf.shape.x * (0.55 + 0.9 * seed) * (0.6 + 0.4 * thick);
    let across = cross(up, to_eye);
    let side = select(vec3<f32>(1.0, 0.0, 0.0), across / max(length(across), 1e-4), length(across) > 1e-4);
    let width = max(height * 0.08, pixel(distance) * 1.2);
    let lean = bend(root, up, seed) + side * (seed - 0.5) * 0.4;
    let tip = root + (up * (1.0 - 0.35 * dot(lean, lean)) + lean) * height;
    var at = root + side * (v.uv.x * width * 0.5);
    if (v.uv.y > 0.5) {
        at = tip;
    }
    let tone = 0.55 + 0.55 * v.uv.y;
    let colour = carpet(turf, spot(v.uv_b)) * tone * (0.9 + 0.2 * seed);
    out.world_position = vec4<f32>(at, 1.0);
    out.position = position_world_to_clip(at);
    out.world_normal = up;
    out.uv = v.uv;
    out.uv_b = v.uv_b;
    out.color = vec4<f32>(colour, 1.0);
    return out;
}

@fragment
fn fragment(in: VertexOutput, @builtin(front_facing) is_front: bool) -> FragmentOutput {
    var pbr_input = pbr_input_from_standard_material(in, is_front);
    pbr_input.material.base_color = in.color;
    pbr_input.N = normalize(in.world_normal);
    pbr_input.world_normal = pbr_input.N;
    var out: FragmentOutput;
    out.color = relit(pbr_input, apply_pbr_lighting(pbr_input), place_from(place_map));
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);
    return out;
}
