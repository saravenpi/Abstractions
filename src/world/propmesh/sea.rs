//! Parts of the big sea-floor solids: smooth rocks and driftwood. The basalt boulder is built in `stones`.

use super::Part::{self, *};

/// The parts of every big sea solid, in `Plant` order from smooth rock to driftwood; the boulder has none.
#[rustfmt::skip]
pub const SPECS: [&[Part]; 3] = [
    &[
        Ball([0.0, 0.25, 0.0], [0.7, 0.4, 0.6], [0.66, 0.63, 0.56]),
        Ball([0.6, 0.15, 0.2], [0.35, 0.22, 0.3], [0.7, 0.67, 0.6]),
    ],
    &[],
    &[
        Stem([-1.0, 0.15, 0.0], [1.0, 0.25, 0.1], [0.14, 0.1], [0.62, 0.55, 0.45]),
        Stem([0.2, 0.2, 0.05], [0.6, 0.7, 0.35], [0.06, 0.03], [0.58, 0.51, 0.42]),
        Stem([-0.4, 0.2, 0.0], [-0.7, 0.55, -0.3], [0.05, 0.03], [0.6, 0.53, 0.43]),
    ],
];
