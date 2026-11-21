import bevy_pbr::render::mesh_view_bindings::{view, lights, fog};
import bevy_pbr::render::mesh_view_types::DIRECTIONAL_LIGHT_FLAGS_SHADOWS_ENABLED_BIT;
import bevy_pbr::render::mesh_types::MESH_FLAGS_SHADOW_RECEIVER_BIT;
import bevy_pbr::render::shadows::fetch_directional_shadow;
import bevy_pbr::render::pbr_types::PbrInput;
import bevy_pbr::render::pbr_functions::{calculate_diffuse_color, calculate_F0, calculate_F0_dielectric};
import bevy_pbr::render::pbr_lighting::{
    LightingInput, LAYER_BASE, derive_lighting_input, Fd_Burley, specular, F_AB, EnvBRDFApprox,
    perceptualRoughnessToRoughness,
};

struct Place {
    centre: vec4<f32>,
    star: vec4<f32>,
    colour: vec4<f32>,
    air: vec4<f32>,
    day: vec4<f32>,
    eye_sun: vec4<f32>,
    eye_sky: vec4<f32>,
    eye: vec4<f32>,
    upward: vec4<f32>,
}

struct Daylight {
    tint: vec3<f32>,
    through: f32,
    risen: f32,
    sky: f32,
}

struct Light {
    sun: vec3<f32>,
    sky: vec3<f32>,
    glow: vec3<f32>,
}

const DEPTH: vec3<f32> = vec3<f32>(0.045, 0.10, 0.23);
const HORIZON_MASS: f32 = 12.5;
const LUMA: vec3<f32> = vec3<f32>(0.2126, 0.7152, 0.0722);
const SUN_EDGE: vec2<f32> = vec2<f32>(-0.06, 0.03);
const SKY_EDGE: vec2<f32> = vec2<f32>(-0.12, 0.25);
const NIGHT_EDGE: vec2<f32> = vec2<f32>(-0.02, 0.3);
const SHADOW_EDGE: vec2<f32> = vec2<f32>(0.97, 1.03);
const NIGHT_AMBIENT_COLOR: vec3<f32> = vec3<f32>(0.35, 0.45, 0.9);
const NIGHT_AMBIENT: f32 = 40.0;
const DAY_AMBIENT: f32 = 350.0;
const SPACE_AMBIENT: f32 = 0.2;
const SPACE_LUX: f32 = 12000.0;
const TOP_SCALES: f32 = 12.0;
const SHADOW_FADE: f32 = 0.2;
const SUN_ALIGN: f32 = 0.9999;
const SHIFT_MIN: f32 = 0.005;
const PI: f32 = 3.14159265;
const SHADE_AT: i32 = 9;
const SHADE_GRID: i32 = 15;
const SHADE_SIDE: i32 = 32;
const SHADE_REACH_FADE: f32 = 0.75;
const BOUNCE_CAP: f32 = 1.5;
const TWILIGHT_SHARE: f32 = 0.35;
const TWILIGHT_EDGES: vec3<f32> = vec3<f32>(-0.18, -0.02, 0.12);
const OCCLUSION_RANGE: vec2<f32> = vec2<f32>(30.0, 90.0);
const AEROSOL: vec3<f32> = vec3<f32>(0.000134, 0.000158, 0.000193);
const AEROSOL_SCALE: f32 = 0.15;
const FLAT: f32 = 0.001;
const GLOW_EDGE: vec2<f32> = vec2<f32>(-0.15, 0.05);
const GLOW_NEAR: vec2<f32> = vec2<f32>(45.0, 0.55);
const GLOW_WIDE: vec2<f32> = vec2<f32>(6.0, 0.18);

struct Shadow {
    share: f32,
    fade: f32,
}

fn place_from(map: texture_2d<f32>) -> Place {
    var place: Place;
    place.centre = textureLoad(map, vec2<i32>(0, 0), 0);
    place.star = textureLoad(map, vec2<i32>(1, 0), 0);
    place.colour = textureLoad(map, vec2<i32>(2, 0), 0);
    place.air = textureLoad(map, vec2<i32>(3, 0), 0);
    place.day = textureLoad(map, vec2<i32>(4, 0), 0);
    place.eye_sun = textureLoad(map, vec2<i32>(5, 0), 0);
    place.eye_sky = textureLoad(map, vec2<i32>(6, 0), 0);
    place.eye = textureLoad(map, vec2<i32>(7, 0), 0);
    place.upward = textureLoad(map, vec2<i32>(8, 0), 0);
    return place;
}

