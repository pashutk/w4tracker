use std::{borrow::BorrowMut, time::Duration};

use crate::{
    inputs::InputEvent, navigation::go_to_pattern_screen, screen::Screen, timers::TIMERS,
    tracker::TRACKER, INPUTS,
};

#[derive(Clone, Copy, PartialEq)]
pub enum InstrumentInput {
    DutyCycle,
    Attack,
    Decay,
    Sustain,
    Release,
}

unsafe fn on_button_down_press() {
    if INPUTS.is_button2_pressed() {
        TIMERS.run_action_debounced("play".to_string(), Duration::from_millis(200), || {
            TRACKER.toggle_play()
        })
    } else if INPUTS.is_button1_pressed() {
        TIMERS.run_action_debounced(
            "instrument_value_down".to_string(),
            Duration::from_millis(200),
            || {
                let mut selected_instrument = *TRACKER.selected_instrument();

                match TRACKER.instrument_focus() {
                    InstrumentInput::Attack => {
                        selected_instrument.update_attack(|a| a.saturating_add(0x10))
                    }
                    InstrumentInput::Decay => {
                        selected_instrument.update_decay(|a| a.saturating_add(0x10))
                    }
                    InstrumentInput::Sustain => {
                        selected_instrument.update_sustain(|a| a.saturating_add(0x10))
                    }
                    InstrumentInput::Release => {
                        selected_instrument.update_release(|a| a.saturating_add(0x10))
                    }
                    _ => {}
                }
            },
        )
    } else {
        TIMERS.run_action_debounced(
            "instrument_input_next".to_string(),
            Duration::from_millis(200),
            || TRACKER.instrument_focus_next(),
        )
    }
}

pub unsafe fn prepare_instrument_screen() {
    INPUTS
        .listen(InputEvent::ButtonDownPress, || on_button_down_press())
        .listen(InputEvent::ButtonUpPress, || {
            if INPUTS.is_button1_pressed() {
                TIMERS.run_action_debounced(
                    "instrument_value_up".to_string(),
                    Duration::from_millis(200),
                    || {
                        let mut selected_instrument = *TRACKER.selected_instrument();

                        match TRACKER.instrument_focus() {
                            InstrumentInput::Attack => {
                                selected_instrument.update_attack(|a| a.saturating_add(0x10))
                            }
                            InstrumentInput::Decay => {
                                selected_instrument.update_decay(|a| a.saturating_add(0x10))
                            }
                            InstrumentInput::Sustain => {
                                selected_instrument.update_sustain(|a| a.saturating_add(0x10))
                            }
                            InstrumentInput::Release => {
                                selected_instrument.update_release(|a| a.saturating_add(0x10))
                            }
                            _ => {}
                        }
                    },
                )
            } else {
                TIMERS.run_action_debounced(
                    "instrument_input_prev".to_string(),
                    Duration::from_millis(200),
                    || TRACKER.instrument_focus_prev(),
                )
            }
        })
        .listen(InputEvent::ButtonLeftPress, || {
            if INPUTS.is_button2_pressed() {
                TIMERS.run_action_debounced(
                    "nav_to_pattern".to_string(),
                    Duration::from_millis(200),
                    || go_to_pattern_screen(),
                );
            } else if INPUTS.is_button1_pressed() {
                TIMERS.run_action_debounced(
                    "instrument_value_prev".to_string(),
                    Duration::from_millis(200),
                    || {
                        let mut selected_instrument = *TRACKER.selected_instrument();

                        match TRACKER.instrument_focus() {
                            InstrumentInput::DutyCycle => {
                                selected_instrument.update_duty_cycle(|a| a.prev())
                            }
                            InstrumentInput::Attack => {
                                selected_instrument.update_attack(|a| a.saturating_sub(1))
                            }
                            InstrumentInput::Decay => {
                                selected_instrument.update_decay(|a| a.saturating_sub(1))
                            }
                            InstrumentInput::Sustain => {
                                selected_instrument.update_sustain(|a| a.saturating_sub(1))
                            }
                            InstrumentInput::Release => {
                                selected_instrument.update_release(|a| a.saturating_sub(1))
                            }
                        }
                    },
                )
            }
        })
        .listen(InputEvent::ButtonRightPress, || {
            if INPUTS.is_button1_pressed() {
                TIMERS.run_action_debounced(
                    "instrument_value_next".to_string(),
                    Duration::from_millis(200),
                    || {
                        let mut selected_instrument = *TRACKER.selected_instrument();

                        match TRACKER.instrument_focus() {
                            InstrumentInput::DutyCycle => {
                                selected_instrument.update_duty_cycle(|a| a.next())
                            }
                            InstrumentInput::Attack => {
                                selected_instrument.update_attack(|a| a.saturating_add(1))
                            }
                            InstrumentInput::Decay => {
                                selected_instrument.update_decay(|a| a.saturating_add(1))
                            }
                            InstrumentInput::Sustain => {
                                selected_instrument.update_sustain(|a| a.saturating_add(1))
                            }
                            InstrumentInput::Release => {
                                selected_instrument.update_release(|a| a.saturating_add(1))
                            }
                        }
                    },
                )
            }
        });
}
