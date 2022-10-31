#[cfg(feature = "buddy-alloc")]
mod alloc;
mod inputs;
mod notes;
mod wasm4;
mod wtime;

use std::{collections::HashMap, default, time::Duration};

use inputs::{InputEvent, Inputs};
use notes::{note_c3_index, note_freq, note_from_string, note_to_render, NOTES_PER_OCTAVE};
use wasm4::*;
use wtime::Winstant;

#[derive(Clone, Copy)]
struct Note {
    index: usize,
    instrument: usize,
}

#[derive(Clone, Copy, Default)]
enum DutyCycle {
    #[default]
    Eighth,
    Fourth,
    Half,
    ThreeFourth,
}

impl DutyCycle {
    fn to_flag(&self) -> u32 {
        match self {
            Self::Eighth => TONE_MODE1,
            Self::Fourth => TONE_MODE2,
            Self::Half => TONE_MODE3,
            Self::ThreeFourth => TONE_MODE4,
        }
    }
}

#[derive(Clone, Copy, Default)]
struct Instrument {
    duty_cycle: DutyCycle,
    attack: u8,
    decay: u8,
    sustain: u8,
    release: u8,
}

#[derive(PartialEq, Clone, Copy, Default)]
enum Column {
    #[default]
    Note,
    Instrument,
}

#[derive(PartialEq, Clone, Copy)]
enum Screen {
    Song,
    Chain,
    Pattern,
    Instrument,
}

#[derive(Clone, Copy, PartialEq)]
enum InstrumentScreenInput {
    DutyCycle,
    Attack,
    Decay,
    Sustain,
    Release,
}

struct Tracker {
    frame: u32,
    tick: u8,
    pattern: [Option<Note>; 16],
    cursor_tick: u8,
    playing: bool,
    selected_column: Column,
    instruments: [Instrument; 0x1F],
    screen: Screen,
    selected_instrument: usize,
    instrument_focus: InstrumentScreenInput,
    // also bpm
}

impl Tracker {
    const fn empty() -> Self {
        Tracker {
            frame: 0,
            tick: 0,
            pattern: [None; 16],
            cursor_tick: 0,
            playing: false,
            selected_column: Column::Note,
            instruments: [Instrument {
                duty_cycle: DutyCycle::Eighth,
                attack: 0,
                decay: 0,
                sustain: 0x0f,
                release: 0x0f,
            }; 0x1f],
            screen: Screen::Pattern,
            selected_instrument: 0,
            instrument_focus: InstrumentScreenInput::DutyCycle,
        }
    }

    fn new() -> Self {
        Tracker {
            pattern: [
                Some(Note {
                    index: note_from_string("C3").unwrap(),
                    instrument: 0,
                }),
                Some(Note {
                    index: note_from_string("C3").unwrap(),
                    instrument: 1,
                }),
                Some(Note {
                    index: note_from_string("C4").unwrap(),
                    instrument: 11,
                }),
                None,
                Some(Note {
                    index: note_from_string("G3").unwrap(),
                    instrument: 0,
                }),
                None,
                None,
                Some(Note {
                    index: note_from_string("F#3").unwrap(),
                    instrument: 0,
                }),
                None,
                None,
                Some(Note {
                    index: note_from_string("F3").unwrap(),
                    instrument: 0,
                }),
                None,
                None,
                None,
                Some(Note {
                    index: note_from_string("D#3").unwrap(),
                    instrument: 0,
                }),
                None,
            ],
            ..Tracker::empty()
        }
    }

    fn play_tick(&self) {
        let pattern_index: usize = self.tick.into();
        if let Some(note) = self.pattern[pattern_index] {
            let instrument = self.instruments[note.instrument];
            let duty_cycle = instrument.duty_cycle.to_flag();
            let attack: u32 = instrument.attack.into();
            let decay: u32 = instrument.decay.into();
            let sustain: u32 = instrument.sustain.into();
            let release: u32 = instrument.release.into();
            tone(
                note_freq[note.index].into(),
                attack << 24 | decay << 16 | sustain | release << 8,
                100,
                TONE_PULSE1 | duty_cycle,
            )
        }
    }

    fn toggle_play(&mut self) {
        if !self.playing {
            self.tick = 0;
            self.frame = 0;
        }
        self.playing = !self.playing;
    }

