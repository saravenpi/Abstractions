//! The liquid of a planet as the water shader draws it: its colours, its own light and whether it shows.

use bevy::math::Vec3;

use crate::world::Globe;

/// Share of the sky reflection molten rock keeps: its crust is rough.
const DULL: f32 = 0.1;

/// How much darker than the open liquid the deep part of a liquid that is not water is.
const DEPTH_SHADE: f32 = 0.3;

/// How much cooler than the open flow the thin edge of molten rock is.
const COOL_EDGE: f32 = 0.45;

/// Linear colour of the crust that floats where water would carry foam.
const CRUST: Vec3 = Vec3::new(0.05, 0.04, 0.04);

/// What the water shader needs to know about the liquid it draws.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Liquid {
    /// Linear colour over a shallow floor.
    pub shallow: Vec3,
    /// Linear colour over a deep floor.
    pub deep: Vec3,
    /// Linear colour of what floats along the shore and on the crests.
    pub foam: Vec3,
    /// Light the liquid gives off by itself, in units of full daylight. Water gives none.
    pub glow: f32,
    /// Share of the sky reflection and of the glints the surface keeps, 1 for water.
    pub mirror: f32,
    /// Least opacity of the surface, 0 for clear water and 1 for a liquid nothing shows through.
    pub cover: f32,
    /// False on a planet without a sea, where no liquid surface is drawn at all.
    pub shown: bool,
}

impl Liquid {
    /// The water of the home planet, everywhere.
    pub const WATER: Liquid = Liquid {
        shallow: Vec3::new(0.03, 0.55, 0.48),
        deep: Vec3::new(0.002, 0.04, 0.16),
        foam: Vec3::new(0.95, 0.98, 1.0),
        glow: 0.0,
        mirror: 1.0,
        cover: 0.0,
        shown: true,
    };

    /// Returns the liquid of `globe`: water where the class has water, else the colour of its look, which
    /// shines by itself when it is molten rock.
    pub fn of(globe: &Globe) -> Self {
        let lands = globe.lands();
        let water = Liquid {
            shown: lands.has_sea(),
            ..Self::WATER
        };
        if lands.watery() {
            return water;
        }
        let look = globe.record().class.look();
        let molten = look.glow > 0.0;
        Liquid {
            shallow: look.sea * [1.0, COOL_EDGE][usize::from(molten)],
            deep: look.sea * [DEPTH_SHADE, 1.0][usize::from(molten)],
            foam: [Self::WATER.foam, CRUST][usize::from(molten)],
            glow: look.glow,
            mirror: [1.0, DULL][usize::from(molten)],
            cover: [0.5, 1.0][usize::from(molten)],
            ..water
        }
    }

    /// Returns the linear light the open liquid gives off by itself, none for water.
    pub fn ember(&self) -> Vec3 {
        self.deep * self.glow
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::universe::{PlanetClass, PlanetRecord};

    /// Returns the liquid of a globe of `class` whose sea covers the `ocean` share of it.
    fn liquid(class: PlanetClass, ocean: f64) -> Liquid {
        let record = PlanetRecord {
            class,
            ocean,
            ..PlanetRecord::TEMPERATE
        };
        Liquid::of(&Globe::new(3, 3000.0, 9.8).with_record(record))
    }

    /// The home planet, and every world of water, is drawn with the water the shader always had.
    #[test]
    fn water_worlds_keep_their_water() {
        assert_eq!(Liquid::of(&Globe::new(3, 3000.0, 9.8)), Liquid::WATER);
        for class in [PlanetClass::Ocean, PlanetClass::Island] {
            assert_eq!(liquid(class, 0.8), Liquid::WATER);
        }
    }

    /// Molten rock has the colour of its look, shines by itself and hides its floor. Water does not shine.
    #[test]
    fn molten_rock_shines_by_itself() {
        let lava = liquid(PlanetClass::Lava, 0.3);
        let look = PlanetClass::Lava.look();
        assert_eq!((lava.deep, lava.glow), (look.sea, look.glow));
        assert!(lava.glow > 0.0 && lava.cover == 1.0 && lava.mirror < 0.5);
        assert!(lava.deep.x > 2.0 * lava.deep.y && lava.deep.y > lava.deep.z);
        for class in PlanetClass::ALL {
            let shines = liquid(class, 0.5).glow > 0.0;
            assert_eq!(shines, class == PlanetClass::Lava, "{class:?}");
        }
    }

    /// A dry world draws no liquid surface, whatever its class, and a world with a sea draws it.
    #[test]
    fn a_dry_world_draws_no_sea() {
        for class in PlanetClass::ALL {
            assert!(
                !liquid(class, 0.0).shown,
                "{class:?} draws a sea it has not"
            );
            assert!(liquid(class, 0.4).shown);
        }
        assert_eq!(liquid(PlanetClass::Desert, 0.2), Liquid::WATER);
    }
}
