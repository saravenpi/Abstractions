//! The plant vocabulary and the meshes of the big props: trees, cacti, bushes, logs and rocks.

mod merge;

use bevy::prelude::*;

use super::shapes::{FlatMesh, shade};
use super::solids::{cone, fan, frustum, icosa};

pub use merge::merge_props;

/// Number of mesh variants built for every big plant.
pub const VARIANTS: usize = 3;

/// Number of big plants, which come first in [`Plant`]; the ground cover follows.
pub const BIG: usize = 13;

/// Number of tree kinds, which come first in [`Plant`].
pub const TREES: usize = 5;

/// Index of the first flower in [`Plant`].
pub const FLOWER: usize = 18;

/// Everything that can be placed on the ground. Declaration order is the index into the mesh tables.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Plant {
    /// Round oak tree.
    Oak,
    /// Tall green pine.
    Pine,
    /// Pink blossom tree.
    Cherry,
    /// Pine with white snow on its tiers.
    SnowPine,
    /// Palm with a leaning trunk.
    Palm,
    /// Saguaro cactus with two arms.
    Saguaro,
    /// Small round cactus with a flower.
    RoundCactus,
    /// Leafy bush.
    Bush,
    /// Bare brown shrub.
    DeadBush,
    /// Fallen log.
    Log,
    /// Grey rock.
    Rock,
    /// Pale ice-grey rock.
    IceRock,
    /// Orange sandstone rock.
    Sandstone,
    /// Grass tuft.
    Grass,
    /// Tall reed with a brown head.
    Reed,
    /// Big leafy tropical plant.
    Jungle,
    /// Fern.
    Fern,
    /// Red-capped mushroom.
    Mushroom,
    /// Red poppy.
    Poppy,
    /// White daisy.
    Daisy,
    /// Yellow buttercup.
    Buttercup,
    /// Blue bell.
    Bluebell,
    /// Pink blossom flower.
    Pink,
    /// Bright tropical orchid.
    Orchid,
}

/// One solid of a big prop, in sRGB colors.
#[derive(Clone, Copy)]
enum Part {
    /// Tube from a point to a point: radii at both ends, color.
    Stem([f32; 3], [f32; 3], [f32; 2], [f32; 3]),
    /// Cone: base center, radius and height, color.
    Cone([f32; 3], [f32; 2], [f32; 3]),
    /// Jittered ball: center, radii, color.
    Ball([f32; 3], [f32; 3], [f32; 3]),
    /// Fan of leaves: center, fan shape, color.
    Fronds([f32; 3], [f32; 5], [f32; 3]),
}

use Part::*;

