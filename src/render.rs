use crate::{
    channel::Channel,
    instrument::{DutyCycle, InstrumentInput},
    notes::note_to_render,
    screen::{Screen, Screens},
    tracker::{Column, PlayMode, Tracker},
    wasm4::{hline, rect, text, text_bytes, vline, DRAW_COLORS, SCREEN_SIZE},
};

enum Color {
    Background,
    Light,
    Primary,
    Dark,
}

fn set_color(color: Color) {
    unsafe {
        *DRAW_COLORS = match color {
            Color::Background => 1,
            Color::Light => 2,
            Color::Primary => 3,
            Color::Dark => 4,
        }
    }
}

pub fn pattern_screen(tracker: &Tracker, origin_x: i32, origin_y: i32) {
    let relative_x = |a: i32| a + origin_x;
    let relative_y = |a: i32| a + origin_y;

    let cursor = tracker.cursor_tick();
    let selected_column = tracker.selected_column();

    let pattern = tracker.selected_pattern();
    text(
        format!("Pattern {:02X}", pattern),
        relative_x(80),
        relative_y(1),
    );

    for line in 0..16 {
        text(
            format!("{:0X}", line),
            relative_x(1),
            relative_y(line * 10 + 1),
        );

        let note = tracker.note_at(line as usize);
        let name = if let Some(note) = note {
            note_to_render(usize::from(note.note_index()))
        } else {
            "---".to_string()
        };

        if line == cursor.into() && selected_column == Column::Note {
            rect(relative_x(20), relative_y(line * 10), 8 * 3 + 1, 10);
            set_color(Color::Background);
            text(name, relative_x(21), relative_y(line * 10 + 1));
            set_color(Color::Primary);
        } else {
            text(name, relative_x(21), relative_y(line * 10 + 1));
        };

        let instrument_name = if let Some(note) = note {
            format!("{:02X}", note.instrument_index())
        } else {
            "--".to_string()
        };
        if line == cursor.into() && selected_column == Column::Instrument {
            rect(relative_x(50), relative_y(line * 10), 8 * 2 + 1, 10);
            set_color(Color::Background);
            text(instrument_name, relative_x(51), relative_y(line * 10 + 1));
            set_color(Color::Primary);
        } else {
            text(instrument_name, relative_x(51), relative_y(line * 10 + 1));
        };
    }

    set_color(Color::Light);
    let first_row_y = 88;
    text_bytes(
        b"nav:   \x84\x85\x86\x87",
        relative_x(70),
        relative_y(first_row_y + 10 * 0),
    );
    text_bytes(
        b"play:   \x81+\x87",
        relative_x(70),
        relative_y(first_row_y + 10 * 1),
    );
    text_bytes(
        b"add note: \x80",
        relative_x(70),
        relative_y(first_row_y + 10 * 2),
    );
    text_bytes(
        b"rm note: \x80\x80",
        relative_x(70),
        relative_y(first_row_y + 10 * 3),
    );
    text_bytes(
        b"edit:\x80+\x84\x85\x86\x87",
        relative_x(70),
        relative_y(first_row_y + 10 * 4),
    );
    text_bytes(
        b"screen:\x81+\x84\x85",
        relative_x(70),
        relative_y(first_row_y + 10 * 5),
    );
    text_bytes(
        b"save:   \x81+\x86",
        relative_x(70),
        relative_y(first_row_y + 10 * 6),
    );

    set_color(Color::Primary);

    let tick: i32 = tracker.tick().into();
    text(">", relative_x(11), relative_y(tick * 10 + 1));
}

fn draw_sqr_waveform(signal_active: u32, signal_width: u32, amplitude: u32, x: i32, y: i32) {
    hline(x, y + amplitude as i32, signal_active);
    vline(x + signal_active as i32, y + 1, amplitude);
    hline(
        x + signal_active as i32,
        y + 1,
        signal_width - signal_active,
    );
}