    fn update(&mut self) {
        if !self.playing {
            return;
        }

        if self.frame == 0 {
            self.play_tick();
        }
        self.frame = if self.frame == 7 {
            self.tick = if self.tick == 15 { 0 } else { self.tick + 1 };
            0
        } else {
            self.frame + 1
        };
    }
}

static mut TRACKER: Tracker = Tracker::empty();

static mut TIMERS: Timers = Timers { last_calls: None };

struct Timers {
    last_calls: Option<HashMap<String, Winstant>>,
}

impl Timers {
    fn init(&mut self) {
        self.last_calls = Some(HashMap::new());
    }

    fn run_action<F>(&mut self, key: String, action: F)
    where
        F: FnOnce(),
    {
        let now = Winstant::now();
        let map = self
            .last_calls
            .as_mut()
            .expect("Timers should be initialized");
        map.insert(key, now);
        action()
    }

    fn run_action_debounced<F>(&mut self, key: String, t: Duration, action: F)
    where
        F: FnOnce(),
    {
        let now = Winstant::now();
        let map = self
            .last_calls
            .as_ref()
            .expect("Timers should be initialized");
        let last_call = map.get(&key);
        match last_call {
            Some(last_call) if now > *last_call + t => self.run_action(key, action),
            None => self.run_action(key, action),
            _ => {}
        }
    }
}

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

fn draw_sqr_waveform(duty_cycle: u32, length: u32, amplitude: u32, x: i32, y: i32) {
    hline(x, y + amplitude as i32, duty_cycle);
    vline(x + duty_cycle as i32, y + 1, amplitude);
    hline(x + duty_cycle as i32, y + 1, length - duty_cycle);
}

static mut INPUTS: Inputs = Inputs::new();

