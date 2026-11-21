//! The layout guessed before the first key press, from what the system says of the keyboard.
//!
//! Neither Bevy 0.20 nor winit 0.30 can be asked for the active layout. What the game can read
//! without a new dependency is the environment: the XKB layout a Linux session was started with,
//! the keyboard file of a Debian system, then the language of the session. On Windows and macOS
//! none of them is set, so the guess is QWERTY until the first press of a key that tells.

use super::known::Layout;

/// The file a Debian system keeps its keyboard layout in.
const KEYBOARD_FILE: &str = "/etc/default/keyboard";

/// The variants of an XKB layout that move the letters, whatever the country.
const VARIANTS: [(&str, Layout); 3] = [
    ("dvorak", Layout::Dvorak),
    ("colemak", Layout::Colemak),
    ("bepo", Layout::Bepo),
];

/// The XKB layout names the game knows.
const XKB: [(&str, Layout); 13] = [
    ("us", Layout::Us),
    ("gb", Layout::Uk),
    ("fr", Layout::Azerty),
    ("be", Layout::Azerty),
    ("de", Layout::Qwertz),
    ("at", Layout::Qwertz),
    ("ch", Layout::Swiss),
    ("se", Layout::Swedish),
    ("fi", Layout::Swedish),
    ("no", Layout::Norwegian),
    ("dk", Layout::Danish),
    ("es", Layout::Spanish),
    ("it", Layout::Italian),
];

/// The languages whose usual keyboard the game knows. A name with a country is that language in
/// that country only and comes before the language alone: French is typed on AZERTY, but on
/// QWERTY in Canada.
const LOCALES: [(&str, Layout); 17] = [
    ("fr_CA", Layout::Us),
    ("fr_CH", Layout::Swiss),
    ("de_CH", Layout::Swiss),
    ("it_CH", Layout::Swiss),
    ("nl_BE", Layout::Azerty),
    ("en_GB", Layout::Uk),
    ("es_ES", Layout::Spanish),
    ("fr", Layout::Azerty),
    ("de", Layout::Qwertz),
    ("sv", Layout::Swedish),
    ("fi", Layout::Swedish),
    ("nb", Layout::Norwegian),
    ("nn", Layout::Norwegian),
    ("no", Layout::Norwegian),
    ("da", Layout::Danish),
    ("it", Layout::Italian),
    ("en", Layout::Us),
];

/// Returns the layout of an XKB layout name and its variant, each the first of a list with
/// commas. The variant wins when it moves the letters.
pub(super) fn of_xkb(layout: &str, variant: &str) -> Option<Layout> {
    let first = |list: &str| {
        list.split(',')
            .next()
            .unwrap_or_default()
            .trim()
            .to_string()
    };
    let (layout, variant) = (first(layout), first(variant));
    let moved = VARIANTS.iter().find(|(name, _)| variant.contains(name));
    let plain = XKB.iter().find(|(name, _)| *name == layout);
    moved.or(plain).map(|(_, layout)| *layout)
}

/// Returns the usual layout of a locale such as `fr_FR.UTF-8`.
pub(super) fn of_locale(locale: &str) -> Option<Layout> {
    let tag = locale.split(['.', '@']).next().unwrap_or_default();
    let language = tag.split('_').next().unwrap_or_default();
    let fits = |name: &str| match name.contains('_') {
        true => name == tag,
        false => name == language,
    };
    let found = LOCALES.iter().find(|(name, _)| fits(name));
    found.map(|(_, layout)| *layout)
}

/// Returns the value of the shell variable `name` in the text of a keyboard file, without its
/// quotes.
fn field<'a>(file: &'a str, name: &str) -> Option<&'a str> {
    let value = |line: &'a str| line.trim().strip_prefix(name)?.strip_prefix('=');
    file.lines().find_map(value).map(|v| v.trim_matches('"'))
}

/// Returns the layout that `env`, the variables of the session, and `file`, the text of the
/// keyboard file, tell: the XKB layout of the session, else the layout of the file, else the
/// usual keyboard of the language, else QWERTY.
pub(super) fn guess(env: impl Fn(&str) -> Option<String>, file: Option<&str>) -> Layout {
    let read = |name: &str| env(name).unwrap_or_default();
    let session = of_xkb(&read("XKB_DEFAULT_LAYOUT"), &read("XKB_DEFAULT_VARIANT"));
    let system = file.and_then(|file| {
        of_xkb(
            field(file, "XKBLAYOUT").unwrap_or_default(),
            field(file, "XKBVARIANT").unwrap_or_default(),
        )
    });
    let spoken = ["LC_ALL", "LC_CTYPE", "LANG"]
        .into_iter()
        .find_map(|name| of_locale(&read(name)));
    session.or(system).or(spoken).unwrap_or_default()
}

/// Returns the layout this machine tells, see [`guess`].
pub(super) fn detect() -> Layout {
    let file = std::fs::read_to_string(KEYBOARD_FILE).ok();
    guess(|name| std::env::var(name).ok(), file.as_deref())
}
