//! The take of the lift nozzles over the sea for the owner: a ship hovering 5 metres over the
//! water, heard from 20 metres.
//!
//! It runs only when `SHIP_AUDIO_DIR` names the folder to write to:
//! `SHIP_AUDIO_DIR=/tmp/ship-audio cargo test --features dev --lib -- audio::tests::render --nocapture`.

use std::path::Path;

use super::render::write;
use super::spray::hover;
use crate::audio::dsp::limit::CEILING;

/// Renders the hover over the sea into the folder `SHIP_AUDIO_DIR` names, and does nothing without
/// it. The take stays within full scale and is heard.
#[test]
fn water_take_renders() {
    let Some(dir) = std::env::var_os("SHIP_AUDIO_DIR") else {
        return;
    };
    std::fs::create_dir_all(&dir).expect("the folder can be made");
    let sound = hover();
    let peak = sound.iter().flatten().fold(0.0_f32, |p, x| p.max(x.abs()));
    assert!(peak <= CEILING && peak > 1.0e-3, "{peak}");
    let path = Path::new(&dir).join("45_jet_over_water.wav");
    write(&path, &sound).expect("the file can be written");
    println!("{}: peak {peak:.4}", path.display());
}