fn air_mass(height: f32) -> f32 {
    return 1.0 / (max(height, 0.0) + 1.0 / HORIZON_MASS);
}

fn transmit(height: f32, share: f32) -> vec3<f32> {
    return exp(-DEPTH * (air_mass(height) * share));
}

fn peak(light: vec3<f32>) -> vec3<f32> {
    let top = max(light.x, max(light.y, light.z));
    if (top <= 0.000001) {
        return vec3<f32>(1.0);
    }
    return light / top;
}

fn luminance(light: vec3<f32>) -> f32 {
    return dot(light, LUMA);
}

fn to_linear(c: vec3<f32>) -> vec3<f32> {
    let low = c / 12.92;
    let high = pow((max(c, vec3<f32>(0.0)) + vec3<f32>(0.055)) / 1.055, vec3<f32>(2.4));
    return select(high, low, c <= vec3<f32>(0.04045));
}

fn daylight(height: f32, star: vec3<f32>, depth: f32) -> Daylight {
    let through = star * transmit(height, depth);
    let full = max(luminance(star * transmit(1.0, 1.0)), 0.000001);
    var out: Daylight;
    out.tint = peak(through);
    out.through = min(luminance(through) / full, 1.0);
    out.risen = smoothstep(SUN_EDGE.x, SUN_EDGE.y, height);
    out.sky = smoothstep(SKY_EDGE.x, SKY_EDGE.y, height);
    return out;
}

fn air(altitude: f32, height: f32) -> f32 {
    if (height <= 0.0 || altitude >= height) {
        return 0.0;
    }
    return exp(-TOP_SCALES * max(altitude, 0.0) / height);
}

fn horizon_dip(altitude: f32, radius: f32) -> f32 {
    let high = max(altitude, 0.0);
    return sqrt(high * (2.0 * radius + high)) / max(radius + high, 1.0);
}

fn shadow_at(height: f32, altitude: f32, radius: f32) -> f32 {
    if (height >= 0.0) {
        return 1.0;
    }
    let miss = (1.0 + altitude / max(radius, 1.0)) * sqrt(max(1.0 - height * height, 0.0));
    return smoothstep(SHADOW_EDGE.x, SHADOW_EDGE.y, miss);
}

fn twilight(height: f32) -> f32 {
    return smoothstep(TWILIGHT_EDGES.x, TWILIGHT_EDGES.y, height)
        * (1.0 - smoothstep(TWILIGHT_EDGES.y, TWILIGHT_EDGES.z, height));
}

fn light_at(place: Place, height: f32, altitude: f32) -> Light {
    let radius = place.centre.w;
    let seen = height + horizon_dip(altitude, radius);
    let day = daylight(seen, place.colour.rgb, place.star.w);
    let left = air(altitude, place.air.x) * place.colour.w;
    let open = SPACE_LUX * shadow_at(height, altitude, radius) * (1.0 - left);
    let lux = open + SPACE_LUX * day.through * day.risen * left;
    let night = smoothstep(NIGHT_EDGE.x, NIGHT_EDGE.y, -seen) * left;
    let thin = SPACE_AMBIENT + (1.0 - SPACE_AMBIENT) * left;
    let level = (NIGHT_AMBIENT + (DAY_AMBIENT - NIGHT_AMBIENT) * day.sky) * thin + place.air.y * night;
    var out: Light;
    out.sun = to_linear(mix(peak(place.colour.rgb), day.tint, left)) * lux;
    out.sky = to_linear(mix(NIGHT_AMBIENT_COLOR, place.day.rgb, day.sky)) * level;
    out.glow = to_linear(day.tint) * (twilight(seen) * TWILIGHT_SHARE * level * left);
    return out;
}

