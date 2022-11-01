use crate::{
    instrument::prepare_instrument_screen, prepare_pattern_screen, screen::Screen,
    tracker::TRACKER, INPUTS,
};

pub unsafe fn go_to_pattern_screen() {
    TRACKER.set_screen(Screen::Pattern);
    INPUTS.unlisten();
    prepare_pattern_screen();
}

pub unsafe fn go_to_instrument_screen() {
    TRACKER.set_screen(Screen::Instrument);
    INPUTS.unlisten();
    prepare_instrument_screen();
}
