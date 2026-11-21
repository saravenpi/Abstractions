//! Crafting: what harvested tissues and stones are made of, and the tools built from them.
//!
//! [`build`] and [`matter`] are pure model code: plain data and functions of the planet and the
//! materials a player carries, so a server can run the same numbers and agree with every client.
//! [`pack`] holds what the robot carries, [`harvest`] takes tissue from a dead animal, [`wield`] is
//! the held tool, and [`ui`] is the pack screen, the workshop and the hotbar.

pub mod build;
pub mod fall;
pub mod harvest;
pub mod matter;
pub mod mend;
pub mod pack;
#[cfg(test)]
mod stage;
#[cfg(test)]
mod tests_two;
pub mod ui;
pub mod wield;

use bevy::prelude::*;

pub use pack::{Burden, Lent, Out, Pack};
pub use wield::{Held, Hold, Worn};

/// Registers the crafting area: the pack and its save, the held tool and its wear, harvesting and
/// the screens.
pub(super) fn plugin(app: &mut App) {
    let changed = resource_changed::<Pack>.or_else(resource_changed::<crate::condition::Condition>);
    crate::player::interacting(app);
    app.init_resource::<Pack>()
        .init_resource::<Burden>()
        .init_resource::<crate::condition::Condition>()
        .init_resource::<Held>()
        .add_message::<Worn>()
        .add_message::<harvest::Harvest>()
        .init_resource::<pack::Notice>()
        .add_message::<pack::Knock>()
        .add_message::<pack::Knocked>()
        .add_message::<crate::audio::Impact>()
        .add_message::<crate::effects::Fling>()
        .add_systems(
            Update,
            (
                pack::load,
                pack::gather.run_if(crate::screens::in_control.and_then(crate::space::on_foot)),
                pack::split.run_if(on_message::<pack::Knocked>),
                pack::fade.run_if(pack::fading),
                harvest::reap.run_if(on_message::<harvest::Harvest>),
                wield::wear.run_if(on_message::<Worn>),
                (pack::store, pack::weigh, wield::hold).run_if(changed),
            )
                .chain(),
        );
    fall::plugin(app);
    mend::plugin(app);
    ui::plugin(app);
}
