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
use render::render_screen;
use screen::Screen;
// use song::add_song_screen_handlers;
use timers::TIMERS;
use tracker::{Tracker, TRACKER};
use wasm4::SCREEN_SIZE;
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

    match tracker.screens() {
        screen::Screens::Single(screen) => render_screen(screen, tracker, 0, 0),
        screen::Screens::Transition(from, to, progress) => {
            let ssf: f32 = SCREEN_SIZE as f32;
            let corrected_progress: f32 = 1.0 - (1.0 - progress).powf(4.0);
            let d = (corrected_progress * ssf) as i32;
            let x_from = d * -1;
            let x_to = SCREEN_SIZE as i32 - d;
            render_screen(from, tracker, x_from, 0);
            render_screen(to, tracker, x_to, 0);
        }
    }

    unsafe {
        TRACKER.update();
        INPUTS.tick();
        TIMERS.tick();
    }

    Winstant::tick();
}
