use std::borrow::BorrowMut;

use crate::{
    channel::Channel,
    instrument::InstrumentInput,
    notes::{note_c3_index, note_freq, note_from_string, NOTES_PER_OCTAVE},
    screen::Screen,
    wasm4::{
        tone, trace, TONE_MODE1, TONE_MODE2, TONE_MODE3, TONE_MODE4, TONE_NOISE, TONE_PULSE1,
        TONE_PULSE2, TONE_TRIANGLE,
    },
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

#[derive(PartialEq, Clone, Copy)]
pub struct Row {
    pulse1: Option<usize>,
    pulse2: Option<usize>,
    triangle: Option<usize>,
    noise: Option<usize>,
}

const MAX_PATTERNS: usize = 0x1F;

impl Row {
    pub fn channel(&self, channel: &Channel) -> &Option<usize> {
        match channel {
            Channel::Pulse1 => &self.pulse1,
            Channel::Pulse2 => &self.pulse2,
            Channel::Triangle => &self.triangle,
            Channel::Noise => &self.noise,
        }
    }

    pub fn channel_mut(&mut self, channel: &Channel) -> &mut Option<usize> {
        match channel {
            Channel::Pulse1 => &mut self.pulse1,
            Channel::Pulse2 => &mut self.pulse2,
            Channel::Triangle => &mut self.triangle,
            Channel::Noise => &mut self.noise,
        }
    }

    pub fn set_channel_value(&mut self, channel: &Channel, value: Option<usize>) {
        match channel {
            Channel::Pulse1 => self.pulse1 = value,
            Channel::Pulse2 => self.pulse2 = value,
            Channel::Triangle => self.triangle = value,
            Channel::Noise => self.noise = value,
        }
    }

    pub fn increment_channel_value(&mut self, channel: &Channel) {
        match channel {
            Channel::Pulse1 => {
                self.pulse1 = self
                    .pulse1
                    .map(|a| if a < MAX_PATTERNS { a + 1 } else { a })
            }
            Channel::Pulse2 => {
                self.pulse2 = self
                    .pulse2
                    .map(|a| if a < MAX_PATTERNS { a + 1 } else { a })
            }
            Channel::Triangle => {
                self.triangle = self
                    .triangle
                    .map(|a| if a < MAX_PATTERNS { a + 1 } else { a })
            }
            Channel::Noise => {
                self.noise = self.noise.map(|a| if a < MAX_PATTERNS { a + 1 } else { a })
            }
        }
    }

    pub fn decrement_channel_value(&mut self, channel: &Channel) {
        match channel {
            Channel::Pulse1 => self.pulse1 = self.pulse1.map(|a| if a > 0 { a - 1 } else { 0 }),
            Channel::Pulse2 => self.pulse2 = self.pulse2.map(|a| if a > 0 { a - 1 } else { 0 }),
            Channel::Triangle => {
                self.triangle = self.triangle.map(|a| if a > 0 { a - 1 } else { 0 })
            }
            Channel::Noise => self.noise = self.noise.map(|a| if a > 0 { a - 1 } else { 0 }),
        }
    }
}

pub enum PlayMode {
    Song,
    Pattern,
    Idle,
}

pub struct Tracker {
    frame: u32,
    tick: u8,
    patterns: Vec<[Option<Note>; 16]>,
    cursor_tick: u8,
    play: PlayMode,
    selected_column: Column,
    instruments: [Instrument; 0x1F],
    screen: Screen,
    selected_instrument_index: usize,
    instrument_focus: InstrumentInput,
    selected_channel: Channel,
    song_cursor_row_index: usize,
    song: [Row; 4], // also bpm
    selected_pattern: usize,
    song_tick: usize,
}

impl Tracker {
    const fn empty() -> Self {
        Tracker {
            frame: 0,
            tick: 0,
            patterns: vec![],
            cursor_tick: 0,
            play: PlayMode::Idle,
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
            selected_channel: Channel::Pulse1,
            song_cursor_row_index: 0,
            song: [Row {
                pulse1: None,
                pulse2: None,
                triangle: None,
                noise: None,
            }; 4],
            selected_pattern: 0,
            song_tick: 0,
        }
    }

    pub fn new() -> Self {
        Tracker {
            patterns: vec![[None; 16]; MAX_PATTERNS],
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
        match self.play {
            PlayMode::Song => {
                let pattern_index: usize = self.tick.into();
                let song = self.song;
                let row = song[self.song_tick];
                if let Some(note) = row.pulse1.and_then(|pulse1_pattern_index| {
                    self.patterns[pulse1_pattern_index][pattern_index]
                }) {
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
                    );
                }

                if let Some(note) = row.pulse2.and_then(|pulse2_pattern_index| {
                    self.patterns[pulse2_pattern_index][pattern_index]
                }) {
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
                        TONE_PULSE2 | duty_cycle,
                    );
                }

                if let Some(note) = row.triangle.and_then(|triangle_pattern_index| {
                    self.patterns[triangle_pattern_index][pattern_index]
                }) {
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
                        TONE_TRIANGLE | duty_cycle,
                    );
                }

                if let Some(note) = row.noise.and_then(|noise_pattern_index| {
                    self.patterns[noise_pattern_index][pattern_index]
                }) {
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
                        TONE_NOISE | duty_cycle,
                    );
                }
            }
            PlayMode::Pattern => {
                let pattern_index: usize = self.tick.into();
                if let Some(note) = self.patterns[self.selected_pattern][pattern_index] {
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
                    );
                }
            }
            PlayMode::Idle => {}
        };
    }

    pub fn toggle_play(&mut self, mode: PlayMode) {
        match self.play {
            PlayMode::Song | PlayMode::Pattern => self.play = PlayMode::Idle,
            PlayMode::Idle => {
                self.song_tick = 0;
                self.tick = 0;
                self.frame = 0;
                self.play = mode
            }
        }
    }

    pub fn update(&mut self) {
        if let PlayMode::Idle = self.play {
            return;
        }

        if self.frame == 0 {
            self.play_tick();
        }
        self.frame = if self.frame == 7 {
            self.tick = if self.tick == 15 {
                if let PlayMode::Song = self.play {
                    self.song_tick = if self.song_tick == 3 {
                        0
                    } else {
                        self.song_tick + 1
                    }
                };
                0
            } else {
                self.tick + 1
            };
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
        &self.patterns[self.selected_pattern][self.cursor_tick as usize]
    }

    pub fn current_note_mut(&mut self) -> &mut Option<Note> {
        &mut self.patterns[self.selected_pattern][self.cursor_tick as usize]
    }

    pub fn set_current_note(&mut self, note: &Option<Note>) {
        self.patterns[self.selected_pattern][self.cursor_tick as usize] = *note
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
        self.patterns[self.selected_pattern]
            .get(index)
            .and_then(|a| *a)
    }

    pub fn selected_channel(&self) -> &Channel {
        &self.selected_channel
    }

    pub fn next_channel(&mut self) {
        self.selected_channel = self.selected_channel.next()
    }

    pub fn prev_channel(&mut self) {
        self.selected_channel = self.selected_channel.prev()
    }

    pub fn song_cursor_row(&self) -> usize {
        self.song_cursor_row_index
    }

    pub fn next_row_song_cursor(&mut self) {
        self.song_cursor_row_index = match self.song_cursor_row_index {
            x @ 0..=2 => x + 1,
            _ => 3,
        }
    }

    pub fn prev_row_song_cursor(&mut self) {
        self.song_cursor_row_index = match self.song_cursor_row_index {
            0 => 0,
            x @ _ => x - 1,
        }
    }

    pub fn song(&self) -> &[Row; 4] {
        &self.song
    }

    pub fn song_mut(&mut self) -> &mut [Row; 4] {
        &mut self.song
    }

    pub fn selected_pattern(&self) -> usize {
        self.selected_pattern
    }

    pub fn set_selected_pattern(&mut self, index: usize) {
        self.selected_pattern = index;
    }

    pub fn play_mode(&self) -> &PlayMode {
        &self.play
    }

    pub fn song_tick(&self) -> usize {
        self.song_tick
    }
}

pub static mut TRACKER: Tracker = Tracker::empty();
