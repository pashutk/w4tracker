use crate::{
    instrument::InstrumentInput,
    notes::note_to_render,
    tracker::{Column, DutyCycle, Tracker},
    wasm4::{hline, rect, text, text_bytes, vline, DRAW_COLORS},
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

pub fn pattern_screen(tracker: &Tracker) {
    let cursor = tracker.cursor_tick();
    let selected_column = tracker.selected_column();

    for line in 0..16 {
        text(format!("{:0X}", line), 1, line * 10 + 1);

        let note = tracker.note_at(line as usize);
        let name = if let Some(note) = note {
            note_to_render(usize::from(note.note_index()))
        } else {
            "---".to_string()
        };

        if line == cursor.into() && selected_column == Column::Note {
            rect(20, line * 10, 8 * 3 + 1, 10);
            set_color(Color::Background);
            text(name, 21, line * 10 + 1);
            set_color(Color::Primary);
        } else {
            text(name, 21, line * 10 + 1);
        };

        let instrument_name = if let Some(note) = note {
            format!("{:02X}", note.instrument_index())
        } else {
            "--".to_string()
        };
        if line == cursor.into() && selected_column == Column::Instrument {
            rect(50, line * 10, 8 * 2 + 1, 10);
            set_color(Color::Background);
            text(instrument_name, 51, line * 10 + 1);
            set_color(Color::Primary);
        } else {
            text(instrument_name, 51, line * 10 + 1);
        };
    }

    set_color(Color::Light);
    let first_row_y = 98;
    text_bytes(b"nav:   \x84\x85\x86\x87", 70, first_row_y + 10 * 0);
    text_bytes(b"play:   \x81+\x87", 70, first_row_y + 10 * 1);
    text_bytes(b"add note: \x80", 70, first_row_y + 10 * 2);
    text_bytes(b"rm note: \x80\x80", 70, first_row_y + 10 * 3);
    text_bytes(b"edit:\x80+\x84\x85\x86\x87", 70, first_row_y + 10 * 4);
    text_bytes(b"screen:\x81+\x84\x85", 70, first_row_y + 10 * 5);

    set_color(Color::Primary);

    let tick: i32 = tracker.tick().into();
    text(">", 11, tick * 10 + 1);
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

pub fn instrument_screen(tracker: &Tracker) {
    set_color(Color::Primary);

    let selected_instrument_index = tracker.selected_instrument_index();
    text(
        format!("Instrument {:02X}", selected_instrument_index),
        10,
        10,
    );

    let instrument = tracker.selected_instrument();
    let focus = tracker.instrument_focus();

    let value_column_x = 120;

    let duty_cycle_x = 10;
    let duty_cycle_y = 30;
    let text_size_x = value_column_x - 10;
    text("Duty cycle", duty_cycle_x, duty_cycle_y);
    if focus == InstrumentInput::DutyCycle {
        rect(duty_cycle_x + text_size_x - 1, duty_cycle_y - 1, 18, 10);
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
        duty_cycle_x + text_size_x,
        duty_cycle_y - 1,
    );
    if focus == InstrumentInput::DutyCycle {
        set_color(Color::Primary);
    }

    let input = |x: i32, y: i32, label: &str, value: u8, id: InstrumentInput| {
        set_color(Color::Primary);
        text(label, x, y);
        let value_x: i32 = value_column_x;
        if focus == id {
            let rect_width: u32 = 8 * 2 + 1;
            rect(value_x - 1, y - 1, rect_width, 9);
            set_color(Color::Background);
        }
        text(format!("{:02X}", value), value_x, y);
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
}

pub fn not_implemented_screen() {
    set_color(Color::Primary);
    text("Screen is not\nimplemented", 10, 10);
}