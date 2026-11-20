import bevy_pbr::render::forward_io::{Vertex, VertexOutput, FragmentOutput, decompress_vertex};
import bevy_pbr::render::mesh_functions::get_world_from_local;
import bevy_pbr::render::mesh_view_bindings::{view, globals, lights};
import bevy_pbr::render::mesh_view_types::DIRECTIONAL_LIGHT_FLAGS_SHADOWS_ENABLED_BIT;
import bevy_pbr::render::view_transformations::position_world_to_clip;
import bevy_pbr::render::pbr_fragment::pbr_input_from_standard_material;
import bevy_pbr::render::pbr_functions::main_pass_post_lighting_processing;
import bevy_pbr::render::pbr_lighting::{EnvBRDFApprox, F_AB};
import bevy_pbr::render::shadows::fetch_directional_shadow;
import package::render::daylight::{place_from, light_of};
import package::world::turf::{Turf, cover};

@group(constants::MATERIAL_BIND_GROUP) @binding(100) var place_map: texture_2d<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(101) var<uniform> turf: Turf;
@group(constants::MATERIAL_BIND_GROUP) @binding(102) var wind_map: texture_2d<f32>;

// The inner ring radius in metres, inside which every blade stands, the outer ring radius past which
// none does, and the share of the blades kept between them.
const NEAR: f32 = 15.0;
const FAR: f32 = 40.0;
const EDGE: f32 = 0.16;

// A blade is lit as a rough diffuse sheet that leans to the local up: the lights of the scene with the
// shadow of the ground under it, the light of the place of the planet the way `relit` adds it, and the
// ambient light. The colour of the carpet under a blade is two colours mixed by the share and the speckle
// baked into the second half of its vertex colour, as `carpet` in `turf.wesl` mixes them.
const PI: f32 = 3.14159265;

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
    let carpet = mix(turf.low.rgb, turf.high.rgb, v.color.z) * (0.84 + 0.32 * v.color.w);
    let colour = carpet * tone * (0.9 + 0.2 * seed);
    out.world_position = vec4<f32>(at, 1.0);
    out.position = position_world_to_clip(at);
    out.world_normal = up;
    out.uv = v.uv;
    out.uv_b = v.uv_b;
    out.color = vec4<f32>(colour, 1.0);
    return out;
}

fn sunlight(index: u32, world: vec4<f32>, up: vec3<f32>, frag: vec2<f32>, view_z: f32) -> vec3<f32> {
    let light = &lights.directional_lights[index];
    let facing = saturate(dot(up, (*light).direction_to_light));
    var shadow = 1.0;
    if (facing > 0.0 && ((*light).flags & DIRECTIONAL_LIGHT_FLAGS_SHADOWS_ENABLED_BIT) != 0u) {
        shadow = fetch_directional_shadow(index, world, up, view_z, frag);
    }
    return (*light).color.rgb * (shadow * facing);
}

@fragment
fn fragment(in: VertexOutput, @builtin(front_facing) is_front: bool) -> FragmentOutput {
    var pbr_input = pbr_input_from_standard_material(in, is_front);
    let base = in.color.rgb;
    let up = normalize(in.world_normal);
    let place = place_from(place_map);
    let view_z = dot(vec4<f32>(
        view.view_from_world[0].z,
        view.view_from_world[1].z,
        view.view_from_world[2].z,
        view.view_from_world[3].z
    ), in.world_position);
    var sun = vec3<f32>(0.0);
    for (var i: u32 = 0u; i < lights.n_directional_lights; i = i + 1u) {
        sun += sunlight(i, in.world_position, up, in.position.xy, view_z);
    }
    var sky = lights.ambient_color.rgb;
    if (place.air.z > 0.0) {
        let here = light_of(place, in.world_position.xyz);
        sun += (here.sun - place.eye_sun.rgb) * saturate(dot(up, place.star.xyz));
        sky += here.sky - place.eye_sky.rgb;
    }
    let facing = max(dot(up, pbr_input.V), 0.0001);
    let soft = EnvBRDFApprox(base, F_AB(1.0, facing));
    let lit = max(view.exposure * (base * sun * (1.0 / PI) + soft * sky), vec3<f32>(0.0));
    var out: FragmentOutput;
    out.color = main_pass_post_lighting_processing(pbr_input, vec4<f32>(lit, 1.0));
    return out;
}
