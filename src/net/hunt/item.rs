//! An item as the bytes that travel: the craft recipe in the codec of the streams, and the tool it
//! rebuilds on the world underfoot.

use abstractions_net::wire::{ITEM_MAX, decode, encode};

use crate::avatar::strike::Held;
use crate::craft::build::{Place, Recipe};
use crate::craft::matter::Stuff;
use crate::life::Tissue;
use crate::world::{Globe, Terrain};

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