fn altitude_at(place: Place, world: vec3<f32>) -> f32 {
    let p = world - place.eye.xyz;
    let radius = place.centre.w;
    let rise = dot(p, p) + 2.0 * (radius + place.eye.w) * dot(p, place.upward.xyz) + place.upward.w;
    return rise / max(sqrt(max(radius * radius + rise, 0.0)) + radius, 0.001);
}

fn height_at(place: Place, world: vec3<f32>) -> vec2<f32> {
    let out = world - place.centre.xyz;
    let far = max(length(out), 0.001);
    return vec2<f32>(dot(out / far, place.star.xyz), altitude_at(place, world));
}

fn light_of(place: Place, world: vec3<f32>) -> Light {
    let at = height_at(place, world);
    return light_at(place, at.x, at.y);
}

fn sky_gain(place: Place, world: vec3<f32>, eye: vec3<f32>) -> f32 {
    let here = height_at(place, world);
    let there = height_at(place, eye);
    let radius = place.centre.w;
    let near = smoothstep(SKY_EDGE.x, SKY_EDGE.y, here.x + horizon_dip(here.y, radius));
    let seen = smoothstep(SKY_EDGE.x, SKY_EDGE.y, there.x + horizon_dip(there.y, radius));
    let swing = DAY_AMBIENT - NIGHT_AMBIENT;
    return (NIGHT_AMBIENT + swing * near) / (NIGHT_AMBIENT + swing * seen);
}

fn sun_response(in: PbrInput, toward: vec3<f32>) -> vec3<f32> {
    let base = in.material.base_color.rgb;
    let rough = in.material.perceptual_roughness;
    let facing = max(dot(in.N, in.V), 0.0001);
    var input: LightingInput;
    input.layers[LAYER_BASE].NdotV = facing;
    input.layers[LAYER_BASE].N = in.N;
    input.layers[LAYER_BASE].R = reflect(-in.V, in.N);
    input.layers[LAYER_BASE].perceptual_roughness = rough;
    input.layers[LAYER_BASE].roughness = perceptualRoughnessToRoughness(rough);
    input.P = in.world_position.xyz;
    input.V = in.V;
    input.diffuse_color = calculate_diffuse_color(
        base,
        in.material.metallic,
        in.material.specular_transmission,
        in.material.diffuse_transmission
    );
    input.metallic = in.material.metallic;
    input.F0_dielectric = calculate_F0_dielectric(in.material.reflectance);
    input.F0_metallic = base;
    input.F_ab = F_AB(rough, facing);
    var derived = derive_lighting_input(in.N, in.V, toward);
    let diffuse = input.diffuse_color * Fd_Burley(&input, &derived);
    let shine = specular(&input, &derived, input.layers[LAYER_BASE].roughness, 1.0);
    return (diffuse + shine) * derived.NdotL;
}

fn ambient_response(diffuse: vec3<f32>, f0: vec3<f32>, rough: f32, facing: f32) -> vec3<f32> {
    let soft = EnvBRDFApprox(diffuse, F_AB(1.0, facing));
    let shine = EnvBRDFApprox(f0, F_AB(rough, facing));
    return soft + shine * saturate(dot(f0, vec3<f32>(50.0 * 0.33)));
}

fn sky_response(in: PbrInput) -> vec3<f32> {
    let base = in.material.base_color.rgb;
    let facing = max(dot(in.N, in.V), 0.0001);
    let diffuse = calculate_diffuse_color(
        base,
        in.material.metallic,
        in.material.specular_transmission,
        in.material.diffuse_transmission
    );
    let f0 = calculate_F0(base, in.material.metallic, in.material.reflectance);
    return ambient_response(diffuse, f0, in.material.perceptual_roughness, facing);
}

fn ground_of(place: Place) -> vec3<f32> {
    return vec3<f32>(place.air.w, place.day.w, place.eye_sky.w);
}

fn hemisphere(sky: vec3<f32>, bounce: vec3<f32>, tilt: f32) -> vec3<f32> {
    return mix(bounce, sky, 0.5 + 0.5 * clamp(tilt, -1.0, 1.0));
}

