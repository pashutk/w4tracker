use crate::{
    add_pattern_screen_handlers, instrument::add_instrument_screen_handlers, screen::Screen,
    tracker::TRACKER, INPUTS,
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
