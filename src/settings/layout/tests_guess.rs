//! Tests of the layout guessed before the first key press.

use super::guess::{guess, of_locale, of_xkb};
use super::known::Layout;

/// Returns a reader of the variables `set`.
fn env<'a>(set: &'a [(&'a str, &'a str)]) -> impl Fn(&str) -> Option<String> + 'a {
    move |name| {
        let found = set.iter().find(|(key, _)| *key == name);
        found.map(|(_, value)| value.to_string())
    }
}

/// The XKB names of the layouts and of the variants that move the letters are known, and an
/// unknown one tells nothing.
#[test]
fn xkb_names_give_their_layout() {
    assert_eq!(of_xkb("fr", ""), Some(Layout::Azerty));
    assert_eq!(of_xkb("fr,us", "bepo,"), Some(Layout::Bepo));
    assert_eq!(of_xkb("us", "dvorak"), Some(Layout::Dvorak));
    assert_eq!(of_xkb("us", "colemak_dh"), Some(Layout::Colemak));
    assert_eq!(of_xkb("de", "nodeadkeys"), Some(Layout::Qwertz));
    assert_eq!(of_xkb("ch", "fr"), Some(Layout::Swiss));
    assert_eq!(of_xkb("no", ""), Some(Layout::Norwegian));
    assert_eq!(of_xkb("", ""), None);
    assert_eq!(of_xkb("jp", ""), None);
}

/// A language gives its usual keyboard, a country can change it, and a language that only
/// starts like another is not mistaken for it.
#[test]
fn locales_give_the_usual_keyboard() {
    assert_eq!(of_locale("fr_FR.UTF-8"), Some(Layout::Azerty));
    assert_eq!(of_locale("fr_BE@euro"), Some(Layout::Azerty));
    assert_eq!(of_locale("fr_CA.UTF-8"), Some(Layout::Us));
    assert_eq!(of_locale("de_DE"), Some(Layout::Qwertz));
    assert_eq!(of_locale("de_CH.UTF-8"), Some(Layout::Swiss));
    assert_eq!(of_locale("en_GB.UTF-8"), Some(Layout::Uk));
    assert_eq!(of_locale("en_US.UTF-8"), Some(Layout::Us));
    assert_eq!(of_locale("sv_SE"), Some(Layout::Swedish));
    assert_eq!(of_locale("es_MX.UTF-8"), None);
    assert_eq!(of_locale("fil_PH"), None, "not Finnish");
    assert_eq!(of_locale("C"), None);
    assert_eq!(of_locale(""), None);
}

/// The session layout wins over the keyboard file, which wins over the language, and with
/// nothing known the guess is QWERTY.
#[test]
fn the_guess_reads_the_session_then_the_file_then_the_language() {
    let file =
        "# KEYBOARD CONFIGURATION FILE\nXKBMODEL=\"pc105\"\nXKBLAYOUT=\"fr\"\nXKBVARIANT=\"\"\n";
    let english = [("LANG", "en_US.UTF-8")];
    assert_eq!(guess(env(&english), Some(file)), Layout::Azerty);
    assert_eq!(guess(env(&english), None), Layout::Us);
    let session = [("XKB_DEFAULT_LAYOUT", "de"), ("LANG", "fr_FR.UTF-8")];
    assert_eq!(guess(env(&session), Some(file)), Layout::Qwertz);
    let french = [("LC_ALL", "C"), ("LANG", "fr_FR.UTF-8")];
    assert_eq!(guess(env(&french), None), Layout::Azerty);
    let bepo = "XKBLAYOUT=fr\nXKBVARIANT=bepo\n";
    assert_eq!(guess(env(&[]), Some(bepo)), Layout::Bepo);
    assert_eq!(guess(env(&[]), None), Layout::Us);
}
