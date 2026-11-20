import bevy_pbr::render::forward_io::{VertexOutput, FragmentOutput};
import bevy_pbr::render::pbr_fragment::pbr_input_from_standard_material;
import bevy_pbr::render::pbr_functions::{alpha_discard, apply_pbr_lighting, main_pass_post_lighting_processing};
@if(VISIBILITY_RANGE_DITHER)
import bevy_pbr::render::pbr_functions::visibility_range_dither;
import package::render::daylight::{altitude_at, place_from, relit};
import package::render::season::{look_at, season_from, tinted};
import package::world::turf::{Turf, carpet, cover, spot};

@group(constants::MATERIAL_BIND_GROUP) @binding(100) var place_map: texture_2d<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(101) var<uniform> turf: Turf;
@group(constants::MATERIAL_BIND_GROUP) @binding(102) var season_map: texture_2d<f32>;

// How far the normal of a fully turfed triangle leans to the local up: a carpet of blades catches
// the light as level ground does, whatever the facet under it.
const SOFT: f32 = 0.6;

@fragment
fn fragment(vertex_output: VertexOutput, @builtin(front_facing) is_front: bool) -> FragmentOutput {
    var in = vertex_output;
    @if(VISIBILITY_RANGE_DITHER)
    visibility_range_dither(in.position, in.visibility_range_dither);
    var pbr_input = pbr_input_from_standard_material(in, is_front);
    pbr_input.material.base_color = alpha_discard(
        pbr_input.material.flags,
        pbr_input.material.alpha_cutoff,
        pbr_input.material.base_color
    );
    let place = place_from(place_map);
    @if(VERTEX_UVS_B) {
        let thick = cover(turf, in.uv_b);
        if (thick > 0.0) {
            let base = pbr_input.material.base_color;
            let mean = 0.5 * (turf.low.rgb + turf.high.rgb);
            let grass = carpet(turf, spot(in.uv_b)) - mean;
            pbr_input.material.base_color = vec4<f32>(max(base.rgb + grass * thick, vec3<f32>(0.0)), base.a);
            let rise = in.world_position.xyz - place.centre.xyz;
            let up = select(pbr_input.N, normalize(rise), place.centre.w > 0.0);
            pbr_input.N = normalize(mix(pbr_input.N, up, SOFT * thick));
        }
    }
    let season = season_from(season_map);
    if (season.axis.w > 0.0 && place.centre.w > 0.0) {
        let dir = normalize(in.world_position.xyz - place.centre.xyz);
        let look = look_at(season, dir, altitude_at(place, in.world_position.xyz));
        let base = pbr_input.material.base_color;
        pbr_input.material.base_color = vec4<f32>(tinted(base.rgb, look), base.a);
    }
    var out: FragmentOutput;
    out.color = relit(pbr_input, apply_pbr_lighting(pbr_input), place);
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);
    return out;
}
