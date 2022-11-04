#[cfg(feature = "buddy-alloc")]
mod alloc;
mod channel;
mod inputs;
mod instrument;
mod navigation;
mod notes;
mod pattern;
mod render;
mod screen;
mod song;
mod timers;
mod tracker;
mod wasm4;
mod wtime;

use inputs::Inputs;
use pattern::add_pattern_screen_handlers;
use screen::Screen;
// use song::add_song_screen_handlers;
use timers::TIMERS;
use tracker::{Tracker, TRACKER};
use wtime::Winstant;

static mut INPUTS: Inputs = Inputs::new();

#[no_mangle]
unsafe fn start() {
    TRACKER = Tracker::new();
    TIMERS.init();
    add_pattern_screen_handlers(&mut INPUTS);
    // TRACKER.set_screen(Screen::Song);
    // add_song_screen_handlers(&mut INPUTS)
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
        Screen::Song => render::song_screen(tracker),
        _ => render::not_implemented_screen(),
    }

    unsafe {
        TRACKER.update();
        INPUTS.tick();
    }

    Winstant::tick();
}
