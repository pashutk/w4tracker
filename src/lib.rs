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
mod unique_usize;
mod wasm4;
mod wtime;

use inputs::Inputs;
use pattern::add_pattern_screen_handlers;
use render::render_screens;
// use song::add_song_screen_handlers;
use timers::TIMERS;
use tracker::{Tracker, TRACKER};
use wtime::Winstant;

static mut INPUTS: Inputs = Inputs::new();

#[no_mangle]
unsafe fn start() {
    TRACKER = Tracker::restore();
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

    render_screens(tracker.screens(), tracker);

    unsafe {
        TRACKER.update();
        INPUTS.tick();
        TIMERS.tick();
    }

    Winstant::tick();
}
