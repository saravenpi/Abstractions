//! The colours of the base, in sRGB: light brushed metal, a few dark parts and one warm accent.

use super::soup::Rgb;

/// Light metal of the container walls and the canopy.
pub const HULL: Rgb = [0.8, 0.82, 0.84];

/// Slightly darker metal of the bevels.
pub const BEVEL: Rgb = [0.66, 0.69, 0.72];

/// Metal of the end frames, the roof fittings and the bench.
pub const TRIM: Rgb = [0.56, 0.6, 0.65];

/// Pale metal of the hatch.
pub const HATCH: Rgb = [0.88, 0.89, 0.9];

/// Dark metal of legs, seams, masts and cables.
pub const DARK: Rgb = [0.17, 0.18, 0.21];

/// Dark glassy blue of the canopy top and the charging cradles.
pub const PANEL: Rgb = [0.1, 0.14, 0.22];

/// Warm accent of the storage crate.
pub const CRATE: Rgb = [0.8, 0.47, 0.18];

/// Grey of the landing pad and the path slabs.
pub const DECK: Rgb = [0.36, 0.38, 0.41];

/// Lighter grey of the centre mark of the pad.
pub const MARK: Rgb = [0.55, 0.57, 0.6];

/// Pale yellow of the edge markings of the pad.
pub const EDGE: Rgb = [0.9, 0.8, 0.4];
