//! An item as the bytes that travel: a crafted tool recipe or a tagged firearm `[0xFF, kind, barrel]`,
//! and the tool or remote gun it rebuilds on the world underfoot.

use abstractions_net::wire::{ITEM_MAX, decode, encode};

use crate::avatar::strike::{Held, RemoteGun};
use crate::craft::build::{Place, Recipe};
use crate::craft::gun::{Gun, GunKind};
use crate::craft::matter::{Mineral, Stuff};
use crate::life::Tissue;
use crate::world::{Globe, Terrain};

/// Leading byte tag marking a held firearm payload `[0xFF, kind, barrel]`.
pub const GUN_TAG: u8 = 0xFF;

/// Encodes `gun` as a 3-byte held firearm tag `[0xFF, kind, barrel]`.
pub fn pack_gun(gun: &Gun) -> Vec<u8> {
    let barrel = Mineral::ALL
        .iter()
        .position(|m| *m == gun.barrel)
        .unwrap_or(0) as u8;
    vec![GUN_TAG, gun.kind as u8, barrel]
}

/// Decodes a 3-byte held firearm tag `[0xFF, kind, barrel]` into a [`RemoteGun`].
pub fn unpack_gun(bytes: &[u8]) -> Option<RemoteGun> {
    let &[GUN_TAG, kind, barrel] = bytes else {
        return None;
    };
    let kind = match kind {
        0 => GunKind::FlintlockPistol,
        1 => GunKind::MineralRifle,
        _ => return None,
    };
    let barrel = Mineral::ALL.get(usize::from(barrel)).copied()?;
    Some(RemoteGun { kind, barrel })
}

/// Returns the bytes of `recipe`, or nothing when they do not fit [`ITEM_MAX`].
pub fn pack(recipe: &Recipe) -> Option<Vec<u8>> {
    let bytes = encode(recipe).ok()?;
    (bytes.len() <= ITEM_MAX).then_some(bytes)
}

/// Returns the tissue of `stuff` on `globe` when it came from this planet, and nothing for stone
/// or for a tissue of another planet, which this machine cannot read.
fn tissue_of<'a>(stuff: &Stuff, globe: &'a Globe) -> Option<&'a Tissue> {
    let Stuff::Tissue {
        planet,
        species,
        tissue,
        ..
    } = stuff
    else {
        return None;
    };
    let species = globe.life()?.species.get(usize::from(*species))?;
    species
        .tissues
        .get(usize::from(*tissue))
        .filter(|_| *planet == globe.seed())
}

/// Returns the tool in the hands the item of `bytes` makes on the world of `terrain`, or nothing
/// when the bytes are no recipe or a material of it cannot be read here.
pub fn unpack(bytes: &[u8], terrain: &Terrain) -> Option<Held> {
    let (recipe, _): (Recipe, usize) = decode(bytes, ITEM_MAX).ok()??;
    let globe = terrain.globe();
    let build = recipe.build(|stuff| stuff.material(tissue_of(stuff, globe)))?;
    let place = Place::on(globe.gravity(), globe.record());
    Some(Held::new(recipe, build, place))
}
