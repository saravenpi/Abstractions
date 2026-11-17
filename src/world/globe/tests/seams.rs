//! Seam tests: the ground is one function of the direction, whichever cube face chart reads it.

use std::sync::Arc;

use bevy::math::DVec3;

use crate::world::globe::{Face, Globe};
use crate::world::{Biome, Surface, Terrain};

/// Returns the face coordinates of the point `t` along side `side` of a face, pushed to `reach`.
fn side_uv(side: usize, t: f64, reach: f64) -> (f64, f64) {
    [(reach, t), (-reach, t), (t, reach), (t, -reach)][side]
}

/// A point of a cube edge with the unit tangent that crosses the edge away from its face.
struct Crossing {
    /// Unit direction of the point on the edge.
    edge: DVec3,
    /// Unit tangent at the edge, perpendicular to it, pointing off the face.
    out: DVec3,
}

impl Crossing {
    /// Builds the crossing at `t` along side `side` of `face`.
    fn new(face: Face, side: usize, t: f64) -> Self {
        let at = |reach: f64| {
            let (u, v) = side_uv(side, t, reach);
            face.dir(u, v)
        };
        let edge = at(1.0);
        let step = at(1.001) - edge;
        let out = (step - edge * step.dot(edge)).normalize();
        Self { edge, out }
    }

    /// Returns the unit direction `metres` past the edge on a globe of `radius` metres.
    fn dir(&self, metres: f64, radius: f64) -> DVec3 {
        (self.edge + self.out * (metres / radius)).normalize()
    }
}

/// Checks one edge crossing: both charts and the direct sample agree, and nothing jumps at the edge.
fn check_crossing(globe: &Arc<Globe>, face: Face, cross: &Crossing) {
    let r = globe.radius();
    let direct = |m: f64| globe.height(cross.dir(m, r));
    let slope = (-3..3)
        .map(|i| (direct(f64::from(i) + 1.5) - direct(f64::from(i) + 0.5)).abs())
        .fold(0.0, f32::max);
    let other = Face::locate(cross.dir(0.5, r)).0;
    assert_ne!(other, face);
    let views = [face, other].map(|f| Terrain::on(globe.clone(), globe.face_chart(f)));
    let through = |view: &Terrain, m: f64| {
        let p = view.chart().locate(cross.dir(m, r));
        view.height(p.x, p.y)
    };
    let gap = (through(&views[0], -0.5) - through(&views[1], 0.5)).abs();
    assert!(
        gap < 0.05 + slope,
        "{face:?}/{other:?} step {gap} over a slope of {slope}"
    );
    for (view, m) in views.iter().flat_map(|v| [(v, -0.5), (v, 0.5)]) {
        let gap = (through(view, m) - direct(m)).abs();
        assert!(
            gap < 0.05 + slope * 0.01,
            "{face:?}/{other:?} chart is {gap} off at {m} m"
        );
    }
    let jump = (direct(0.0005) - direct(-0.0005)).abs();
    assert!(
        jump < 0.05 + slope * 0.001,
        "{face:?}/{other:?} jumps by {jump} at the edge"
    );
}

/// Heights 0.5 m either side of each of the 12 cube edges match through both face charts and
/// directly, within the local slope.
#[test]
fn heights_are_continuous_across_cube_edges() {
    for (seed, radius) in [(9, 6000.0), (42, 3000.0)] {
        let globe = Arc::new(Globe::new(seed, radius, 22.0));
        let mut pairs = std::collections::HashSet::new();
        for (face, side) in Face::ALL
            .into_iter()
            .flat_map(|f| (0..4).map(move |s| (f, s)))
        {
            for i in 0..60 {
                let cross = Crossing::new(face, side, -0.97 + 0.0329 * f64::from(i));
                check_crossing(&globe, face, &cross);
                let other = Face::locate(cross.dir(0.5, radius)).0;
                pairs.insert([(face as u8).min(other as u8), (face as u8).max(other as u8)]);
            }
        }
        assert_eq!(pairs.len(), 12);
    }
}

/// Returns the biome and the surface `view` reads in unit direction `dir`.
fn read(view: &Terrain, dir: DVec3) -> (Biome, Surface) {
    let p = view.chart().locate(dir);
    (view.biome(p.x, p.y), view.surface(p.x, p.y))
}

/// Biome and surface are the same when one direction is read through two different face charts.
#[test]
fn biome_and_surface_ignore_the_chart() {
    let globe = Arc::new(Globe::new(42, 6000.0, 22.0));
    let views = Face::ALL.map(|f| Terrain::on(globe.clone(), globe.face_chart(f)));
    let mut compared = 0;
    for (face, k) in Face::ALL
        .into_iter()
        .flat_map(|f| (0..900).map(move |k| (f, k)))
    {
        let (u, v) = (
            f64::from(k % 30) / 14.5 - 1.0,
            f64::from(k / 30) / 14.5 - 1.0,
        );
        let dir = face.dir(u * 0.999, v * 0.999);
        let direct = (globe.biome(dir), globe.surface(dir));
        let seen = Face::ALL
            .into_iter()
            .zip(&views)
            .filter(|(other, _)| other.axes()[0].dot(dir) > 0.35)
            .inspect(|(other, view)| {
                assert_eq!(read(view, dir), direct, "{face:?} {other:?} {dir}")
            })
            .count();
        assert!(seen >= 1, "{face:?} does not see its own direction {dir}");
        compared += seen - 1;
    }
    assert!(
        compared > 1000,
        "only {compared} directions were read through two charts"
    );
}
