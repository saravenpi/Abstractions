//! Computer tests: the sentences of the Go card, the keys they name, and what a press of Go
//! does for each state of the route.

use super::course::{Ask, FLY_HOME, HERE, Press, READY, card_line, hud_line, press};
use super::route::{Block, Keys, RouteStatus, Target};
use crate::universe::Universe;

/// Returns the names of the default keys: the board key F, the rise key Space and the Go key G.
fn keys() -> Keys {
    Keys {
        board: "F".to_string(),
        rise: "Space".to_string(),
        go: "G".to_string(),
    }
}

/// Returns the question of a card for Mars that is not the course, with the player seated, the
/// route idle and no time to tell.
fn ask(keys: &Keys) -> Ask<'_> {
    Ask {
        name: "Mars",
        ours: false,
        here: false,
        seated: true,
        status: RouteStatus::Idle,
        eta: None,
        keys,
        home: false,
    }
}

/// Every reason the drive cannot take the course is one plain sentence with the fix and the
/// keys in the words of the player's bindings.
#[test]
fn the_block_words_are_the_sentences_of_the_spec() {
    let keys = keys();
    let planet = Block::Inside {
        star: false,
        lock: 12,
        tenths: 8,
    };
    let star = Block::Inside {
        star: true,
        lock: 50,
        tenths: 30,
    };
    let said = [Block::OnFoot, Block::Landed, planet, star].map(|block| block.words(&keys));
    assert_eq!(
        said,
        [
            "Board your ship first: walk to the hatch and press F.",
            "Take off first: hold Space.",
            "Too close to the planet: fly away from it, then press G.",
            "Too close to the star: fly away from it, then press G.",
        ]
    );
    let rebound = Keys {
        board: "E".to_string(),
        rise: "Up".to_string(),
        go: "Enter".to_string(),
    };
    assert!(Block::OnFoot.words(&rebound).ends_with("press E."));
    assert_eq!(Block::Landed.words(&rebound), "Take off first: hold Up.");
}

/// The card says how long the trip takes while Go can start, and why it cannot otherwise, with
/// the player on foot before every other reason.
#[test]
fn the_card_line_says_why_go_cannot_start() {
    let keys = keys();
    let base = ask(&keys);
    let timed = Ask {
        eta: Some("about 4 min"),
        ..base
    };
    assert_eq!(card_line(&timed), (true, "About 4 min.".to_string()));
    assert_eq!(card_line(&base), (true, READY.to_string()));
    let here = Ask { here: true, ..base };
    assert_eq!(card_line(&here), (false, HERE.to_string()));
    let landed = RouteStatus::Blocked(Block::Landed);
    let on_foot = Ask {
        seated: false,
        status: landed,
        ..base
    };
    let board = "Board your ship first: walk to the hatch and press F.";
    assert_eq!(card_line(&on_foot), (false, board.to_string()));
    let stuck = Ask {
        status: landed,
        ..base
    };
    assert_eq!(
        card_line(&stuck),
        (false, "Take off first: hold Space.".to_string())
    );
}

/// While the route to the place is flown, the sentence says what the ship does, and Go stays
/// disabled; the button reads Stop. A route to another place does not stop Go for this one.
#[test]
fn the_card_line_follows_the_route() {
    let keys = keys();
    let ours = Ask {
        ours: true,
        ..ask(&keys)
    };
    let steps = [
        (RouteStatus::Climbing, "Climbing out of the air."),
        (RouteStatus::Dropping, "Slowing down."),
        (RouteStatus::Aligning(37), "Turning toward Mars."),
        (RouteStatus::Spooling, "Charging the warp drive."),
        (RouteStatus::InWarp, "Warping to Mars. G stops."),
        (RouteStatus::Arrived, "Arrived at Mars."),
    ];
    for (status, words) in steps {
        let asked = Ask { status, ..ours };
        assert_eq!(card_line(&asked), (false, words.to_string()), "{status:?}");
        assert_eq!(hud_line("Mars", status, &keys), words);
    }
    let other = Ask {
        status: RouteStatus::InWarp,
        eta: Some("about 2 min"),
        ..ask(&keys)
    };
    assert_eq!(card_line(&other), (true, "About 2 min.".to_string()));
    assert_eq!(hud_line("Mars", RouteStatus::Idle, &keys), "");
}

/// A press of Go sends the ship to the picked place, or to the course when nothing is picked,
/// stops the route that is flown, and does nothing for a player on foot or a blocked drive.
#[test]
fn a_press_of_go_starts_stops_or_does_nothing() {
    let universe = Universe::new(7);
    let (mars, sol) = (
        Target::Body(universe.home()),
        Target::Star(universe.home().star),
    );
    let (idle, warp) = (RouteStatus::Idle, RouteStatus::InWarp);
    let landed = RouteStatus::Blocked(Block::Landed);
    let cases = [
        ((None, None, idle, true), Press::Nothing),
        ((Some(mars), None, idle, true), Press::Go(mars)),
        ((None, Some(mars), idle, true), Press::Go(mars)),
        ((None, Some(mars), warp, true), Press::Stop),
        ((Some(mars), Some(mars), warp, true), Press::Stop),
        ((Some(sol), Some(mars), warp, true), Press::Go(sol)),
        ((Some(mars), None, idle, false), Press::Nothing),
        ((Some(mars), Some(mars), landed, true), Press::Nothing),
        ((Some(mars), Some(mars), warp, false), Press::Stop),
    ];
    for ((pick, course, status, can), wanted) in cases {
        assert_eq!(press(pick, course, status, can), wanted, "{status:?}");
    }
}

/// The card of the home planet offers Fly home to a seated player, on the planet itself too, and
/// asks a player on foot away from it to board first.
#[test]
fn the_home_planet_card_flies_home() {
    let keys = keys();
    let home = Ask {
        home: true,
        here: true,
        ..ask(&keys)
    };
    assert_eq!(card_line(&home), (true, FLY_HOME.to_string()));
    let on_foot = Ask {
        seated: false,
        here: false,
        ..home
    };
    assert!(card_line(&on_foot).1.starts_with("Board your ship first"));
}
