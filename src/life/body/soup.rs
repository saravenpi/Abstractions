//! A flat-shaded triangle soup as plain data: what every creature part is drawn with.

use bevy::math::Vec3;

/// One flat triangle with its own normal and colour.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Face {
    /// Corners, wound counter-clockwise seen from the front.
    pub corners: [Vec3; 3],
    /// Unit normal of the front side.
    pub normal: Vec3,
    /// Linear RGB colour.
    pub colour: [f32; 3],
}

/// Flat-shaded triangles, one normal and one colour each.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Soup {
    /// Every triangle.
    pub faces: Vec<Face>,
}

impl Soup {
    /// Adds a white triangle wound so that its front faces the direction `out`.
    pub fn tri(&mut self, p: [Vec3; 3], out: Vec3) {
        let flip = (p[1] - p[0]).cross(p[2] - p[0]).dot(out) < 0.0;
        let corners = if flip { [p[0], p[2], p[1]] } else { p };
        let normal = (corners[1] - corners[0])
            .cross(corners[2] - corners[0])
            .normalize_or(out.normalize_or(Vec3::Y));
        self.faces.push(Face {
            corners,
            normal,
            colour: [1.0; 3],
        });
    }

    /// Adds a quad as two white triangles wound so that their fronts face the direction `out`.
    pub fn quad(&mut self, p: [Vec3; 4], out: Vec3) {
        self.tri([p[0], p[1], p[2]], out);
        self.tri([p[0], p[2], p[3]], out);
    }
}
