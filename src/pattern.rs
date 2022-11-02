use std::time::Duration;

use crate::{
    inputs::InputEvent,
    navigation::go_to_instrument_screen,
    timers::TIMERS,
    tracker::{Column, Note, TRACKER},
    wasm4::trace,
    INPUTS,
};

fn on_button_down_press() {
    unsafe {
        if INPUTS.is_button1_pressed() && TRACKER.selected_column() == Column::Note {
            TIMERS.run_action_debounced(
                "pitch_octave_down".to_string(),
                Duration::from_millis(100),
                || {
                    if let Some(note) = TRACKER.current_note_mut() {
                        note.decrease_octave();
                    }
                },
            )
        } else if INPUTS.is_button2_pressed() {
            TIMERS.run_action_debounced("play".to_string(), Duration::from_millis(200), || {
                TRACKER.toggle_play()
            })
        } else if INPUTS.is_button1_pressed() && TRACKER.selected_column() == Column::Instrument {
        } else {
            TIMERS.run_action_debounced("nav_down".to_string(), Duration::from_millis(100), || {
                TRACKER.saturating_increase_cursor_tick();
                if let Some(note) = TRACKER.current_note() {
                    TRACKER.set_selected_instrument_index(note.instrument_index());
                }
            })
        }
    }
}

fn on_button_up_press() {
    unsafe {
        if INPUTS.is_button1_pressed() && TRACKER.selected_column() == Column::Note {
            TIMERS.run_action_debounced(
                "pitch_octave_up".to_string(),
                Duration::from_millis(100),
                || {
                    if let Some(note) = TRACKER.current_note_mut() {
                        note.increase_octave();
                    }
                },
            )
        } else if INPUTS.is_button1_pressed() && TRACKER.selected_column() == Column::Instrument {
        } else {
            TIMERS.run_action_debounced("nav_up".to_string(), Duration::from_millis(100), || {
                TRACKER.saturating_decrease_cursor_tick();
                if let Some(note) = TRACKER.current_note() {
                    TRACKER.set_selected_instrument_index(note.instrument_index());
                }
            })
        }
    }
}

fn on_button_1_press() {
    unsafe {
        if let None = TRACKER.current_note() {
            let new_note = Note::new();
            TRACKER.set_current_note(&Some(new_note));
        }
    }
}

fn on_button_1_double_press() {
    unsafe {
        if let Some(_) = TRACKER.current_note() {
            TRACKER.set_current_note(&None);
        }
    }
}

fn on_button_right_press() {
    unsafe {
        if INPUTS.is_button1_pressed() {
            match TRACKER.selected_column() {
                Column::Note => TIMERS.run_action_debounced(
                    "pitch_up".to_string(),
                    Duration::from_millis(100),
                    || {
                        if let Some(note) = TRACKER.current_note_mut() {
                            note.increase_pitch();
                        }
                    },
                ),
                Column::Instrument => TIMERS.run_action_debounced(
                    "instrument_next".to_string(),
                    Duration::from_millis(200),
                    || {
                        if let Some(note) = TRACKER.current_note_mut() {
                            note.next_instrument();
                        }
                    },
                ),
            };
        } else if INPUTS.is_button2_pressed() {
            TIMERS.run_action_debounced(
                "nav_to_instrument".to_string(),
                Duration::from_millis(200),
                || {
                    if let Some(note) = TRACKER.current_note() {
                        TRACKER.set_selected_instrument_index(note.instrument_index());
                    }
                    go_to_instrument_screen();
                },
            );
        } else if TRACKER.selected_column() == Column::Note {
            TRACKER.set_selected_column(Column::Instrument);
        }
    }
}

fn on_button_left_press() {
    unsafe {
        if INPUTS.is_button1_pressed() {
            match TRACKER.selected_column() {
                Column::Note => TIMERS.run_action_debounced(
                    "pitch_down".to_string(),
                    Duration::from_millis(100),
                    || {
                        if let Some(note) = TRACKER.current_note_mut() {
                            note.decrease_pitch()
                        }
                    },
                ),
                Column::Instrument => TIMERS.run_action_debounced(
                    "instrument_prev".to_string(),
                    Duration::from_millis(200),
                    || {
                        if let Some(note) = TRACKER.current_note_mut() {
                            note.prev_instrument()
                        }
                    },
                ),
            }
        } else if INPUTS.is_button2_pressed() {
        } else if TRACKER.selected_column() == Column::Instrument {
            TRACKER.set_selected_column(Column::Note);
        }
    }
}

pub unsafe fn prepare_pattern_screen() {
    INPUTS
        .listen(InputEvent::ButtonDownPress, on_button_down_press)
        .listen(InputEvent::ButtonUpPress, on_button_up_press)
        .listen(InputEvent::Button1Press, on_button_1_press)
        .listen(InputEvent::Button1DoublePress, on_button_1_double_press)
        .listen(InputEvent::ButtonRightPress, on_button_right_press)
        .listen(InputEvent::ButtonLeftPress, on_button_left_press);
}
