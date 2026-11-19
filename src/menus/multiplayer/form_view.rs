//! The look of the form of the multiplayer menu: a title and fields like those of the new world
//! menu, drawn in place of the list while a form is filled.

use bevy::prelude::*;

use super::{Field, Form, FormKind, Mode, Pane};
use crate::menus::name::with_caret;
use crate::ui::{caps, column, hairline, text, theme};

/// Marks the line that says what the form asks for.
#[derive(Component)]
pub(super) struct FormTitle;

/// Returns whether `pane` is drawn in `mode`.
pub(super) fn shown(pane: Pane, mode: &Mode) -> bool {
    match (pane, mode) {
        (Pane::List, Mode::Browse) => true,
        (Pane::Form, Mode::Form(_)) => true,
        (Pane::Name, Mode::Form(form)) => {
            !matches!(form.kind, FormKind::Direct | FormKind::Password)
        }
        (Pane::Address, Mode::Form(form)) => {
            !matches!(form.kind, FormKind::Player | FormKind::Password)
        }
        (Pane::Password, Mode::Form(form)) => form.kind == FormKind::Password,
        _ => false,
    }
}

/// Returns the line over the form, and the text and colour of `field`.
pub(super) fn texts(form: &Form, field: Field) -> (&'static str, String, Color) {
    let title = match form.kind {
        FormKind::Player => "Choose the name other players see",
        FormKind::Direct => "Join a server once",
        FormKind::Add => "Add a server",
        FormKind::Edit(_) => "Edit the server",
        FormKind::Password => "This server needs a password",
    };
    let hidden = "*".repeat(form.name.chars().count());
    let value = match field {
        Field::Name if form.kind == FormKind::Password => &hidden,
        Field::Name => &form.name,
        Field::Address => &form.address,
        Field::Password => &hidden,
    };
    let hint = match (field, form.kind) {
        (Field::Name, FormKind::Player) => "your name",
        (Field::Name, _) => "optional, the address shows",
        (Field::Address, _) => "domain or IP, port optional",
        (Field::Password, _) => "the password of the server",
    };
    let (shown, color) = match (form.field == field, value.is_empty()) {
        (true, _) => (with_caret(value), theme::TEXT),
        (false, false) => (value.clone(), theme::TEXT),
        (false, true) => (hint.to_string(), theme::MUTED),
    };
    (title, shown, color)
}

/// The name of a field and the underlined row that holds its text.
fn field(pane: Pane, label: &str, kind: Field) -> impl Bundle {
    (
        pane,
        column(percent(100), theme::GAP_XS),
        children![
            caps(label),
            (
                Node {
                    height: px(theme::GAP_XL),
                    align_items: AlignItems::Center,
                    padding: UiRect::horizontal(px(theme::GAP_S)),
                    border: UiRect::bottom(px(theme::LINE)),
                    margin: UiRect::bottom(px(theme::GAP_M)),
                    overflow: Overflow::clip(),
                    ..default()
                },
                hairline(),
                children![(
                    text("", theme::FONT_BODY, theme::TEXT),
                    TextLayout::no_wrap(),
                    kind
                )],
            ),
        ],
    )
}

/// The hidden form: its title and both fields.
pub(super) fn block(width: f32) -> impl Bundle {
    (
        Pane::Form,
        column(px(width), theme::GAP_S),
        children![
            (FormTitle, text("", theme::FONT_BODY, theme::TEXT)),
            field(Pane::Name, "Name", Field::Name),
            field(Pane::Address, "Address", Field::Address),
            field(Pane::Password, "Password", Field::Password),
        ],
    )
}

/// The panes and the texts of the form.
type Shown<'w, 's> = (
    Query<'w, 's, (&'static Pane, &'static mut Node)>,
    Query<'w, 's, (&'static Field, &'static mut Text, &'static mut TextColor), Without<FormTitle>>,
    Query<'w, 's, &'static mut Text, With<FormTitle>>,
);

/// Draws the panes that fit the mode and writes the texts of the form.
pub(super) fn show(mode: Res<Mode>, (mut panes, mut fields, mut titles): Shown) {
    if !mode.is_changed() {
        return;
    }
    for (pane, mut node) in &mut panes {
        let display = if shown(*pane, &mode) {
            Display::Flex
        } else {
            Display::None
        };
        if node.display != display {
            node.display = display;
        }
    }
    let Mode::Form(form) = &*mode else {
        return;
    };
    for (field, mut content, mut tint) in &mut fields {
        let (_, shown, color) = texts(form, *field);
        content.set_if_neq(Text(shown));
        tint.set_if_neq(TextColor(color));
    }
    let (title, _, _) = texts(form, Field::Name);
    for mut line in &mut titles {
        line.set_if_neq(Text(title.to_string()));
    }
}
