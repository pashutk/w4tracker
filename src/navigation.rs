use std::{borrow::Borrow, time::Duration};

use crate::{
    instrument::add_instrument_screen_handlers,
    pattern::add_pattern_screen_handlers,
    screen::{Screen, Screens},
    song::add_song_screen_handlers,
    timers::TIMERS,
    tracker::TRACKER,
    wasm4::trace,
    wtime::Winstant,
    INPUTS,
};

pub unsafe fn go_to_pattern_screen() {
    TRACKER.set_screens(Screens::Single(Screen::Pattern));
    INPUTS.unlisten();
    add_pattern_screen_handlers(&mut INPUTS);
}

pub unsafe fn go_to_instrument_screen(from: Screen) {
    TRACKER.set_screens(Screens::Transition(from, Screen::Instrument));
    // let mut interval = None;
    let start = Winstant::now();
    let mut x: Option<usize> = None;
    x = Some(TIMERS.set_interval(
        || {
            let now = Winstant::now();
            trace("action");
            if start + Duration::from_millis(200) > now {
                trace(now.duration_since(start).as_millis().to_string())
            } else if let Some(a) = x {
                TIMERS.cancel_interval(a);
            }
            // if start
            // if let Some(id) = interval {}
        },
        1,
    ));

    INPUTS.unlisten();
    add_instrument_screen_handlers(&mut INPUTS);
}

pub unsafe fn go_to_song_screen() {
    TRACKER.set_screens(Screens::Single(Screen::Song));
    INPUTS.unlisten();
    add_song_screen_handlers(&mut INPUTS);
}
