//! Multiplayer menu tests: rows built from the settings and the lobby, the status of each row,
//! the edits of the saved servers and what the form shows.

use super::form_view::{shown, texts};
use super::joining::connecting;
use super::rows::{Part, cell_text, listed, status_text};
use super::{Field, Form, FormKind, Mode, Pane, Section, Servers, ops};
use crate::net::{Lobby, RowStatus, ServerRow, Session};
use crate::settings::{SavedServer, default_servers};
use crate::ui::{Tone, theme};

/// Returns a saved server.
fn saved(name: &str, address: &str) -> SavedServer {
    SavedServer {
        name: name.to_string(),
        address: address.to_string(),
    }
}

/// Returns a server that answered with `players` of 16 and a ping of 42 ms.
fn online(players: u8, compatible: bool) -> RowStatus {
    RowStatus::Online {
        players,
        cap: 16,
        ping_ms: 42,
        version: "0.9.0".to_string(),
        compatible,
    }
}

/// Returns the rows of the public server, a saved server with no name and a network game.
fn rows() -> Vec<super::Listed> {
    let lobby = Lobby {
        saved: vec![ServerRow {
            name: "Space".to_string(),
            address: "space.facile.studio".to_string(),
            status: online(3, true),
            key: None,
        }],
        lan: vec![ServerRow {
            name: "Ada's world".to_string(),
            address: "192.168.1.20:24650".to_string(),
            status: online(1, true),
            key: Some([7; 32]),
        }],
    };
    let mut mine = default_servers();
    mine.push(saved("", "10.0.0.2"));
    listed(&mine, &lobby)
}

/// Saved servers come first with the status the lobby has for their address, then the network
/// games with their beacon key. A server the lobby does not know yet is pinging.
#[test]
fn rows_join_the_settings_and_the_lobby() {
    let rows = rows();
    let sections: Vec<Section> = rows.iter().map(|row| row.section).collect();
    assert_eq!(
        sections,
        [Section::Saved(0), Section::Saved(1), Section::Lan]
    );
    assert_eq!(rows[0].status, online(3, true));
    assert_eq!(rows[1].status, RowStatus::Pinging);
    assert_eq!((rows[2].key, rows[1].key), (Some([7; 32]), None));
    assert_eq!(cell_text(&rows[1], Part::Name).0, "10.0.0.2");
    assert_eq!(cell_text(&rows[0], Part::Name).0, "Space");
    assert_eq!(cell_text(&rows[0], Part::Address).1, Tone::Muted);
}

/// The selection knows the section of its row and stays inside the list.
#[test]
fn the_selection_knows_its_section() {
    let mut servers = Servers {
        list: rows(),
        cursor: 2,
    };
    assert_eq!((servers.shape(), servers.saved()), ((2, 1), None));
    servers.cursor = 1;
    assert_eq!(servers.saved(), Some(1));
    servers.fill(Vec::new());
    assert_eq!((servers.cursor, servers.selected()), (0, None));
}

/// Each status reads as the menu shows it, on the right of its row.
#[test]
fn every_status_has_its_text() {
    let cases = [
        (RowStatus::Pinging, "Pinging", Tone::Muted),
        (online(3, true), "3/16  42 ms", Tone::Plain),
        (online(3, false), "Needs v0.9.0", Tone::Muted),
        (RowStatus::Offline, "Offline", Tone::Muted),
        (RowStatus::KeyChanged, "Key changed", Tone::Danger),
    ];
    for (status, text, tone) in cases {
        assert_eq!(status_text(&status), (text.to_string(), tone));
    }
}

/// A server needs an address and may have no name; edits replace in place, adds append, and
/// removing past the end does nothing.
#[test]
fn saved_server_edits() {
    assert_eq!(
        ops::server("  ", "   "),
        Err("A server needs an address".to_string())
    );
    assert_eq!(
        ops::server(" Home ", " 10.0.0.2 "),
        Ok(saved("Home", "10.0.0.2"))
    );
    assert!(ops::player("  ").is_err());
    assert_eq!(ops::player(" Ada "), Ok("Ada".to_string()));
    let mut servers = default_servers();
    assert_eq!(ops::put(&mut servers, None, saved("", "10.0.0.2")), 1);
    assert_eq!(ops::put(&mut servers, Some(0), saved("Main", "a.b")), 0);
    assert_eq!(ops::put(&mut servers, Some(9), saved("", "c.d")), 2);
    assert_eq!(servers[0], saved("Main", "a.b"));
    assert!(ops::remove(&mut servers, 1));
    assert!(!ops::remove(&mut servers, 5));
    assert_eq!(servers, [saved("Main", "a.b"), saved("", "c.d")]);
}

/// Each form draws the fields it asks for, types into one at a time and shows a hint in an
/// empty field.
#[test]
fn forms_draw_their_fields() {
    let mut add = Form::new(FormKind::Add, String::new(), String::new());
    add.type_text(false, "Home");
    add.switch();
    add.type_text(false, "10.0.0.2x");
    add.type_text(true, "");
    assert_eq!(
        (add.name.as_str(), add.address.as_str()),
        ("Home", "10.0.0.2")
    );
    let mut direct = Form::new(FormKind::Direct, String::new(), String::new());
    direct.switch();
    assert_eq!(direct.field, Field::Address);
    let player = Mode::Form(Form::new(FormKind::Player, String::new(), String::new()));
    assert!(shown(Pane::Name, &player) && !shown(Pane::Address, &player));
    assert!(!shown(Pane::List, &player) && shown(Pane::Form, &player));
    assert!(!shown(Pane::Name, &Mode::Form(direct)));
    assert!(shown(Pane::List, &Mode::Browse) && !shown(Pane::Form, &Mode::Browse));
    let typing = texts(&add, Field::Address);
    assert_eq!((typing.0, typing.1.as_str()), ("Add a server", "10.0.0.2_"));
    let empty = Form::new(FormKind::Edit(0), String::new(), String::new());
    assert_eq!(texts(&empty, Field::Address).2, theme::MUTED);
    assert_eq!(
        connecting(&Session::Joining {
            address: "space.facile.studio".to_string()
        }),
        "Connecting to space.facile.studio"
    );
}
