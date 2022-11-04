use std::time::Duration;

use crate::{
    inputs::{InputEvent, Inputs},
    navigation::go_to_pattern_screen,
    timers::TIMERS,
    tracker::{PlayMode, TRACKER},
    wasm4::trace,
};

fn on_button_down_press(inputs: &Inputs) {
    unsafe {
        if inputs.is_button2_pressed() {
            TIMERS.run_action_debounced("play".to_string(), Duration::from_millis(200), || {
                TRACKER.toggle_play(PlayMode::Song)
            })
        } else if inputs.is_button1_pressed() {
        } else {
            TIMERS.run_action_debounced(
                "next_row_cursor".to_string(),
                Duration::from_millis(200),
                || TRACKER.next_row_song_cursor(),
            )
        }
    }
}

fn on_button_up_press(inputs: &Inputs) {
    unsafe {
        if inputs.is_button1_pressed() {
        } else if inputs.is_button2_pressed() {
            TIMERS.run_action_debounced("persist".to_string(), Duration::from_millis(1000), || {
                TRACKER.persist();
                trace("PERSISTED");
            })
        } else {
            TIMERS.run_action_debounced(
                "prev_row_cursor".to_string(),
                Duration::from_millis(200),
                || TRACKER.prev_row_song_cursor(),
            )
        }
    }
}

fn on_button_left_press(inputs: &Inputs) {
    unsafe {
        if inputs.is_button2_pressed() {
        } else if inputs.is_button1_pressed() {
            TIMERS.run_action_debounced(
                "decrement_pattern".to_string(),
                Duration::from_millis(200),
                || {
                    let selected_row = TRACKER.song_cursor_row();
                    let song = TRACKER.song_mut();
                    let row = song.get_mut(selected_row);
                    if let Some(row) = row {
                        let selected_channel = TRACKER.selected_channel();
                        row.decrement_channel_value(selected_channel)
                    }
                },
            )
        } else {
            TIMERS.run_action_debounced(
                "prev_channel".to_string(),
                Duration::from_millis(200),
                || TRACKER.prev_channel(),
            )
        }
    }
}

fn on_button_right_press(inputs: &Inputs) {
    unsafe {
        if inputs.is_button2_pressed() {
            TIMERS.run_action_debounced(
                "nav_next_screen".to_string(),
                Duration::from_millis(200),
                || {
                    let selected_row = TRACKER.song_cursor_row();
                    let song = TRACKER.song();
                    let row = song.get(selected_row);
                    let selected_channel = TRACKER.selected_channel();
                    let selected_pattern = match row {
                        Some(r) => r.channel(selected_channel).unwrap_or(0),
                        None => 0,
                    };
                    TRACKER.set_selected_pattern(selected_pattern);
                    go_to_pattern_screen();
                },
            );
        } else if inputs.is_button1_pressed() {
            TIMERS.run_action_debounced(
                "increment_pattern".to_string(),
                Duration::from_millis(200),
                || {
                    let selected_row = TRACKER.song_cursor_row();
                    let song = TRACKER.song_mut();
                    let row = song.get_mut(selected_row);
                    if let Some(row) = row {
                        let selected_channel = TRACKER.selected_channel();
                        row.increment_channel_value(selected_channel)
                    }
                },
            )
        } else {
            TIMERS.run_action_debounced(
                "next_channel".to_string(),
                Duration::from_millis(200),
                || TRACKER.next_channel(),
            )
        }
    }
}

fn on_button_1_press(inputs: &Inputs) {
    unsafe {
        TIMERS.run_action_debounced(
            "add_pattern".to_string(),
            Duration::from_millis(200),
            || {
                let selected_channel = TRACKER.selected_channel();
                let selected_row = TRACKER.song_cursor_row();
                let song = TRACKER.song_mut();
                let row = song.get_mut(selected_row);
                if let Some(row) = row {
                    if let None = row.channel(selected_channel) {
                        row.set_channel_value(selected_channel, Some(0));
                    }
                }
            },
        );
    }
}

pub fn add_song_screen_handlers(inputs: &mut Inputs) {
    inputs
        .listen(InputEvent::ButtonDownPress, on_button_down_press)
        .listen(InputEvent::ButtonUpPress, on_button_up_press)
        .listen(InputEvent::ButtonLeftPress, on_button_left_press)
        .listen(InputEvent::ButtonRightPress, on_button_right_press)
        .listen(InputEvent::Button1Press, on_button_1_press);
}
