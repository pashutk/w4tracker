use std::{mem::size_of, ptr::addr_of};

use crate::{
    channel::Channel,
    instrument::{DutyCycle, Instrument, InstrumentInput, MAX_INSTRUMENTS},
    notes::{Note, NOTE_FREQ},
    screen::Screen,
    wasm4::{diskr, diskw, tone, TONE_NOISE, TONE_PULSE1, TONE_PULSE2, TONE_TRIANGLE},
};

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

const MAX_PATTERNS: usize = 0x10;

impl Row {
    pub fn channel(&self, channel: &Channel) -> &Option<usize> {
        match channel {
            Channel::Pulse1 => &self.pulse1,
            Channel::Pulse2 => &self.pulse2,
            Channel::Triangle => &self.triangle,
            Channel::Noise => &self.noise,
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
                    .map(|a| if a < MAX_PATTERNS - 1 { a + 1 } else { a })
            }
            Channel::Pulse2 => {
                self.pulse2 = self
                    .pulse2
                    .map(|a| if a < MAX_PATTERNS - 1 { a + 1 } else { a })
            }
            Channel::Triangle => {
                self.triangle = self
                    .triangle
                    .map(|a| if a < MAX_PATTERNS - 1 { a + 1 } else { a })
            }
            Channel::Noise => {
                self.noise = self
                    .noise
                    .map(|a| if a < MAX_PATTERNS - 1 { a + 1 } else { a })
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

    pub fn to_bytes(&self, api_version: u8) -> Vec<u8> {
        match api_version {
            1 => {
                let mut v = vec![0_u8; 4];
                v[0] = self.pulse1.unwrap_or(255).try_into().unwrap();
                v[1] = self.pulse2.unwrap_or(255).try_into().unwrap();
                v[2] = self.triangle.unwrap_or(255).try_into().unwrap();
                v[3] = self.noise.unwrap_or(255).try_into().unwrap();
                v
            }
            _ => panic!("Unsupported api version"),
        }
    }
}

pub enum PlayMode {
    Song,
    Pattern,
    Idle,
}

const SONG_SIZE: usize = 4;

type Song = [Row; SONG_SIZE];

pub struct Tracker {
    frame: u32,
    tick: u8,
    patterns: Vec<[Option<Note>; 16]>, // save - 2b note * 16 * MAX_PATTERNS = 2 * 16 * 16 = 512b
    cursor_tick: u8,
    play: PlayMode,
    selected_column: Column,
    instruments: [Instrument; MAX_INSTRUMENTS], // save - 7 * 32 = 224b
    screen: Screen,
    selected_instrument_index: usize,
    instrument_focus: InstrumentInput,
    selected_channel: Channel,
    song_cursor_row_index: usize,
    song: Song, // save Song.len() * 4
    selected_pattern: usize,
    song_tick: usize,
}

const STORAGE_LAYOUT_VERSION: u8 = 1;

impl Tracker {
    const fn empty() -> Self {
        Tracker {
            frame: 0,
            tick: 0,
            patterns: vec![],
            cursor_tick: 0,
            play: PlayMode::Idle,
            selected_column: Column::Note,
            instruments: [Instrument::new(DutyCycle::Eighth, 0, 0, 0x0f, 0x0f, 0x64, 0x64);
                MAX_INSTRUMENTS],
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
            }; SONG_SIZE],
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
                    let duty_cycle = instrument.duty_cycle().to_flag();
                    tone(
                        NOTE_FREQ[note.index].into(),
                        instrument.get_duration(),
                        instrument.get_volume(),
                        TONE_PULSE1 | duty_cycle,
                    );
                }

                if let Some(note) = row.pulse2.and_then(|pulse2_pattern_index| {
                    self.patterns[pulse2_pattern_index][pattern_index]
                }) {
                    let instrument = self.instruments[note.instrument];
                    let duty_cycle = instrument.duty_cycle().to_flag();
                    tone(
                        NOTE_FREQ[note.index].into(),
                        instrument.get_duration(),
                        instrument.get_volume(),
                        TONE_PULSE2 | duty_cycle,
                    );
                }

                if let Some(note) = row.triangle.and_then(|triangle_pattern_index| {
                    self.patterns[triangle_pattern_index][pattern_index]
                }) {
                    let instrument = self.instruments[note.instrument];
                    let duty_cycle = instrument.duty_cycle().to_flag();
                    tone(
                        NOTE_FREQ[note.index].into(),
                        instrument.get_duration(),
                        instrument.get_volume(),
                        TONE_TRIANGLE | duty_cycle,
                    );
                }

                if let Some(note) = row.noise.and_then(|noise_pattern_index| {
                    self.patterns[noise_pattern_index][pattern_index]
                }) {
                    let instrument = self.instruments[note.instrument];
                    let duty_cycle = instrument.duty_cycle().to_flag();
                    tone(
                        NOTE_FREQ[note.index].into(),
                        instrument.get_duration(),
                        instrument.get_volume(),
                        TONE_NOISE | duty_cycle,
                    );
                }
            }
            PlayMode::Pattern => {
                let pattern_index: usize = self.tick.into();
                if let Some(note) = self.patterns[self.selected_pattern][pattern_index] {
                    let instrument = self.instruments[note.instrument];
                    let duty_cycle = instrument.duty_cycle().to_flag();
                    let channel = self.selected_channel;
                    tone(
                        NOTE_FREQ[note.index].into(),
                        instrument.get_duration(),
                        instrument.get_volume(),
                        match channel {
                            Channel::Pulse1 => TONE_PULSE1,
                            Channel::Pulse2 => TONE_PULSE2,
                            Channel::Triangle => TONE_TRIANGLE,
                            Channel::Noise => TONE_NOISE,
                        } | duty_cycle,
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
        if index > MAX_INSTRUMENTS {
            panic!("Trying to set instrument index > MAX_INSTRUMENTS")
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
            InstrumentInput::Release => InstrumentInput::Volume,
            InstrumentInput::Volume => InstrumentInput::Peak,
            InstrumentInput::Peak => InstrumentInput::Peak,
        }
    }

    pub fn instrument_focus_prev(&mut self) {
        self.instrument_focus = match self.instrument_focus {
            InstrumentInput::DutyCycle => InstrumentInput::DutyCycle,
            InstrumentInput::Attack => InstrumentInput::DutyCycle,
            InstrumentInput::Decay => InstrumentInput::Attack,
            InstrumentInput::Sustain => InstrumentInput::Decay,
            InstrumentInput::Release => InstrumentInput::Sustain,
            InstrumentInput::Volume => InstrumentInput::Release,
            InstrumentInput::Peak => InstrumentInput::Volume,
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
        const LAST_TO_MOVE: usize = SONG_SIZE - 2;
        self.song_cursor_row_index = match self.song_cursor_row_index {
            x @ 0..=LAST_TO_MOVE => x + 1,
            _ => SONG_SIZE - 1,
        }
    }

    pub fn prev_row_song_cursor(&mut self) {
        self.song_cursor_row_index = match self.song_cursor_row_index {
            0 => 0,
            x @ _ => x - 1,
        }
    }

    pub fn song(&self) -> &Song {
        &self.song
    }

    pub fn song_mut(&mut self) -> &mut Song {
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

    pub fn persist(&self) {
        let layout_version_section_size: usize = 1;
        let song_section_size = self.song.len() * 4;
        let instrumens_section_size = self.instruments.len() * size_of::<Instrument>();
        let patterns_section_size = self.patterns.len() * 16 * 2;
        let stored_size = layout_version_section_size
            + song_section_size
            + instrumens_section_size
            + patterns_section_size;

        let mut buf = vec![0_u8; stored_size];
        let mut next_byte: usize = 0;

        // storage version (1)
        buf[0] = STORAGE_LAYOUT_VERSION;
        next_byte += 1;

        // song (song.len()*4)
        for row in self.song {
            let row_bytes = row.to_bytes(STORAGE_LAYOUT_VERSION);
            for byte in row_bytes {
                buf[next_byte] = byte;
                next_byte += 1;
            }
        }

        // instruments (MAX_INSTRUMENTS * 5)
        for instrument in self.instruments {
            let instrument_bytes = instrument.to_bytes(STORAGE_LAYOUT_VERSION);
            for byte in instrument_bytes {
                buf[next_byte] = byte;
                next_byte += 1;
            }
        }

        // patterns (MAX_PATTERNS * 16 * 2 (note size))
        for pattern in &self.patterns {
            for note in pattern {
                let bytes = match note {
                    Some(note) => note.to_bytes(),
                    None => (0xff, 0xff),
                };
                buf[next_byte] = bytes.0;
                buf[next_byte + 1] = bytes.1;
                next_byte += 2;
            }
        }

        unsafe {
            diskw(addr_of!(buf.as_slice()[0]), stored_size as u32);
        }
    }

    pub fn restore() -> Tracker {
        let mut tracker = Tracker::new();

        let mut buf = [0u8; 1
            + SONG_SIZE * 4
            + MAX_INSTRUMENTS * size_of::<Instrument>()
            + MAX_PATTERNS * 16 * 2];

        unsafe {
            diskr(buf.as_mut_ptr(), buf.len() as u32);
        }

        let mut next_byte: usize = 0;

        // storage version (1)
        let version = buf[next_byte];
        if version != STORAGE_LAYOUT_VERSION {
            return tracker;
        }
        next_byte += 1;

        // song (4*4)
        for row_index in 0..SONG_SIZE {
            let pulse1: Option<usize> = if let 255 = buf[next_byte + 0] {
                None
            } else {
                Some(buf[next_byte + 0].into())
            };
            let pulse2: Option<usize> = if let 255 = buf[next_byte + 1] {
                None
            } else {
                Some(buf[next_byte + 1].into())
            };
            let triangle: Option<usize> = if let 255 = buf[next_byte + 2] {
                None
            } else {
                Some(buf[next_byte + 2].into())
            };
            let noise: Option<usize> = if let 255 = buf[next_byte + 3] {
                None
            } else {
                Some(buf[next_byte + 3].into())
            };
            next_byte += 4;

            let row = Row {
                pulse1,
                pulse2,
                triangle,
                noise,
            };
            tracker.song[row_index] = row;
        }

        // instruments (MAX_INSTRUMENTS * 5)
        for instrument_index in 0..MAX_INSTRUMENTS {
            let bytes = (
                buf[next_byte + 0],
                buf[next_byte + 1],
                buf[next_byte + 2],
                buf[next_byte + 3],
                buf[next_byte + 4],
                buf[next_byte + 5],
                buf[next_byte + 6],
            );
            next_byte += 7;
            let instrument = Instrument::from_bytes(bytes);
            tracker.instruments[instrument_index] = instrument;
        }

        // patterns (MAX_PATTERNS * 16 * 2 (note size))
        for pattern_index in 0..MAX_PATTERNS {
            for note_index in 0..16 {
                let bytes = (buf[next_byte + 0], buf[next_byte + 1]);
                next_byte += 2;
                let note = match bytes {
                    (0xff, 0xff) => None,
                    (index, instrument) => Some(Note {
                        index: index.into(),
                        instrument: instrument.into(),
                    }),
                };
                tracker.patterns[pattern_index][note_index] = note;
            }
        }

        tracker
    }
}

pub static mut TRACKER: Tracker = Tracker::empty();
