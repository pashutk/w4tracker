use std::time::Duration;

use crate::{
    inputs::{InputEvent, Inputs},
    navigation::go_to_pattern_screen,
    timers::{ActionId, TIMERS},
    tracker::{PlayMode, TRACKER},
};

#[derive(Clone, Copy, PartialEq)]
pub enum InstrumentInput {
    DutyCycle,
    Attack,
    Decay,
    Sustain,
    Release,
    Volume,
    Peak,
}

fn on_button_down_press(inputs: &Inputs) {
    unsafe {
        if inputs.is_button2_pressed() {
            TIMERS.run_action_debounced(ActionId::Play, Duration::from_millis(200), || {
                TRACKER.toggle_play(PlayMode::Pattern)
            })
        } else if inputs.is_button1_pressed() {
            TIMERS.run_action_debounced(
                ActionId::InstrumentValueDownActionId,
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
                        InstrumentInput::Volume => {
                            selected_instrument.update_volume(|a| a.saturating_sub(0x10))
                        }
                        InstrumentInput::Peak => {
                            selected_instrument.update_peak(|a| a.saturating_sub(0x10))
                        }
                        InstrumentInput::DutyCycle => {}
                    }
                },
            )
        } else {
            TIMERS.run_action_debounced(
                ActionId::InstrumentNextInputActionId,
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
                ActionId::InstrumentValueUpActionId,
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
                        InstrumentInput::Volume => {
                            selected_instrument.update_volume(|a| a.saturating_add(0x10))
                        }
                        InstrumentInput::Peak => {
                            selected_instrument.update_peak(|a| a.saturating_add(0x10))
                        }
                        InstrumentInput::DutyCycle => {}
                    }
                },
            )
        } else if inputs.is_button2_pressed() {
            TIMERS.run_action_debounced(ActionId::Persist, Duration::from_millis(1000), || {
                TRACKER.persist();
            })
        } else {
            TIMERS.run_action_debounced(
                ActionId::InstrumentPrevInputActionId,
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
                ActionId::NavPrevScreen,
                Duration::from_millis(200),
                || go_to_pattern_screen(),
            );
        } else if inputs.is_button1_pressed() {
            TIMERS.run_action_debounced(
                ActionId::InstrumentValuePrevActionId,
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
                        InstrumentInput::Volume => {
                            selected_instrument.update_volume(|a| a.saturating_sub(1))
                        }
                        InstrumentInput::Peak => {
                            selected_instrument.update_peak(|a| a.saturating_sub(1))
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
                ActionId::InstrumentValueNextActionId,
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
                        InstrumentInput::Volume => {
                            selected_instrument.update_volume(|a| a.saturating_add(1))
                        }
                        InstrumentInput::Peak => {
                            selected_instrument.update_peak(|a| a.saturating_add(1))
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
