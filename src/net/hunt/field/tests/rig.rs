//! The headless app of the field tests: the field acts, a scripted link and a place on a planet.

use std::sync::{Arc, Mutex};

use abstractions_net::log::PlayerId as Wired;
use abstractions_net::wire::{Act, ClientMsg};
use bevy::math::{DQuat, DVec3};
use bevy::prelude::*;

use super::super::codec::{tool_code, tool_of};
use super::super::plugin;
use crate::audio::{Body, Impact, Outer};
use crate::net::hunt::{Echoes, Hunted};
use crate::net::link::{Link, Role, Script, Source, Wire};
use crate::space::{Anchor, Clock, Frame, Location, Travel};
use crate::universe::BodyId;
use crate::world::rocks::Taken;

/// Returns an app on the ground of a planet with the field acts, a scripted link the server
/// welcomed as player 3, and the script.
pub fn app() -> (App, Script) {
    let body = BodyId::default();
    let script: Script = Arc::new(Mutex::new(Wire::default()));
    let role = Role::Join {
        address: "friends.example:24650".to_string(),
        expected: None,
    };
    let mut link = Link::over(Source::Script(script.clone()), role);
    link.you = Some(Wired(3));
    let anchor = Anchor {
        frame: Frame::Surface(body),
        origin: DVec3::new(0.0, 6_000_000.0, 0.0),
        rotation: DQuat::IDENTITY,
        flat: true,
    };
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<Echoes>()
        .init_resource::<Hunted>()
        .init_resource::<Taken>()
        .init_resource::<Clock>()
        .insert_resource(anchor)
        .insert_resource(Location {
            body,
            travel: Travel::OnFoot,
        })
        .insert_resource(link)
        .add_plugins(plugin);
    app.update();
    (app, script)
}

/// Returns the acts the game sent through the link.
pub fn acts(script: &Script) -> Vec<Act> {
    let sent = script.lock().unwrap().sent.clone();
    sent.into_iter()
        .filter_map(|msg| match msg {
            ClientMsg::Hunt(act) => Some(act),
            _ => None,
        })
        .collect()
}

/// Returns every message of type `M` written so far.
pub fn written<M: Message + Clone>(app: &App) -> Vec<M> {
    let all = app.world().resource::<Messages<M>>();
    all.get_cursor().read(all).cloned().collect()
}

/// Returns an impact on the ground, as a swing leaves it.
pub fn impact() -> Impact {
    Impact {
        at: Vec3::new(1.0, 2.0, 3.0),
        joules: 40.0,
        cut: true,
        outer: Outer::Bone,
        struck: Body {
            stiffness: 18.0,
            mass: 10.0,
        },
        tool: tool_of(tool_code(&Body {
            stiffness: 70.0,
            mass: 0.5,
        })),
        deflected: false,
    }
}