/// The parts of every big plant, in [`Plant`] order.
#[rustfmt::skip]
const SPECS: [&[Part]; BIG] = [
    &[
        Stem([0.0, 0.0, 0.0], [0.0, 1.6, 0.0], [0.3, 0.2], [0.36, 0.24, 0.14]),
        Ball([0.0, 2.5, 0.0], [1.5, 1.2, 1.5], [0.2, 0.46, 0.18]),
        Ball([0.8, 2.0, 0.3], [0.9, 0.8, 0.9], [0.24, 0.5, 0.2]),
        Ball([-0.7, 2.1, -0.4], [0.9, 0.9, 0.9], [0.17, 0.42, 0.17]),
    ],
    &[
        Stem([0.0, 0.0, 0.0], [0.0, 1.4, 0.0], [0.26, 0.2], [0.34, 0.22, 0.13]),
        Cone([0.0, 0.9, 0.0], [1.7, 1.7], [0.13, 0.38, 0.2]),
        Cone([0.0, 1.8, 0.0], [1.35, 1.5], [0.15, 0.42, 0.22]),
        Cone([0.0, 2.6, 0.0], [1.0, 1.4], [0.13, 0.38, 0.2]),
        Cone([0.0, 3.3, 0.0], [0.6, 1.2], [0.16, 0.44, 0.23]),
    ],
    &[
        Stem([0.0, 0.0, 0.0], [0.15, 1.4, 0.0], [0.26, 0.2], [0.4, 0.27, 0.2]),
        Stem([0.15, 1.4, 0.0], [0.7, 2.0, 0.2], [0.14, 0.1], [0.4, 0.27, 0.2]),
        Ball([0.0, 2.6, 0.0], [1.5, 1.1, 1.5], [0.97, 0.66, 0.76]),
        Ball([0.9, 2.1, 0.3], [0.9, 0.8, 0.9], [0.95, 0.58, 0.7]),
        Ball([-0.8, 2.2, -0.3], [0.9, 0.8, 0.9], [1.0, 0.76, 0.84]),
    ],
    &[
        Stem([0.0, 0.0, 0.0], [0.0, 1.4, 0.0], [0.26, 0.2], [0.34, 0.22, 0.13]),
        Cone([0.0, 0.9, 0.0], [1.7, 1.7], [0.13, 0.36, 0.2]),
        Cone([0.0, 1.67, 0.0], [0.96, 0.94], [0.93, 0.96, 1.0]),
        Cone([0.0, 1.8, 0.0], [1.35, 1.5], [0.15, 0.4, 0.22]),
        Cone([0.0, 2.48, 0.0], [0.76, 0.83], [0.93, 0.96, 1.0]),
        Cone([0.0, 2.6, 0.0], [1.0, 1.4], [0.13, 0.36, 0.2]),
        Cone([0.0, 3.23, 0.0], [0.57, 0.77], [0.93, 0.96, 1.0]),
    ],
    &[
        Stem([0.0, 0.0, 0.0], [0.4, 1.8, 0.1], [0.3, 0.22], [0.58, 0.44, 0.28]),
        Stem([0.4, 1.8, 0.1], [1.0, 3.5, 0.2], [0.22, 0.16], [0.52, 0.4, 0.26]),
        Fronds([1.0, 3.5, 0.2], [8.0, 2.0, -0.4, 0.9, 0.45], [0.2, 0.58, 0.25]),
        Fronds([1.0, 3.55, 0.2], [5.0, 1.3, 0.3, 0.5, 0.35], [0.3, 0.66, 0.3]),
        Ball([1.0, 3.35, 0.2], [0.3, 0.25, 0.3], [0.4, 0.28, 0.16]),
    ],
    &[
        Stem([0.0, 0.0, 0.0], [0.0, 3.0, 0.0], [0.32, 0.28], [0.26, 0.5, 0.3]),
        Ball([0.0, 3.0, 0.0], [0.3, 0.3, 0.3], [0.3, 0.55, 0.33]),
        Stem([0.2, 1.3, 0.0], [0.9, 1.3, 0.0], [0.2, 0.2], [0.28, 0.52, 0.3]),
        Stem([0.9, 1.3, 0.0], [0.9, 2.3, 0.0], [0.2, 0.17], [0.3, 0.55, 0.33]),
        Stem([-0.2, 1.8, 0.0], [-0.8, 1.8, 0.0], [0.18, 0.18], [0.28, 0.52, 0.3]),
        Stem([-0.8, 1.8, 0.0], [-0.8, 2.5, 0.0], [0.18, 0.15], [0.3, 0.55, 0.33]),
    ],
    &[
        Ball([0.0, 0.4, 0.0], [0.45, 0.4, 0.45], [0.3, 0.55, 0.3]),
        Ball([0.0, 0.82, 0.0], [0.14, 0.1, 0.14], [0.95, 0.4, 0.55]),
    ],
    &[
        Ball([0.0, 0.35, 0.0], [0.7, 0.5, 0.7], [0.2, 0.45, 0.17]),
        Ball([0.5, 0.25, 0.2], [0.45, 0.35, 0.45], [0.24, 0.5, 0.2]),
        Ball([-0.45, 0.28, -0.3], [0.45, 0.38, 0.45], [0.17, 0.4, 0.15]),
    ],
    &[
        Stem([0.0, 0.0, 0.0], [0.5, 0.9, 0.1], [0.05, 0.02], [0.5, 0.42, 0.32]),
        Stem([0.0, 0.0, 0.0], [-0.4, 0.8, 0.3], [0.05, 0.02], [0.46, 0.38, 0.28]),
        Stem([0.0, 0.0, 0.0], [0.1, 0.7, -0.5], [0.05, 0.02], [0.52, 0.44, 0.33]),
        Stem([0.0, 0.0, 0.0], [-0.2, 0.9, -0.3], [0.05, 0.02], [0.48, 0.4, 0.3]),
        Stem([0.0, 0.0, 0.0], [0.4, 0.5, 0.4], [0.05, 0.02], [0.5, 0.42, 0.32]),
    ],
    &[
        Stem([-0.9, 0.3, 0.0], [0.9, 0.3, 0.1], [0.3, 0.3], [0.4, 0.28, 0.17]),
        Stem([0.2, 0.3, 0.05], [0.5, 0.75, 0.2], [0.07, 0.05], [0.4, 0.28, 0.17]),
        Ball([-0.3, 0.55, 0.02], [0.4, 0.15, 0.3], [0.25, 0.5, 0.2]),
    ],
    &[
        Ball([0.0, 0.45, 0.0], [0.95, 0.65, 0.85], [0.47, 0.47, 0.5]),
        Ball([0.8, 0.25, 0.3], [0.45, 0.3, 0.4], [0.42, 0.42, 0.45]),
        Ball([-0.6, 0.2, -0.4], [0.35, 0.25, 0.3], [0.5, 0.5, 0.53]),
    ],
    &[
        Ball([0.0, 0.45, 0.0], [0.95, 0.65, 0.85], [0.62, 0.72, 0.8]),
        Ball([0.8, 0.25, 0.3], [0.45, 0.3, 0.4], [0.55, 0.65, 0.75]),
        Ball([-0.6, 0.2, -0.4], [0.35, 0.25, 0.3], [0.7, 0.8, 0.88]),
    ],
    &[
        Ball([0.0, 0.45, 0.0], [0.95, 0.65, 0.85], [0.78, 0.55, 0.33]),
        Ball([0.8, 0.25, 0.3], [0.45, 0.3, 0.4], [0.72, 0.48, 0.28]),
        Ball([-0.6, 0.2, -0.4], [0.35, 0.25, 0.3], [0.84, 0.62, 0.4]),
    ],
];