unsafe fn prepare_pattern_screen() {
    INPUTS
        .listen(InputEvent::Button2Press, || {})
        .listen(InputEvent::ButtonDownPress, || {
            let cursor = TRACKER.cursor_tick;
            if INPUTS.is_button1_pressed() && TRACKER.selected_column == Column::Note {
                TIMERS.run_action_debounced(
                    "pitch_octave_down".to_string(),
                    Duration::from_millis(100),
                    || {
                        if let Some(note) = TRACKER.pattern[cursor as usize] {
                            if (note.index as u32) >= NOTES_PER_OCTAVE {
                                TRACKER.pattern[cursor as usize] = Some(Note {
                                    index: note.index - NOTES_PER_OCTAVE as usize,
                                    ..note
                                })
                            }
                        }
                    },
                )
            } else if INPUTS.is_button2_pressed() {
                TIMERS.run_action_debounced("play".to_string(), Duration::from_millis(200), || {
                    TRACKER.toggle_play()
                })
            } else if cursor < 15 {
                TIMERS.run_action_debounced(
                    "nav_down".to_string(),
                    Duration::from_millis(100),
                    || {
                        let new_cursor = cursor + 1;
                        TRACKER.cursor_tick = new_cursor;
                        if let Some(note) = TRACKER.pattern[new_cursor as usize] {
                            TRACKER.selected_instrument = note.instrument;
                        }
                    },
                )
            }
        })
        .listen(InputEvent::ButtonUpPress, || {
            let cursor = TRACKER.cursor_tick;
            if INPUTS.is_button1_pressed() && TRACKER.selected_column == Column::Note {
                TIMERS.run_action_debounced(
                    "pitch_octave_up".to_string(),
                    Duration::from_millis(100),
                    || {
                        if let Some(note) = TRACKER.pattern[cursor as usize] {
                            if (note.index as u32) < note_freq.len() as u32 - NOTES_PER_OCTAVE {
                                TRACKER.pattern[cursor as usize] = Some(Note {
                                    index: NOTES_PER_OCTAVE as usize + note.index,
                                    ..note
                                })
                            }
                        }
                    },
                )
            } else if cursor != 0 {
                TIMERS.run_action_debounced(
                    "nav_up".to_string(),
                    Duration::from_millis(100),
                    || {
                        let new_cursor = cursor - 1;
                        TRACKER.cursor_tick = new_cursor;
                        if let Some(note) = TRACKER.pattern[new_cursor as usize] {
                            TRACKER.selected_instrument = note.instrument;
                        }
                    },
                )
            }
        })
        .listen(InputEvent::Button1Press, || {
            let cursor = TRACKER.cursor_tick;
            if let None = TRACKER.pattern[cursor as usize] {
                TRACKER.pattern[cursor as usize] = Some(Note {
                    index: note_c3_index,
                    instrument: 0,
                })
            }
        })
        .listen(InputEvent::Button1DoublePress, || {
            let cursor = TRACKER.cursor_tick;
            if let Some(_) = TRACKER.pattern[cursor as usize] {
                TRACKER.pattern[cursor as usize] = None
            }
        })
        .listen(InputEvent::ButtonRightPress, || {
            if INPUTS.is_button1_pressed() {
                let cursor = TRACKER.cursor_tick;
                match TRACKER.selected_column {
                    Column::Note => TIMERS.run_action_debounced(
                        "pitch_up".to_string(),
                        Duration::from_millis(100),
                        || {
                            if let Some(note) = TRACKER.pattern[cursor as usize] {
                                if note.index < note_freq.len() - 1 {
                                    TRACKER.pattern[cursor as usize] = Some(Note {
                                        index: note.index + 1,
                                        ..note
                                    })
                                }
                            }
                        },
                    ),
                    Column::Instrument => TIMERS.run_action_debounced(
                        "instrument_next".to_string(),
                        Duration::from_millis(200),
                        || {
                            if let Some(note) = TRACKER.pattern[cursor as usize] {
                                if note.instrument < 255 {
                                    TRACKER.pattern[cursor as usize] = Some(Note {
                                        instrument: note.instrument + 1,
                                        ..note
                                    })
                                }
                            }
                        },
                    ),
                }
            } else if INPUTS.is_button2_pressed() {
                TIMERS.run_action_debounced(
                    "nav_to_instrument".to_string(),
                    Duration::from_millis(200),
                    || {
                        TRACKER.screen = Screen::Instrument;
                        INPUTS.unlisten();
                        prepare_instrument_screen();
                    },
                );
            } else if TRACKER.selected_column == Column::Note {
                TRACKER.selected_column = Column::Instrument;
            }
        })
        .listen(InputEvent::ButtonLeftPress, || {
            if INPUTS.is_button1_pressed() {
                let cursor = TRACKER.cursor_tick;
                match TRACKER.selected_column {
                    Column::Note => TIMERS.run_action_debounced(
                        "pitch_down".to_string(),
                        Duration::from_millis(100),
                        || {
                            if let Some(note) = TRACKER.pattern[cursor as usize] {
                                if note.index != 0 {
                                    TRACKER.pattern[cursor as usize] = Some(Note {
                                        index: note.index - 1,
                                        ..note
                                    })
                                }
                            }
                        },
                    ),
                    Column::Instrument => TIMERS.run_action_debounced(
                        "instrument_prev".to_string(),
                        Duration::from_millis(200),
                        || {
                            if let Some(note) = TRACKER.pattern[cursor as usize] {
                                if note.instrument > 0 {
                                    TRACKER.pattern[cursor as usize] = Some(Note {
                                        instrument: note.instrument - 1,
                                        ..note
                                    })
                                }
                            }
                        },
                    ),
                }
            } else if TRACKER.selected_column == Column::Instrument {
                TRACKER.selected_column = Column::Note;
            }
        });
}

