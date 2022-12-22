use std::{time::Duration};

use crate::{
    instrument::add_instrument_screen_handlers,
    pattern::add_pattern_screen_handlers,
    screen::{Screen, Screens},
    song::add_song_screen_handlers,
    timers::TIMERS,
    tracker::TRACKER,
    unique_usize::get_unique_usize,
    wtime::Winstant,
    INPUTS,
};

const ANIM_DURATION_MS: u64 = 300;

unsafe fn run_transition(from: Screen, to: Screen, duration: Duration) {
    let start = Winstant::now();
    let duration_ms_f32 = duration.as_millis() as f32;

    let interval_id = get_unique_usize();
    TIMERS.set_interval(
        interval_id,
        move || {
            let now = Winstant::now();
            let since_start = now.duration_since(start);
            let progress: f32 = since_start.as_millis() as f32 / duration_ms_f32;
            if progress < 1.0 {
                TRACKER.set_screens(Screens::Transition(from, to, progress));
            } else {
                TRACKER.set_screens(Screens::Single(to));
                TIMERS.cancel_interval(interval_id);
            }
        },
        1,
    );
}

pub unsafe fn go_to_pattern_screen(from: Screen) {
    TRACKER.set_screens(Screens::Single(Screen::Pattern));
    run_transition(
        from,
        Screen::Pattern,
        Duration::from_millis(ANIM_DURATION_MS),
    );
    INPUTS.unlisten();
    add_pattern_screen_handlers(&mut INPUTS);
}

pub unsafe fn go_to_instrument_screen(from: Screen) {
    TRACKER.set_screens(Screens::Transition(from, Screen::Instrument, 0.0));
    run_transition(
        from,
        Screen::Instrument,
        Duration::from_millis(ANIM_DURATION_MS),
    );
    INPUTS.unlisten();
    add_instrument_screen_handlers(&mut INPUTS);
}

pub unsafe fn go_to_song_screen(from: Screen) {
    TRACKER.set_screens(Screens::Single(Screen::Song));
    run_transition(from, Screen::Song, Duration::from_millis(ANIM_DURATION_MS));
    INPUTS.unlisten();
    add_song_screen_handlers(&mut INPUTS);
}