/// Builds the mesh of big plant number `kind` (an index into [`Plant`]), variant `variant`, base at the origin.
pub fn big_mesh(kind: usize, variant: usize) -> Mesh {
    let mut m = FlatMesh::default();
    let v = variant as f32 - 1.0;
    let s = Vec3::new(1.0 - 0.05 * v, 1.0 + 0.1 * v, 1.0 - 0.05 * v);
    let tone = 1.0 + 0.05 * v;
    let at = |a: [f32; 3]| Vec3::from(a) * s;
    for (i, part) in SPECS[kind].iter().enumerate() {
        match *part {
            Stem(a, b, r, c) => frustum(&mut m, at(a), at(b), Vec2::from(r) * s.x, shade(c, tone)),
            Cone(a, r, c) => cone(
                &mut m,
                at(a),
                Vec2::new(r[0] * s.x, r[1] * s.y),
                7,
                shade(c, tone),
            ),
            Ball(a, r, c) => {
                let shape = (0.22, (variant * 31 + i) as u32);
                icosa(&mut m, at(a), Vec3::from(r) * s, shape, shade(c, tone));
            }
            Fronds(a, f, c) => fan(&mut m, at(a), f, variant as f32 * 0.8, shade(c, tone)),
        }
    }
    m.into_mesh()
}
