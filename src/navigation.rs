use crate::{
    instrument::add_instrument_screen_handlers, pattern::add_pattern_screen_handlers,
    screen::Screen, song::add_song_screen_handlers, tracker::TRACKER, INPUTS,
};

pub unsafe fn go_to_pattern_screen() {
    TRACKER.set_screen(Screen::Pattern);
    INPUTS.unlisten();
    add_pattern_screen_handlers(&mut INPUTS);
}

pub unsafe fn go_to_instrument_screen() {
    TRACKER.set_screen(Screen::Instrument);
    INPUTS.unlisten();
    add_instrument_screen_handlers(&mut INPUTS);
}

pub unsafe fn go_to_song_screen() {
    TRACKER.set_screen(Screen::Song);
    INPUTS.unlisten();
    add_song_screen_handlers(&mut INPUTS);
}