unsafe fn prepare_instrument_screen() {
    INPUTS
        .listen(InputEvent::ButtonDownPress, || {
            if INPUTS.is_button2_pressed() {
                TIMERS.run_action_debounced("play".to_string(), Duration::from_millis(200), || {
                    TRACKER.toggle_play()
                })
            } else if INPUTS.is_button1_pressed() {
                TIMERS.run_action_debounced(
                    "instrument_value_down".to_string(),
                    Duration::from_millis(200),
                    || {
                        let selected_instrument =
                            &mut TRACKER.instruments[TRACKER.selected_instrument];

                        match TRACKER.instrument_focus {
                            InstrumentScreenInput::Attack => {
                                selected_instrument.attack =
                                    selected_instrument.attack.saturating_sub(0x10)
                            }
                            InstrumentScreenInput::Decay => {
                                selected_instrument.decay =
                                    selected_instrument.decay.saturating_sub(0x10)
                            }
                            InstrumentScreenInput::Sustain => {
                                selected_instrument.sustain =
                                    selected_instrument.sustain.saturating_sub(0x10)
                            }
                            InstrumentScreenInput::Release => {
                                selected_instrument.release =
                                    selected_instrument.release.saturating_sub(0x10)
                            }
                            _ => {}
                        }
                    },
                )
            } else {
                TIMERS.run_action_debounced(
                    "instrument_input_next".to_string(),
                    Duration::from_millis(200),
                    || {
                        TRACKER.instrument_focus = match TRACKER.instrument_focus {
                            InstrumentScreenInput::DutyCycle => InstrumentScreenInput::Attack,
                            InstrumentScreenInput::Attack => InstrumentScreenInput::Decay,
                            InstrumentScreenInput::Decay => InstrumentScreenInput::Sustain,
                            InstrumentScreenInput::Sustain => InstrumentScreenInput::Release,
                            InstrumentScreenInput::Release => InstrumentScreenInput::Release,
                        }
                    },
                )
            }
        })
        .listen(InputEvent::ButtonUpPress, || {
            if INPUTS.is_button1_pressed() {
                TIMERS.run_action_debounced(
                    "instrument_value_up".to_string(),
                    Duration::from_millis(200),
                    || {
                        let selected_instrument =
                            &mut TRACKER.instruments[TRACKER.selected_instrument];

                        match TRACKER.instrument_focus {
                            InstrumentScreenInput::Attack => {
                                selected_instrument.attack =
                                    selected_instrument.attack.saturating_add(0x10);
                            }
                            InstrumentScreenInput::Decay => {
                                selected_instrument.decay =
                                    selected_instrument.decay.saturating_add(0x10)
                            }
                            InstrumentScreenInput::Sustain => {
                                selected_instrument.sustain =
                                    selected_instrument.sustain.saturating_add(0x10)
                            }
                            InstrumentScreenInput::Release => {
                                selected_instrument.release =
                                    selected_instrument.release.saturating_add(0x10)
                            }
                            _ => {}
                        }
                    },
                )
            } else {
                TIMERS.run_action_debounced(
                    "instrument_input_prev".to_string(),
                    Duration::from_millis(200),
                    || {
                        TRACKER.instrument_focus = match TRACKER.instrument_focus {
                            InstrumentScreenInput::DutyCycle => InstrumentScreenInput::DutyCycle,
                            InstrumentScreenInput::Attack => InstrumentScreenInput::DutyCycle,
                            InstrumentScreenInput::Decay => InstrumentScreenInput::Attack,
                            InstrumentScreenInput::Sustain => InstrumentScreenInput::Decay,
                            InstrumentScreenInput::Release => InstrumentScreenInput::Sustain,
                        }
                    },
                )
            }
        })
        .listen(InputEvent::ButtonLeftPress, || {
            if INPUTS.is_button2_pressed() {
                TIMERS.run_action_debounced(
                    "nav_to_pattern".to_string(),
                    Duration::from_millis(200),
                    || {
                        TRACKER.screen = Screen::Pattern;
                        INPUTS.unlisten();
                        prepare_pattern_screen();
                    },
                );
            } else if INPUTS.is_button1_pressed() {
                TIMERS.run_action_debounced(
                    "instrument_value_prev".to_string(),
                    Duration::from_millis(200),
                    || {
                        let selected_instrument =
                            &mut TRACKER.instruments[TRACKER.selected_instrument];

                        match TRACKER.instrument_focus {
                            InstrumentScreenInput::DutyCycle => {
                                selected_instrument.duty_cycle =
                                    match selected_instrument.duty_cycle {
                                        DutyCycle::Eighth => DutyCycle::Eighth,
                                        DutyCycle::Fourth => DutyCycle::Eighth,
                                        DutyCycle::Half => DutyCycle::Fourth,
                                        DutyCycle::ThreeFourth => DutyCycle::Half,
                                    }
                            }
                            InstrumentScreenInput::Attack => {
                                selected_instrument.attack =
                                    selected_instrument.attack.saturating_sub(1)
                            }
                            InstrumentScreenInput::Decay => {
                                selected_instrument.decay =
                                    selected_instrument.decay.saturating_sub(1)
                            }
                            InstrumentScreenInput::Sustain => {
                                selected_instrument.sustain =
                                    selected_instrument.sustain.saturating_sub(1)
                            }
                            InstrumentScreenInput::Release => {
                                selected_instrument.release =
                                    selected_instrument.release.saturating_sub(1)
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
                        let selected_instrument =
                            &mut TRACKER.instruments[TRACKER.selected_instrument];

                        match TRACKER.instrument_focus {
                            InstrumentScreenInput::DutyCycle => {
                                selected_instrument.duty_cycle =
                                    match selected_instrument.duty_cycle {
                                        DutyCycle::Eighth => DutyCycle::Fourth,
                                        DutyCycle::Fourth => DutyCycle::Half,
                                        DutyCycle::Half => DutyCycle::ThreeFourth,
                                        DutyCycle::ThreeFourth => DutyCycle::ThreeFourth,
                                    }
                            }
                            InstrumentScreenInput::Attack => {
                                selected_instrument.attack =
                                    selected_instrument.attack.saturating_add(1);
                            }
                            InstrumentScreenInput::Decay => {
                                selected_instrument.decay =
                                    selected_instrument.decay.saturating_add(1)
                            }
                            InstrumentScreenInput::Sustain => {
                                selected_instrument.sustain =
                                    selected_instrument.sustain.saturating_add(1)
                            }
                            InstrumentScreenInput::Release => {
                                selected_instrument.release =
                                    selected_instrument.release.saturating_add(1)
                            }
                        }
                    },
                )
            }
        });
}

#[no_mangle]
unsafe fn start() {
    TRACKER = Tracker::new();
    TIMERS.init();
    prepare_pattern_screen();
}

#[no_mangle]
fn update() {
    set_color(Color::Primary);

    let cursor: u8;
    let selected_column: Column;
    let screen: Screen;
    let selected_instrument: usize;

    unsafe {
        cursor = TRACKER.cursor_tick;
        selected_column = TRACKER.selected_column;
        screen = TRACKER.screen;
        selected_instrument = TRACKER.selected_instrument;
    };

    match screen {
        Screen::Pattern => {
            for line in 0..16 {
                text(format!("{:0X}", line), 1, line * 10 + 1);
                let note: Option<Note>;
                unsafe {
                    note = TRACKER.pattern[line as usize];
                };
                let name = if let Some(note) = note {
                    note_to_render(usize::from(note.index))
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
                    format!("{:02X}", note.instrument)
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

            unsafe {
                let tick: i32 = TRACKER.tick.into();
                text(">", 11, tick * 10 + 1);
            }
        }

        Screen::Instrument => {
            set_color(Color::Primary);
            text(format!("Instrument {:02X}", selected_instrument), 10, 10);

            let instrument: Instrument;
            let focus: InstrumentScreenInput;
            unsafe {
                instrument = TRACKER.instruments[TRACKER.selected_instrument];
                focus = TRACKER.instrument_focus;
            };

            let value_column_x = 120;

            let duty_cycle_x = 10;
            let duty_cycle_y = 30;
            let text_size_x = value_column_x - 10;
            text("Duty cycle", duty_cycle_x, duty_cycle_y);
            if focus == InstrumentScreenInput::DutyCycle {
                rect(duty_cycle_x + text_size_x - 1, duty_cycle_y - 1, 18, 10);
                set_color(Color::Background);
            }
            match instrument.duty_cycle {
                DutyCycle::Eighth => {
                    draw_sqr_waveform(2, 16, 8, duty_cycle_x + text_size_x, duty_cycle_y - 1);
                }
                DutyCycle::Fourth => {
                    draw_sqr_waveform(4, 16, 8, duty_cycle_x + text_size_x, duty_cycle_y - 1);
                }
                DutyCycle::Half => {
                    draw_sqr_waveform(8, 16, 8, duty_cycle_x + text_size_x, duty_cycle_y - 1);
                }
                DutyCycle::ThreeFourth => {
                    draw_sqr_waveform(12, 16, 8, duty_cycle_x + text_size_x, duty_cycle_y - 1);
                }
            }
            if focus == InstrumentScreenInput::DutyCycle {
                set_color(Color::Primary);
            }

            let input = |x: i32, y: i32, label: &str, value: u8, id: InstrumentScreenInput| {
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
                instrument.attack,
                InstrumentScreenInput::Attack,
            );

            input(
                10,
                50,
                "Decay",
                instrument.decay,
                InstrumentScreenInput::Decay,
            );

            input(
                10,
                60,
                "Sustain",
                instrument.sustain,
                InstrumentScreenInput::Sustain,
            );

            input(
                10,
                70,
                "Release",
                instrument.release,
                InstrumentScreenInput::Release,
            );
        }

        _ => {
            set_color(Color::Primary);
            text("Screen is not\nimplemented", 10, 10);
        }
    }

    unsafe {
        TRACKER.update();
        INPUTS.tick();
    }

    Winstant::tick();
}
