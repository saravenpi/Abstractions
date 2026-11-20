// The season of the ground, the same for every shader that draws ground: the ground of the world and the
// ball a planet becomes from afar. A block of the state of the planet and the direction and height of one
// fragment give how far its leaves have turned and how much snow lies on it. `season.rs` holds the rules
// and the numbers of these constants.

struct Season {
    // Spin axis in render axes in xyz, 1 in w when the planet has a season to show and 0 when it has none.
    axis: vec4<f32>,
    // Mean temperature in kelvin, the pole to equator shape, the swing of the season, the lapse per metre.
    heat: vec4<f32>,
    // Lean of the sun path per unit of the sine of the latitude, autumn of the north, autumn of the south,
    // and 1 where there is air for snow.
    leaf: vec4<f32>,
    // Direction of the warm side of a locked moon in render axes in xyz and its extra kelvin in w.
    near: vec4<f32>,
}

const FREEZE: vec2<f32> = vec2<f32>(269.0, 275.0);
const MEAN_SQUARE: f32 = 0.33333334;
const AUTUMN_SHARE: f32 = 0.75;
const GOLD: vec3<f32> = vec3<f32>(0.8, 0.56, 0.18);
const RUST: vec3<f32> = vec3<f32>(0.7, 0.3, 0.12);
const SNOW: vec3<f32> = vec3<f32>(0.93, 0.95, 0.98);
const LEAF_GAIN: f32 = 6.0;
const SHINE: vec3<f32> = vec3<f32>(0.2126, 0.7152, 0.0722);

fn season_from(map: texture_2d<f32>) -> Season {
    var season: Season;
    season.axis = textureLoad(map, vec2<i32>(0, 0), 0);
    season.heat = textureLoad(map, vec2<i32>(1, 0), 0);
    season.leaf = textureLoad(map, vec2<i32>(2, 0), 0);
    season.near = textureLoad(map, vec2<i32>(3, 0), 0);
    return season;
}

// Returns how far the leaves have turned in x and how much snow lies in y, at the unit direction `dir` from
// the planet centre, `height` metres above its sea level.
fn look_at(season: Season, dir: vec3<f32>, height: f32) -> vec2<f32> {
    if (season.axis.w <= 0.0) {
        return vec2<f32>(0.0);
    }
    let sine = clamp(dot(dir, season.axis.xyz), -1.0, 1.0);
    let shape = season.heat.y * (MEAN_SQUARE - sine * sine);
    let side = season.near.w * dot(dir, season.near.xyz);
    let kelvin = season.heat.x + shape + season.heat.z * sine + side - season.heat.w * max(height, 0.0);
    let strength = min(season.leaf.x * abs(sine), 1.0);
    let window = select(season.leaf.y, season.leaf.z, sine < 0.0);
    let snow = (1.0 - smoothstep(FREEZE.x, FREEZE.y, kelvin)) * season.leaf.w;
    return vec2<f32>(strength * window, snow);
}

fn to_srgb(c: vec3<f32>) -> vec3<f32> {
    let low = c * 12.92;
    let high = 1.055 * pow(max(c, vec3<f32>(0.0)), vec3<f32>(1.0 / 2.4)) - vec3<f32>(0.055);
    return select(high, low, c <= vec3<f32>(0.0031308));
}

fn from_srgb(c: vec3<f32>) -> vec3<f32> {
    let low = c / 12.92;
    let high = pow((max(c, vec3<f32>(0.0)) + vec3<f32>(0.055)) / 1.055, vec3<f32>(2.4));
    return select(high, low, c <= vec3<f32>(0.04045));
}

// Returns the linear colour `color` under the season `look`: grass and leaves turn toward gold, then rust,
// and fade to snow, as `tint` does in `season.rs`. Rock, sand, wood and water stay as they are.
fn tinted(color: vec3<f32>, look: vec2<f32>) -> vec3<f32> {
    if (look.x <= 0.0 && look.y <= 0.0) {
        return color;
    }
    let srgb = to_srgb(color);
    let over = srgb.y - max(srgb.x, srgb.z);
    let leaf = clamp(over / max(srgb.y, 0.001) * LEAF_GAIN, 0.0, 1.0);
    if (leaf <= 0.0) {
        return color;
    }
    let hue = mix(GOLD, RUST, look.x);
    let alike = hue * (dot(srgb, SHINE) / dot(hue, SHINE));
    let turned = mix(srgb, alike, look.x * AUTUMN_SHARE * leaf);
    return from_srgb(min(mix(turned, SNOW, look.y * leaf), vec3<f32>(1.0)));
}
