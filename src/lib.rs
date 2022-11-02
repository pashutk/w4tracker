#[cfg(feature = "buddy-alloc")]
mod alloc;
mod inputs;
mod instrument;
mod navigation;
mod notes;
mod pattern;
mod render;
mod screen;
mod timers;
mod tracker;
mod wasm4;
mod wtime;

use inputs::Inputs;
use pattern::prepare_pattern_screen;
use screen::Screen;
use timers::TIMERS;
use tracker::{Tracker, TRACKER};
use wtime::Winstant;

static mut INPUTS: Inputs = Inputs::new();

#[no_mangle]
unsafe fn start() {
    TRACKER = Tracker::new();
    TIMERS.init();
    prepare_pattern_screen();
}

#[no_mangle]
fn update() {
    let tracker: &Tracker;
    unsafe {
        tracker = &TRACKER;
    };

    match tracker.screen() {
        Screen::Pattern => render::pattern_screen(tracker),
        Screen::Instrument => render::instrument_screen(tracker),
        _ => render::not_implemented_screen(),
    }

    unsafe {
        TRACKER.update();
        INPUTS.tick();
    }

    Winstant::tick();
}
