use std::time::Duration;

use crate::{
    inputs::{InputEvent, Inputs},
    navigation::go_to_pattern_screen,
    timers::TIMERS,
    tracker::{PlayMode, TRACKER},
    INPUTS,
};

#[derive(Clone, Copy, PartialEq)]
pub enum InstrumentInput {
    DutyCycle,
    Attack,
    Decay,
    Sustain,
    Release,
}

fn on_button_down_press(inputs: &Inputs) {
    unsafe {
        if inputs.is_button2_pressed() {
            TIMERS.run_action_debounced("play".to_string(), Duration::from_millis(200), || {
                TRACKER.toggle_play(PlayMode::Pattern)
            })
        } else if inputs.is_button1_pressed() {
            TIMERS.run_action_debounced(
                "instrument_value_down".to_string(),
                Duration::from_millis(200),
                || {
                    let selected_instrument = TRACKER.selected_instrument_mut();

                    match TRACKER.instrument_focus() {
                        InstrumentInput::Attack => {
                            selected_instrument.update_attack(|a| a.saturating_sub(0x10))
                        }
                        InstrumentInput::Decay => {
                            selected_instrument.update_decay(|a| a.saturating_sub(0x10))
                        }
                        InstrumentInput::Sustain => {
                            selected_instrument.update_sustain(|a| a.saturating_sub(0x10))
                        }
                        InstrumentInput::Release => {
                            selected_instrument.update_release(|a| a.saturating_sub(0x10))
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
}

fn on_button_up_press(inputs: &Inputs) {
    unsafe {
        if inputs.is_button1_pressed() {
            TIMERS.run_action_debounced(
                "instrument_value_up".to_string(),
                Duration::from_millis(200),
                || {
                    let selected_instrument = TRACKER.selected_instrument_mut();

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
    }
}

fn on_button_left_press(inputs: &Inputs) {
    unsafe {
        if inputs.is_button2_pressed() {
            TIMERS.run_action_debounced(
                "nav_prev_screen".to_string(),
                Duration::from_millis(200),
                || go_to_pattern_screen(),
            );
        } else if inputs.is_button1_pressed() {
            TIMERS.run_action_debounced(
                "instrument_value_prev".to_string(),
                Duration::from_millis(200),
                || {
                    let selected_instrument = TRACKER.selected_instrument_mut();

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
    }
}

fn on_button_right_press(inputs: &Inputs) {
    unsafe {
        if inputs.is_button1_pressed() {
            TIMERS.run_action_debounced(
                "instrument_value_next".to_string(),
                Duration::from_millis(200),
                || {
                    let selected_instrument = TRACKER.selected_instrument_mut();

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
    }
}

pub fn add_instrument_screen_handlers(inputs: &mut Inputs) {
    inputs
        .listen(InputEvent::ButtonDownPress, on_button_down_press)
        .listen(InputEvent::ButtonUpPress, on_button_up_press)
        .listen(InputEvent::ButtonLeftPress, on_button_left_press)
        .listen(InputEvent::ButtonRightPress, on_button_right_press);
}
