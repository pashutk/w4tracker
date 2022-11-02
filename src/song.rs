use std::time::Duration;

use crate::{
    inputs::{InputEvent, Inputs},
    navigation::go_to_pattern_screen,
    timers::TIMERS,
    tracker::TRACKER,
    INPUTS,
};

fn on_button_down_press(inputs: &Inputs) {
    unsafe {
        if inputs.is_button2_pressed() {
            TIMERS.run_action_debounced("play".to_string(), Duration::from_millis(200), || {
                TRACKER.toggle_play()
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
                || go_to_pattern_screen(),
            );
        } else if inputs.is_button1_pressed() {
        } else {
            TIMERS.run_action_debounced(
                "next_channel".to_string(),
                Duration::from_millis(200),
                || TRACKER.next_channel(),
            )
        }
    }
}

pub fn add_song_screen_handlers(inputs: &mut Inputs) {
    inputs
        .listen(InputEvent::ButtonDownPress, on_button_down_press)
        .listen(InputEvent::ButtonUpPress, on_button_up_press)
        .listen(InputEvent::ButtonLeftPress, on_button_left_press)
        .listen(InputEvent::ButtonRightPress, on_button_right_press);
}
