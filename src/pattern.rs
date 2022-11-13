use std::time::Duration;

use crate::{
    inputs::{InputEvent, Inputs},
    navigation::{go_to_instrument_screen, go_to_song_screen},
    timers::{ActionId, TIMERS},
    tracker::{Column, Note, PlayMode, TRACKER},
};

fn on_button_down_press(inputs: &Inputs) {
    unsafe {
        if inputs.is_button1_pressed() && TRACKER.selected_column() == Column::Note {
            TIMERS.run_action_debounced(
                ActionId::PatternPitchOctaveDown,
                Duration::from_millis(100),
                || {
                    if let Some(note) = TRACKER.current_note_mut() {
                        note.decrease_octave();
                    }
                },
            )
        } else if inputs.is_button2_pressed() {
            TIMERS.run_action_debounced(ActionId::Play, Duration::from_millis(200), || {
                TRACKER.toggle_play(PlayMode::Pattern)
            })
        } else if inputs.is_button1_pressed() && TRACKER.selected_column() == Column::Instrument {
        } else {
            TIMERS.run_action_debounced(
                ActionId::PatternNavDown,
                Duration::from_millis(100),
                || {
                    TRACKER.saturating_increase_cursor_tick();
                    if let Some(note) = TRACKER.current_note() {
                        TRACKER.set_selected_instrument_index(note.instrument_index());
                    }
                },
            )
        }
    }
}

fn on_button_up_press(inputs: &Inputs) {
    unsafe {
        if inputs.is_button1_pressed() && TRACKER.selected_column() == Column::Note {
            TIMERS.run_action_debounced(
                ActionId::PatternPitchOctaveUp,
                Duration::from_millis(100),
                || {
                    if let Some(note) = TRACKER.current_note_mut() {
                        note.increase_octave();
                    }
                },
            )
        } else if inputs.is_button1_pressed() && TRACKER.selected_column() == Column::Instrument {
        } else if inputs.is_button2_pressed() {
            TIMERS.run_action_debounced(ActionId::Persist, Duration::from_millis(1000), || {
                TRACKER.persist();
            })
        } else {
            TIMERS.run_action_debounced(ActionId::PatternNavUp, Duration::from_millis(100), || {
                TRACKER.saturating_decrease_cursor_tick();
                if let Some(note) = TRACKER.current_note() {
                    TRACKER.set_selected_instrument_index(note.instrument_index());
                }
            })
        }
    }
}

fn on_button_1_press(inputs: &Inputs) {
    unsafe {
        if let None = TRACKER.current_note() {
            let new_note = Note::new();
            TRACKER.set_current_note(&Some(new_note));
        }
    }
}

fn on_button_1_double_press(inputs: &Inputs) {
    unsafe {
        if let Some(_) = TRACKER.current_note() {
            TRACKER.set_current_note(&None);
        }
    }
}

fn on_button_right_press(inputs: &Inputs) {
    unsafe {
        if inputs.is_button1_pressed() {
            match TRACKER.selected_column() {
                Column::Note => TIMERS.run_action_debounced(
                    ActionId::PatternPitchUp,
                    Duration::from_millis(100),
                    || {
                        if let Some(note) = TRACKER.current_note_mut() {
                            note.increase_pitch();
                        }
                    },
                ),
                Column::Instrument => TIMERS.run_action_debounced(
                    ActionId::PatternInstrumentNext,
                    Duration::from_millis(200),
                    || {
                        if let Some(note) = TRACKER.current_note_mut() {
                            note.next_instrument();
                        }
                    },
                ),
            };
        } else if inputs.is_button2_pressed() {
            TIMERS.run_action_debounced(
                ActionId::NavNextScreen,
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

fn on_button_left_press(inputs: &Inputs) {
    unsafe {
        if inputs.is_button1_pressed() {
            match TRACKER.selected_column() {
                Column::Note => TIMERS.run_action_debounced(
                    ActionId::PatternPitchDown,
                    Duration::from_millis(100),
                    || {
                        if let Some(note) = TRACKER.current_note_mut() {
                            note.decrease_pitch()
                        }
                    },
                ),
                Column::Instrument => TIMERS.run_action_debounced(
                    ActionId::PatternInstrumentPrev,
                    Duration::from_millis(200),
                    || {
                        if let Some(note) = TRACKER.current_note_mut() {
                            note.prev_instrument()
                        }
                    },
                ),
            }
        } else if inputs.is_button2_pressed() {
            TIMERS.run_action_debounced(
                ActionId::NavPrevScreen,
                Duration::from_millis(200),
                || {
                    go_to_song_screen();
                },
            );
        } else if TRACKER.selected_column() == Column::Instrument {
            TRACKER.set_selected_column(Column::Note);
        }
    }
}

pub fn add_pattern_screen_handlers(inputs: &mut Inputs) {
    inputs
        .listen(InputEvent::ButtonDownPress, on_button_down_press)
        .listen(InputEvent::ButtonUpPress, on_button_up_press)
        .listen(InputEvent::Button1Press, on_button_1_press)
        .listen(InputEvent::Button1DoublePress, on_button_1_double_press)
        .listen(InputEvent::ButtonRightPress, on_button_right_press)
        .listen(InputEvent::ButtonLeftPress, on_button_left_press);
}
