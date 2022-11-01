use std::borrow::BorrowMut;

use crate::{
    instrument::InstrumentInput,
    notes::{note_c3_index, note_freq, note_from_string, NOTES_PER_OCTAVE},
    screen::Screen,
    wasm4::{tone, trace, TONE_MODE1, TONE_MODE2, TONE_MODE3, TONE_MODE4, TONE_PULSE1},
};

#[derive(Clone, Copy)]
pub struct Note {
    index: usize,
    instrument: usize,
}

impl Note {
    pub fn new() -> Self {
        Note {
            index: note_c3_index,
            instrument: 0,
        }
    }

    pub fn increase_pitch(&mut self) {
        if self.index < note_freq.len() - 1 {
            self.index += 1;
        }
    }

    pub fn decrease_pitch(&mut self) {
        if self.index > 0 {
            self.index -= 1;
        }
    }

    pub fn increase_octave(&mut self) {
        let max_value: usize = note_freq.len() - NOTES_PER_OCTAVE as usize;
        if self.index < max_value {
            self.index = self.index + NOTES_PER_OCTAVE as usize;
        } else {
            self.index = note_freq.len();
        }
    }

    pub fn decrease_octave(&mut self) {
        if (self.index as u32) >= NOTES_PER_OCTAVE {
            self.index = self.index - NOTES_PER_OCTAVE as usize;
        } else {
            self.index = 0;
        }
    }

    pub fn next_instrument(&mut self) {
        if self.instrument < 0x1F {
            self.instrument += 1;
        }
    }

    pub fn prev_instrument(&mut self) {
        if self.instrument > 0 {
            self.instrument -= 1;
        }
    }

    pub fn instrument_index(&self) -> usize {
        self.instrument
    }

    pub fn note_index(&self) -> usize {
        self.index
    }
}

#[derive(Clone, Copy, Default)]
pub enum DutyCycle {
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

    pub fn next(&self) -> Self {
        match self {
            DutyCycle::Eighth => DutyCycle::Fourth,
            DutyCycle::Fourth => DutyCycle::Half,
            DutyCycle::Half => DutyCycle::ThreeFourth,
            DutyCycle::ThreeFourth => DutyCycle::ThreeFourth,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            DutyCycle::Eighth => DutyCycle::Eighth,
            DutyCycle::Fourth => DutyCycle::Eighth,
            DutyCycle::Half => DutyCycle::Fourth,
            DutyCycle::ThreeFourth => DutyCycle::Half,
        }
    }
}

#[derive(Clone, Copy, Default)]
pub struct Instrument {
    duty_cycle: DutyCycle,
    attack: u8,
    decay: u8,
    sustain: u8,
    release: u8,
}

impl Instrument {
    pub fn duty_cycle(&self) -> DutyCycle {
        self.duty_cycle
    }

    pub fn attack(&self) -> u8 {
        self.attack
    }

    pub fn decay(&self) -> u8 {
        self.decay
    }

    pub fn sustain(&self) -> u8 {
        self.sustain
    }

    pub fn release(&self) -> u8 {
        self.release
    }

    pub fn update_duty_cycle<F>(&mut self, f: F)
    where
        F: FnOnce(DutyCycle) -> DutyCycle,
    {
        self.duty_cycle = f(self.duty_cycle)
    }

    pub fn update_attack<F>(&mut self, f: F)
    where
        F: FnOnce(u8) -> u8,
    {
        self.attack = f(self.attack)
    }

    pub fn update_decay<F>(&mut self, f: F)
    where
        F: FnOnce(u8) -> u8,
    {
        self.decay = f(self.decay)
    }

    pub fn update_sustain<F>(&mut self, f: F)
    where
        F: FnOnce(u8) -> u8,
    {
        self.sustain = f(self.sustain)
    }

    pub fn update_release<F>(&mut self, f: F)
    where
        F: FnOnce(u8) -> u8,
    {
        self.release = f(self.release)
    }
}

