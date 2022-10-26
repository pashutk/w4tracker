#[cfg(feature = "buddy-alloc")]
mod alloc;
mod inputs;
mod notes;
mod wasm4;

use std::collections::HashMap;

use inputs::{InputEvent, Inputs};
use notes::{note_c3_index, note_freq, note_from_string, note_to_render};
use wasm4::*;

struct Tracker {
    frame: u32,
    tick: u8,
    pattern: [Option<usize>; 16],
    cursor_tick: u8,
    playing: bool,
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
        }
    }

    fn new() -> Self {
        Tracker {
            tick: 0,
            frame: 0,
            cursor_tick: 0,
            playing: false,
            pattern: [
                Some(note_from_string("C3").unwrap()),
                Some(note_from_string("C3").unwrap()),
                Some(note_from_string("C4").unwrap()),
                None,
                Some(note_from_string("G3").unwrap()),
                None,
                None,
                Some(note_from_string("F#3").unwrap()),
                None,
                None,
                Some(note_from_string("F3").unwrap()),
                None,
                None,
                None,
                Some(note_from_string("D#3").unwrap()),
                None,
            ],
        }
    }

    fn play_tick(&self) {
        let pattern_index: usize = self.tick.into();
        if let Some(note) = self.pattern[pattern_index] {
            tone(note_freq[note].into(), 4 | (8 << 8), 100, TONE_PULSE1)
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

static mut TIMERS: Timers = Timers {
    tick: 0,
    last_calls: None,
};

struct Timers {
    tick: u32,
    last_calls: Option<HashMap<String, u32>>,
}

impl Timers {
    fn init(&mut self) {
        self.last_calls = Some(HashMap::new());
    }

    fn tick(&mut self) {
        self.tick += 1;
    }

    fn run_action<F>(&mut self, key: String, action: F)
    where
        F: FnOnce(),
    {
        let map = self
            .last_calls
            .as_mut()
            .expect("Timers should be initialized");
        map.insert(key, self.tick);
        action()
    }

    fn run_action_debounced<F>(&mut self, key: String, t: u32, action: F)
    where
        F: FnOnce(),
    {
        let map = self
            .last_calls
            .as_ref()
            .expect("Timers should be initialized");
        let last_call = map.get(&key).map_or(0, |a| *a);
        if self.tick - last_call > t {
            self.run_action(key, action)
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
                TIMERS.run_action_debounced("play".to_string(), 12, || TRACKER.toggle_play())
            })
            .listen(InputEvent::ButtonDownPress, || {
                let cursor = TRACKER.cursor_tick;
                if cursor < 15 {
                    TIMERS.run_action_debounced("nav_down".to_string(), 4, || {
                        TRACKER.cursor_tick = cursor + 1
                    })
                }
            })
            .listen(InputEvent::ButtonUpPress, || {
                let cursor = TRACKER.cursor_tick;
                if cursor != 0 {
                    TIMERS.run_action_debounced("nav_up".to_string(), 4, || {
                        TRACKER.cursor_tick = cursor - 1
                    })
                }
            })
            .listen(InputEvent::Button1Press, || {
                let cursor = TRACKER.cursor_tick;
                if let None = TRACKER.pattern[cursor as usize] {
                    TRACKER.pattern[cursor as usize] = Some(note_c3_index)
                }
            })
            .listen(InputEvent::ButtonRightPress, || {
                if INPUTS.is_button1_pressed() {
                    let cursor = TRACKER.cursor_tick;
                    TIMERS.run_action_debounced("pitch_up".to_string(), 4, || {
                        if let Some(note) = TRACKER.pattern[cursor as usize] {
                            if note < note_freq.len() - 1 {
                                TRACKER.pattern[cursor as usize] = Some(note + 1)
                            }
                        }
                    })
                }
            })
            .listen(InputEvent::ButtonLeftPress, || {
                if INPUTS.is_button1_pressed() {
                    TIMERS.run_action_debounced("pitch_down".to_string(), 4, || {
                        let cursor = TRACKER.cursor_tick;
                        if let Some(note) = TRACKER.pattern[cursor as usize] {
                            if note != 0 {
                                TRACKER.pattern[cursor as usize] = Some(note - 1)
                            }
                        }
                    })
                }
            });
    }
}

#[no_mangle]
fn update() {
    set_color(Color::Primary);

    let cursor: u8;
    unsafe { cursor = TRACKER.cursor_tick };

    for line in 0..16 {
        text(format!("{:0X}", line), 1, line * 10 + 1);
        let note: Option<usize>;
        unsafe {
            note = TRACKER.pattern[line as usize];
        };
        let name = if let Some(index) = note {
            note_to_render(usize::from(index))
        } else {
            "---".to_string()
        };

        if line == cursor.into() {
            rect(20, line * 10, 8 * 3 + 2, 10);
            set_color(Color::Background);
            text(name, 21, line * 10 + 1);
            set_color(Color::Primary);
        } else {
            text(name, 21, line * 10 + 1);
        };
    }

    set_color(Color::Light);
    text("nav:   arrows", 50, 54);
    text("play/stop:  Z", 50, 64);
    text("add note:   X", 50, 74);
    text("rm note:   XX", 50, 84);
    text("pitch:  X+L/R", 50, 94);
    set_color(Color::Primary);

    unsafe {
        let tick: i32 = TRACKER.tick.into();
        text(">", 11, tick * 10 + 1);

        TRACKER.update();
        INPUTS.tick();
    }
    unsafe { TIMERS.tick() }
}
