//! Minerals: the stones a planet's geology leaves on the ground and underground.

use super::Material;
pub use crate::world::rocks::Mineral;

impl Mineral {
    /// Returns the measured numbers of the stone or ore mineral.
    pub fn material(self) -> Material {
        let [
            density,
            stiffness,
            strength,
            toughness,
            hardness,
            conductivity,
        ] = match self {
            Mineral::Basalt => [3000.0, 60.0, 25.0, 2.0, 6.0, 1.7],
            Mineral::Obsidian => [2400.0, 70.0, 60.0, 0.75, 5.5, 1.2],
            Mineral::Flint => [2600.0, 80.0, 100.0, 1.5, 7.0, 3.0],
            Mineral::Ice => [917.0, 9.0, 1.5, 0.1, 1.5, 2.2],
            Mineral::Quartz => [2650.0, 95.0, 130.0, 1.8, 7.0, 3.2],
            Mineral::Jade => [3300.0, 110.0, 185.0, 4.2, 6.5, 2.8],
            Mineral::Ironstone => [5200.0, 160.0, 210.0, 3.2, 6.5, 12.0],
            Mineral::Malachite => [3950.0, 85.0, 115.0, 2.4, 4.5, 8.0],
            Mineral::Brimstone => [2070.0, 28.0, 45.0, 0.8, 2.5, 0.4],
            Mineral::Siderite => [7800.0, 200.0, 380.0, 6.5, 8.0, 45.0],
            Mineral::Copper => [8960.0, 120.0, 210.0, 4.0, 3.0, 380.0],
            Mineral::Pyrite => [5010.0, 145.0, 140.0, 1.9, 6.5, 19.0],
            Mineral::Galena => [7500.0, 75.0, 110.0, 2.6, 2.5, 18.0],
            Mineral::Cobaltite => [6330.0, 195.0, 310.0, 4.8, 5.5, 22.0],
            Mineral::Cinnabar => [8100.0, 45.0, 70.0, 1.2, 2.5, 3.0],
            Mineral::Amethyst => [2650.0, 105.0, 155.0, 2.1, 7.0, 3.5],
            Mineral::Ilmenite => [4720.0, 190.0, 340.0, 5.5, 6.0, 26.0],
            Mineral::Cryolite => [2970.0, 65.0, 95.0, 1.6, 3.0, 4.0],
        };
        Material {
            density,
            stiffness,
            strength,
            toughness,
            hardness,
            conductivity,
        }
    }
}
