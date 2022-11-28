use std::{borrow::Borrow, time::Duration, vec};

use crate::{
    instrument::add_instrument_screen_handlers,
    pattern::add_pattern_screen_handlers,
    screen::{Screen, Screens},
    song::add_song_screen_handlers,
    timers::TIMERS,
    tracker::TRACKER,
    unique_usize::get_unique_usize,
    wasm4::trace,
    wtime::Winstant,
    INPUTS,
};

pub unsafe fn go_to_pattern_screen() {
    TRACKER.set_screens(Screens::Single(Screen::Pattern));
    INPUTS.unlisten();
    add_pattern_screen_handlers(&mut INPUTS);
}

const ANIM_DURATION: f32 = 300.0;

pub unsafe fn go_to_instrument_screen(from: Screen) {
    TRACKER.set_screens(Screens::Transition(from, Screen::Instrument, 0.0));

    let start = Winstant::now();

    let interval_id = get_unique_usize();
    TIMERS.set_interval(
        interval_id,
        move || {
            let now = Winstant::now();
            let since_start = now.duration_since(start);
            let progress: f32 = since_start.as_millis() as f32 / ANIM_DURATION;
            if progress < 1.0 {
                TRACKER.set_screens(Screens::Transition(from, Screen::Instrument, progress));
            } else {
                TRACKER.set_screens(Screens::Single(Screen::Instrument));
                TIMERS.cancel_interval(interval_id);
            }
        },
        1,
    );

    INPUTS.unlisten();
    add_instrument_screen_handlers(&mut INPUTS);
}

pub unsafe fn go_to_song_screen() {
    TRACKER.set_screens(Screens::Single(Screen::Song));
    INPUTS.unlisten();
    add_song_screen_handlers(&mut INPUTS);
}
