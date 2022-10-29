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

#[derive(Default)]
struct Tracker {
    frame: u32,
    tick: u8,
    pattern: [Option<Note>; 16],
    cursor_tick: u8,
    playing: bool,
    selected_column: Column,
    instruments: [Instrument; 0x1F],
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
                sustain: 0xff,
                release: 0,
            }; 0x1f],
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
            tone(
                note_freq[note.index].into(),
                4 | (8 << 8),
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

static mut INPUTS: Inputs = Inputs::new();

#[no_mangle]
fn start() {
    unsafe {
        TRACKER = Tracker::new();
        TIMERS.init();
        INPUTS
            .listen(InputEvent::Button2Press, || {
                TIMERS.run_action_debounced("play".to_string(), Duration::from_millis(200), || {
                    TRACKER.toggle_play()
                })
            })
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
                } else if cursor < 15 {
                    TIMERS.run_action_debounced(
                        "nav_down".to_string(),
                        Duration::from_millis(100),
                        || TRACKER.cursor_tick = cursor + 1,
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
                        || TRACKER.cursor_tick = cursor - 1,
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
}

#[no_mangle]
fn update() {
    set_color(Color::Primary);

    let cursor: u8;
    unsafe { cursor = TRACKER.cursor_tick };

    let selected_column: Column;
    unsafe { selected_column = TRACKER.selected_column };

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
    let first_row_y = 108;
    text_bytes(b"nav:   \x84\x85\x86\x87", 70, first_row_y + 10 * 0);
    text_bytes(b"play/stop:\x81", 70, first_row_y + 10 * 1);
    text_bytes(b"add note: \x80", 70, first_row_y + 10 * 2);
    text_bytes(b"rm note: \x80\x80", 70, first_row_y + 10 * 3);
    text_bytes(b"edit:\x80+\x84\x85\x86\x87", 70, first_row_y + 10 * 4);

    set_color(Color::Primary);

    unsafe {
        let tick: i32 = TRACKER.tick.into();
        text(">", 11, tick * 10 + 1);

        TRACKER.update();
        INPUTS.tick();
    }

    Winstant::tick();
}