#[derive(PartialEq, Clone, Copy, Default)]
pub enum Column {
    #[default]
    Note,
    Instrument,
}

pub struct Tracker {
    frame: u32,
    tick: u8,
    pattern: [Option<Note>; 16],
    cursor_tick: u8,
    playing: bool,
    selected_column: Column,
    instruments: [Instrument; 0x1F],
    screen: Screen,
    selected_instrument_index: usize,
    instrument_focus: InstrumentInput,
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
            selected_instrument_index: 0,
            instrument_focus: InstrumentInput::DutyCycle,
        }
    }

    pub fn new() -> Self {
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

    pub fn tick(&self) -> u8 {
        self.tick
    }

    pub fn screen(&self) -> Screen {
        self.screen
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

    pub fn toggle_play(&mut self) {
        if !self.playing {
            self.tick = 0;
            self.frame = 0;
        }
        self.playing = !self.playing;
    }

    pub fn update(&mut self) {
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

    pub fn cursor_tick(&self) -> u8 {
        self.cursor_tick
    }

    pub fn saturating_increase_cursor_tick(&mut self) {
        if self.cursor_tick < 15 {
            self.cursor_tick += 1
        }
    }

    pub fn saturating_decrease_cursor_tick(&mut self) {
        if self.cursor_tick != 0 {
            self.cursor_tick -= 1
        }
    }

    pub fn selected_column(&self) -> Column {
        self.selected_column
    }

    pub fn set_selected_column(&mut self, column: Column) {
        self.selected_column = column
    }

    pub fn selected_instrument(&self) -> &Instrument {
        &self.instruments[self.selected_instrument_index]
    }

    pub fn selected_instrument_mut(&mut self) -> &mut Instrument {
        &mut self.instruments[self.selected_instrument_index]
    }

    pub fn selected_instrument_index(&self) -> usize {
        self.selected_instrument_index
    }

    pub fn set_selected_instrument_index(&mut self, index: usize) {
        if index > 0x1F {
            panic!("Trying to set instrument index > 0x1f")
        }

        self.selected_instrument_index = index;
    }

    pub fn instrument_focus(&self) -> InstrumentInput {
        self.instrument_focus
    }

    pub fn instrument_focus_next(&mut self) {
        self.instrument_focus = match self.instrument_focus {
            InstrumentInput::DutyCycle => InstrumentInput::Attack,
            InstrumentInput::Attack => InstrumentInput::Decay,
            InstrumentInput::Decay => InstrumentInput::Sustain,
            InstrumentInput::Sustain => InstrumentInput::Release,
            InstrumentInput::Release => InstrumentInput::Release,
        }
    }

    pub fn instrument_focus_prev(&mut self) {
        self.instrument_focus = match self.instrument_focus {
            InstrumentInput::DutyCycle => InstrumentInput::DutyCycle,
            InstrumentInput::Attack => InstrumentInput::DutyCycle,
            InstrumentInput::Decay => InstrumentInput::Attack,
            InstrumentInput::Sustain => InstrumentInput::Decay,
            InstrumentInput::Release => InstrumentInput::Sustain,
        }
    }

    pub fn set_screen(&mut self, screen: Screen) {
        self.screen = screen;
    }

    pub fn current_note(&self) -> &Option<Note> {
        &self.pattern[self.cursor_tick as usize]
    }

    pub fn current_note_mut(&mut self) -> &mut Option<Note> {
        &mut self.pattern[self.cursor_tick as usize]
    }

    pub fn set_current_note(&mut self, note: &Option<Note>) {
        self.pattern[self.cursor_tick as usize] = *note
    }

    pub fn update_current_note<F>(&mut self, f: F)
    where
        F: FnOnce(&Option<Note>) -> &Option<Note>,
    {
        let current_note = self.current_note().clone();
        let new_note = f(&current_note);
        self.set_current_note(new_note)
    }

    pub fn note_at(&self, index: usize) -> Option<Note> {
        self.pattern.get(index).and_then(|a| *a)
    }
}

pub static mut TRACKER: Tracker = Tracker::empty();