pub fn instrument_screen(tracker: &Tracker, origin_x: i32, origin_y: i32) {
    let relative_x = |a: i32| a + origin_x;
    let relative_y = |a: i32| a + origin_y;

    set_color(Color::Primary);

    let selected_instrument_index = tracker.selected_instrument_index();
    text(
        format!("Instrument {:02X}", selected_instrument_index),
        relative_x(10),
        relative_y(10),
    );

    let instrument = tracker.selected_instrument();
    let focus = tracker.instrument_focus();

    let value_column_x = 120;

    let duty_cycle_x = 10;
    let duty_cycle_y = 30;
    let text_size_x = value_column_x - 10;
    text(
        "Duty cycle",
        relative_x(duty_cycle_x),
        relative_y(duty_cycle_y),
    );
    if focus == InstrumentInput::DutyCycle {
        rect(
            relative_x(duty_cycle_x + text_size_x - 1),
            relative_y(duty_cycle_y - 1),
            18,
            10,
        );
        set_color(Color::Background);
    }
    let signal_width = 16;
    let signal_active = match instrument.duty_cycle() {
        DutyCycle::Eighth => signal_width / 8,
        DutyCycle::Fourth => signal_width / 4,
        DutyCycle::Half => signal_width / 2,
        DutyCycle::ThreeFourth => 3 * signal_width / 4,
    };

    draw_sqr_waveform(
        signal_active,
        signal_width,
        8,
        relative_x(duty_cycle_x + text_size_x),
        relative_y(duty_cycle_y - 1),
    );
    if focus == InstrumentInput::DutyCycle {
        set_color(Color::Primary);
    }

    let input = |x: i32, y: i32, label: &str, value: u8, id: InstrumentInput| {
        set_color(Color::Primary);
        text(label, relative_x(x), relative_y(y));
        let value_x: i32 = value_column_x;
        if focus == id {
            let rect_width: u32 = 8 * 2 + 1;
            rect(relative_x(value_x - 1), relative_y(y - 1), rect_width, 9);
            set_color(Color::Background);
        }
        text(format!("{:02X}", value), relative_x(value_x), relative_y(y));
        if focus == id {
            set_color(Color::Primary);
        }
    };

    input(
        10,
        40,
        "Attack",
        instrument.attack(),
        InstrumentInput::Attack,
    );

    input(10, 50, "Decay", instrument.decay(), InstrumentInput::Decay);

    input(
        10,
        60,
        "Sustain",
        instrument.sustain(),
        InstrumentInput::Sustain,
    );

    input(
        10,
        70,
        "Release",
        instrument.release(),
        InstrumentInput::Release,
    );

    input(
        10,
        80,
        "Volume",
        instrument.volume(),
        InstrumentInput::Volume,
    );

    input(10, 90, "Peak", instrument.peak(), InstrumentInput::Peak);
}

pub fn song_screen(tracker: &Tracker, origin_x: i32, origin_y: i32) {
    let relative_x = |a: i32| a + origin_x;
    let relative_y = |a: i32| a + origin_y;

    set_color(Color::Primary);

    impl Channel {
        fn to_x(&self) -> i32 {
            let x0 = 30;
            let d = 30;
            match self {
                Channel::Pulse1 => x0,
                Channel::Pulse2 => x0 + d,
                Channel::Triangle => x0 + d * 2,
                Channel::Noise => x0 + d * 3,
            }
        }
    }

    text("P1", relative_x(Channel::Pulse1.to_x()), relative_y(10));
    text("P2", relative_x(Channel::Pulse2.to_x()), relative_y(10));
    text("TR", relative_x(Channel::Triangle.to_x()), relative_y(10));
    text("NS", relative_x(Channel::Noise.to_x()), relative_y(10));

    let selected_channel = tracker.selected_channel();
    let row = tracker.song_cursor_row();
    let song = tracker.song();
    for channel in Channel::iterator() {
        let x = channel.to_x();
        for line in 0..song.len() {
            let y: i32 = 30 + line as i32 * 10;

            let val = match song[line].channel(&channel) {
                Some(index) => format!("{:02X}", index),
                None => "--".to_string(),
            };
            if *selected_channel == channel && line == row {
                set_color(Color::Primary);
                rect(relative_x(x - 1), relative_y(y - 1), 18, 9);
                set_color(Color::Background);
                text(val, relative_x(x), relative_y(y));
                set_color(Color::Primary);
            } else {
                text(val, relative_x(x), relative_y(y));
            }

            if let PlayMode::Song = tracker.play_mode() {
                if tracker.song_tick() == line {
                    text(">", relative_x(x - 10), relative_y(y));
                }
            }
        }
    }
}

pub fn not_implemented_screen() {
    set_color(Color::Primary);
    text("Screen is not\nimplemented", 10, 10);
}

fn render_screen(screen: &Screen, tracker: &Tracker, x: i32, y: i32) {
    match screen {
        Screen::Pattern => pattern_screen(tracker, x, y),
        Screen::Instrument => instrument_screen(tracker, x, y),
        Screen::Song => song_screen(tracker, x, y),
        _ => not_implemented_screen(),
    }
}

enum TransitionDirection {
    Left,
    Right,
}

fn direction(from: Screen, to: Screen) -> TransitionDirection {
    match (from, to) {
        (Screen::Song, Screen::Pattern) => TransitionDirection::Right,
        (Screen::Pattern, Screen::Song) => TransitionDirection::Left,
        (Screen::Pattern, Screen::Instrument) => TransitionDirection::Right,
        (Screen::Instrument, Screen::Pattern) => TransitionDirection::Left,
        _ => TransitionDirection::Right,
    }
}

const SCREEN_SIZE_F32: f32 = SCREEN_SIZE as f32;

fn ease_out_quart(progress: f32) -> f32 {
    1.0 - (1.0 - progress).powi(4)
}

pub fn render_screens(screens: &Screens, tracker: &Tracker) {
    match screens {
        Screens::Single(screen) => render_screen(screen, tracker, 0, 0),
        Screens::Transition(from, to, progress) => {
            let corrected_progress = ease_out_quart(*progress);
            let delta = (corrected_progress * SCREEN_SIZE_F32) as i32;
            let dir_coeff = match direction(*from, *to) {
                TransitionDirection::Left => 1,
                TransitionDirection::Right => -1,
            };
            let x_from = delta * dir_coeff;
            let x_to = (delta - SCREEN_SIZE as i32) * dir_coeff;

            render_screen(from, tracker, x_from, 0);
            render_screen(to, tracker, x_to, 0);
        }
    }
}
