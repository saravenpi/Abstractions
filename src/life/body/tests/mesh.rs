//! Tests of part geometry: the triangle count helper agrees with the mesh, fronts face out, and patterns
//! shade the tissue colour without leaving the colour range.

use bevy::math::Vec3;

use super::every;
use crate::life::body::shape::geometry;
use crate::life::body::{Part, ShapeKind, soup, soup_toned, triangles};
use crate::life::{Pattern, Reaction, Tissue, TissueKind};

/// Returns a tissue of colour `colour` with `pattern`.
fn tissue(pattern: Pattern) -> Tissue {
    Tissue {
        kind: TissueKind::Hide,
        hardness: 2.0,
        toughness: 0.5,
        density: 1100.0,
        elasticity: 0.4,
        heat: 330.0,
        conductivity: 0.2,
        reaction: Reaction::Inert,
        colour: [0.6, 0.4, 0.2],
        pattern,
        gloss: 0.1,
        translucency: 0.0,
        glow: 0.0,
    }
}

/// Returns the point inside a part that every front of a closed shape faces away from.
fn inside(part: &Part) -> Vec3 {
    match part.shape.hangs() {
        true => Vec3::NEG_Y * part.size.y * 0.5,
        false => Vec3::ZERO,
    }
}

#[test]
fn the_triangle_count_matches_the_mesh_and_fronts_face_out() {
    every(|body, form, _| {
        for part in &body.parts {
            let soup = geometry(part);
            assert_eq!(
                soup.faces.len(),
                triangles(part),
                "{form:?} {:?}",
                part.shape
            );
            if part.shape == ShapeKind::Fan {
                continue;
            }
            for face in &soup.faces {
                let centre = (face.corners[0] + face.corners[1] + face.corners[2]) / 3.0;
                assert!(
                    face.normal.dot(centre - inside(part)) > 0.0,
                    "{form:?} {:?} {:?}",
                    part.kind,
                    part.shape
                );
            }
        }
    });
}

#[test]
fn patterns_shade_the_tissue_within_range() {
    every(|body, _, _| {
        let Some(torso) = body.parts.first() else {
            return;
        };
        let plain = soup(torso, &tissue(Pattern::Plain));
        for pattern in [
            Pattern::Stripes,
            Pattern::Spots,
            Pattern::Bands,
            Pattern::Countershade,
        ] {
            let painted = soup(torso, &tissue(pattern));
            assert!(
                painted
                    .faces
                    .iter()
                    .all(|f| f.colour.iter().all(|c| (0.0..=1.0).contains(c)))
            );
            assert_eq!(painted.faces.len(), plain.faces.len());
        }
        let shaded = soup(torso, &tissue(Pattern::Countershade));
        assert!(
            shaded
                .faces
                .iter()
                .zip(&plain.faces)
                .any(|(a, b)| a.colour != b.colour)
        );
        let mut accent = tissue(Pattern::Plain);
        accent.colour = [0.1, 0.8, 0.9];
        let toned = soup_toned(torso, &tissue(Pattern::Stripes), Some(&accent));
        assert!(toned.faces.iter().any(|f| f.colour[2] > 0.5));
    });
}
