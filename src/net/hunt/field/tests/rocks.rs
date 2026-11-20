//! The pieces of rocks on the network: what goes out and what is quarried when it comes back.

use abstractions_net::wire::{Act, Rock};

use super::rig::{acts, app, written};
use crate::net::hunt::Echoes;
use crate::universe::BodyId;
use crate::world::rocks::{RockId, RockTaken, Taken};

/// A piece of a rock goes out with the mass of the whole rock, and a piece told by the server is
/// quarried in the same place, told to the world and not sent again.
#[test]
fn a_rock_piece_goes_out_and_the_one_that_comes_back_is_quarried() {
    let (mut app, script) = app();
    let id = RockId(77);
    app.world_mut().resource_mut::<Taken>().quarry(id, 0.6, 1.0);
    let piece = RockTaken {
        planet: BodyId::default(),
        id,
        kilograms: 0.6,
    };
    app.world_mut().write_message(piece);
    app.update();
    let all = acts(&script);
    let [Act::Rock(sent)] = all.as_slice() else {
        panic!("one rock");
    };
    assert_eq!((sent.id, sent.grams, sent.whole), (77, 600, 1000));
    let told = Rock {
        id: 5,
        grams: 250,
        whole: 900,
        ..*sent
    };
    app.world_mut().resource_mut::<Echoes>().rocks.push(told);
    app.update();
    let taken = app.world().resource::<Taken>();
    assert!((taken.kilograms(RockId(5)) - 0.25).abs() < 1e-6);
    assert_eq!(taken.whole(RockId(5)), Some(0.9));
    assert_eq!(written::<RockTaken>(&app).len(), 2);
    assert_eq!(acts(&script).len(), 1);
}

/// What the server holds of a rock on joining only tops up what this world already took of it.
#[test]
fn the_total_of_a_rock_tops_up_what_was_taken() {
    let (mut app, _) = app();
    let id = RockId(9);
    app.world_mut().resource_mut::<Taken>().quarry(id, 0.2, 1.0);
    let total = Rock {
        planet: BodyId::default(),
        id: 9,
        grams: 500,
        whole: 1000,
    };
    app.world_mut().resource_mut::<Echoes>().totals.push(total);
    app.update();
    assert!((app.world().resource::<Taken>().kilograms(id) - 0.5).abs() < 1e-6);
}