fn bounce_of(ground: vec3<f32>, sun: vec3<f32>, height: f32, sky: vec3<f32>) -> vec3<f32> {
    let level = luminance(sun) * max(height, 0.0) / PI + luminance(sky);
    let given = ground * level;
    let top = BOUNCE_CAP * luminance(sky);
    let seen = luminance(given);
    return select(given, given * (top / max(seen, 0.000001)), seen > top);
}

fn view_z(world: vec4<f32>) -> f32 {
    return dot(vec4<f32>(
        view.view_from_world[0].z,
        view.view_from_world[1].z,
        view.view_from_world[2].z,
        view.view_from_world[3].z
    ), world);
}

fn sun_shadow(in: PbrInput, toward: vec3<f32>, wanted: f32) -> Shadow {
    var out: Shadow;
    out.share = 1.0;
    out.fade = 0.0;
    let count = lights.n_directional_lights;
    for (var i: u32 = 0u; i < count; i = i + 1u) {
        let light = &lights.directional_lights[i];
        let casts = ((*light).flags & DIRECTIONAL_LIGHT_FLAGS_SHADOWS_ENABLED_BIT) != 0u;
        if (casts && dot((*light).direction_to_light, toward) > SUN_ALIGN) {
            let z = view_z(in.world_position);
            let far = (*light).cascades[max((*light).num_cascades, 1u) - 1u].far_bound;
            out.fade = smoothstep(far * (1.0 - SHADOW_FADE), far, -z);
            let receives = (in.flags & MESH_FLAGS_SHADOW_RECEIVER_BIT) != 0u;
            if (receives && (out.fade > 0.0 || wanted > 0.0)) {
                let blocked = fetch_directional_shadow(i, in.world_position, in.world_normal, z, in.frag_coord.xy);
                out.share = blocked * in.directional_shadow_factor;
            }
            return out;
        }
    }
    return out;
}

fn shade_cell(map: texture_2d<f32>, column: i32, row: i32) -> f32 {
    let x = clamp(column, 0, SHADE_SIDE - 1);
    let y = clamp(row, 0, SHADE_SIDE - 1);
    return textureLoad(map, vec2<i32>(SHADE_GRID + y * SHADE_SIDE + x, 0), 0).x;
}

fn cloud_shade(map: texture_2d<f32>, place: Place, world: vec3<f32>) -> f32 {
    let spot = textureLoad(map, vec2<i32>(SHADE_AT + 3, 0), 0);
    if (spot.w <= 0.0 || place.air.z <= 0.0) {
        return 1.0;
    }
    let col_x = textureLoad(map, vec2<i32>(SHADE_AT, 0), 0).xyz;
    let col_y = textureLoad(map, vec2<i32>(SHADE_AT + 1, 0), 0).xyz;
    let col_z = textureLoad(map, vec2<i32>(SHADE_AT + 2, 0), 0).xyz;
    let east = textureLoad(map, vec2<i32>(SHADE_AT + 4, 0), 0).xyz;
    let north = textureLoad(map, vec2<i32>(SHADE_AT + 5, 0), 0).xyz;
    let out = world - place.centre.xyz;
    let dir = normalize(col_x * out.x + col_y * out.y + col_z * out.z);
    let along = max(dot(dir, spot.xyz), 0.05);
    let across = place.centre.w * vec2<f32>(dot(dir, east), dot(dir, north)) / along;
    let uv = across / spot.w * 0.5 + vec2<f32>(0.5);
    let edge = max(abs(uv.x * 2.0 - 1.0), abs(uv.y * 2.0 - 1.0));
    if (edge >= 1.0) {
        return 1.0;
    }
    let g = uv * f32(SHADE_SIDE) - vec2<f32>(0.5);
    let corner = floor(g);
    let f = g - corner;
    let i = vec2<i32>(corner);
    let low = mix(shade_cell(map, i.x, i.y), shade_cell(map, i.x + 1, i.y), f.x);
    let high = mix(shade_cell(map, i.x, i.y + 1), shade_cell(map, i.x + 1, i.y + 1), f.x);
    let share = mix(low, high, f.y);
    return mix(1.0, share, 1.0 - smoothstep(SHADE_REACH_FADE, 1.0, edge));
}

