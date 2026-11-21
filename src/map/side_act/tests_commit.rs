//! Go on a pin: on its own planet it is the active pin, from anywhere else the course is the planet and
//! no pin is active, so the course ends in orbit and never waits for a spot on the surface.

use super::commit;
use crate::computer::Target;
use crate::map::places::Places;
use crate::saves::Pin;
use crate::universe::Universe;

/// Returns a pin on the home planet of seed 3 and the places that hold it.
fn kept() -> (Places, Pin) {
    let body = Universe::new(3).home();
    let pin = Pin {
        name: "Camp".to_string(),
        body,
        dir: [1.0, 0.0, 0.0],
        height: None,
    };
    let places = Places {
        pins: vec![pin.clone()],
        active: Some(0),
        ..Places::default()
    };
    (places, pin)
}

/// From another planet, Go sets the course to the planet and lets go of the active pin.
#[test]
fn from_afar_the_course_is_the_planet_and_no_pin_leads() {
    let (mut places, pin) = kept();
    let target = commit(&mut places, (0, &pin), None);
    assert_eq!(target, Some(Target::Body(pin.body)));
    assert_eq!(places.active, None);
}

/// On the planet of the pin, Go makes it the active pin and sets no course.
#[test]
fn on_its_planet_the_pin_leads() {
    let (mut places, pin) = kept();
    places.set_active(None);
    let target = commit(&mut places, (0, &pin), Some(pin.body));
    assert_eq!(target, None);
    assert_eq!(places.active, Some(0));
}