fn relit(in: PbrInput, lit: vec4<f32>, place: Place) -> vec4<f32> {
    return relit_under(in, lit, place, 1.0);
}

fn relit_under(in: PbrInput, lit: vec4<f32>, place: Place, cloud: f32) -> vec4<f32> {
    if (place.air.z <= 0.0) {
        return lit;
    }
    let at = height_at(place, in.world_position.xyz);
    var here = light_at(place, at.x, at.y);
    here.sun = here.sun * cloud;
    let shift = here.sun - place.eye_sun.rgb;
    let moved = max(abs(shift.x), max(abs(shift.y), abs(shift.z)));
    let eye_top = max(place.eye_sun.x, max(place.eye_sun.y, place.eye_sun.z));
    let wanted = select(0.0, 1.0, moved > SHIFT_MIN * max(eye_top, 1.0));
    let shadow = sun_shadow(in, place.star.xyz, wanted);
    let back = (1.0 - shadow.share) * shadow.fade;
    var sun = vec3<f32>(0.0);
    if (wanted > 0.0 || back > 0.0) {
        sun = (shift * shadow.share + here.sun * back) * sun_response(in, place.star.xyz);
    }
    let near = 1.0 - smoothstep(OCCLUSION_RANGE.x, OCCLUSION_RANGE.y, -view_z(in.world_position));
    let seen = mix(vec3<f32>(1.0), in.diffuse_occlusion, near);
    let up = normalize(in.world_position.xyz - place.centre.xyz);
    let ground = bounce_of(ground_of(place), here.sun, at.x, here.sky);
    let lean = place.star.xyz - up * dot(place.star.xyz, up);
    let toward = select(0.0, saturate(dot(in.N, lean / max(length(lean), 0.000001))), length(lean) > 0.001);
    let around = hemisphere(here.sky, ground, dot(in.N, up)) + here.glow * toward;
    let sky = (around * seen - place.eye_sky.rgb * in.diffuse_occlusion) * sky_response(in);
    return vec4<f32>(max(lit.rgb + (sun + sky) * view.exposure, vec3<f32>(0.0)), lit.a);
}

fn layer(here: f32, there: f32, scale: f32) -> f32 {
    let low = max(min(here, there), 0.0) / scale;
    let high = max(max(here, there), 0.0) / scale;
    let span = high - low;
    if (span < FLAT) {
        return exp(-low);
    }
    return (exp(-low) - exp(-high)) / span;
}

fn air_between(place: Place, world: vec3<f32>) -> vec3<f32> {
    let knobs = fog.directional_light_color.rgb;
    if (knobs.z <= 0.0 || place.air.z <= 0.0) {
        return fog.be;
    }
    let here = place.eye.w;
    let there = altitude_at(place, world);
    let gas = DEPTH * (knobs.x * layer(here, there, knobs.z));
    let low = AEROSOL * (knobs.y * layer(here, there, knobs.z * AEROSOL_SCALE));
    return gas + low + vec3<f32>(fog.directional_light_exponent);
}

fn star_glow(place: Place, toward: vec3<f32>) -> vec3<f32> {
    if (place.air.z <= 0.0) {
        return vec3<f32>(0.0);
    }
    let at = height_at(place, view.world_position);
    let day = daylight(at.x + horizon_dip(at.y, place.centre.w), place.colour.rgb, place.star.w);
    let lit = smoothstep(GLOW_EDGE.x, GLOW_EDGE.y, at.x) * air(at.y, place.air.x) * place.colour.w;
    let off = 1.0 - dot(toward, place.star.xyz);
    let lobes = exp(-off * GLOW_NEAR.x) * GLOW_NEAR.y + exp(-off * GLOW_WIDE.x) * GLOW_WIDE.y;
    return to_linear(day.tint) * (lobes * lit);
}

fn aerial(color: vec4<f32>, world: vec3<f32>, place: Place) -> vec4<f32> {
    let ray = world - view.world_position;
    let far = max(length(ray), 0.001);
    let through = exp(-air_between(place, world) * far);
    let sky = fog.base_color.rgb + star_glow(place, ray / far);
    return vec4<f32>(color.rgb * through + sky * (vec3<f32>(1.0) - through), color.a);
}
