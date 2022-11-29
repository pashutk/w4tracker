use std::time::Duration;

use crate::{
    inputs::{InputEvent, Inputs},
    navigation::go_to_pattern_screen,
    screen::Screen,
    timers::{ActionId, TIMERS},
    tracker::{PlayMode, TRACKER},
    wasm4::{TONE_MODE1, TONE_MODE2, TONE_MODE3, TONE_MODE4},
};

pub const MAX_INSTRUMENTS: usize = 0x20;

#[derive(Clone, Copy, Default)]
pub enum DutyCycle {
    #[default]
    Eighth,
    Fourth,
    Half,
    ThreeFourth,
}

impl DutyCycle {
    pub fn to_flag(&self) -> u32 {
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
    volume: u8,
    peak: u8,
}

const MAX_VOLUME: u8 = 0x64;
const MAX_PEAK: u8 = 0x64;

impl Instrument {
    pub const fn new(
        duty_cycle: DutyCycle,
        attack: u8,
        decay: u8,
        sustain: u8,
        release: u8,
        volume: u8,
        peak: u8,
    ) -> Instrument {
        Instrument {
            duty_cycle,
            attack,
            decay,
            sustain,
            release,
            volume,
            peak,
        }
    }

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

    pub fn volume(&self) -> u8 {
        self.volume
    }

    pub fn peak(&self) -> u8 {
        self.peak
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

    pub fn update_volume<F>(&mut self, f: F)
    where
        F: FnOnce(u8) -> u8,
    {
        self.volume = f(self.volume).clamp(0, MAX_VOLUME);
    }

    pub fn update_peak<F>(&mut self, f: F)
    where
        F: FnOnce(u8) -> u8,
    {
        self.peak = f(self.peak).clamp(0, MAX_PEAK);
    }

    pub fn to_bytes(&self, api_version: u8) -> Vec<u8> {
        match api_version {
            1 => {
                let mut v = vec![0_u8; 7];
                v[0] = match self.duty_cycle {
                    DutyCycle::Eighth => 0,
                    DutyCycle::Fourth => 1,
                    DutyCycle::Half => 2,
                    DutyCycle::ThreeFourth => 3,
                };
                v[1] = self.attack;
                v[2] = self.decay;
                v[3] = self.release;
                v[4] = self.sustain;
                v[5] = self.volume;
                v[6] = self.peak;
                v
            }
            _ => panic!("Unsupported api version"),
        }
    }

    pub fn from_bytes(bytes: (u8, u8, u8, u8, u8, u8, u8)) -> Self {
        Instrument {
            duty_cycle: match bytes.0 {
                0 => DutyCycle::Eighth,
                1 => DutyCycle::Fourth,
                2 => DutyCycle::Half,
                3 => DutyCycle::ThreeFourth,
                _ => DutyCycle::Eighth,
            },
            attack: bytes.1,
            decay: bytes.2,
            sustain: bytes.3,
            release: bytes.4,
            volume: bytes.5,
            peak: bytes.6,
        }
    }

    pub fn get_duration(&self) -> u32 {
        (self.attack as u32) << 24
            | (self.decay as u32) << 16
            | self.sustain as u32
            | (self.release as u32) << 8
    }

    pub fn get_volume(&self) -> u32 {
        (self.peak as u32) << 8 | self.volume as u32
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum InstrumentInput {
    DutyCycle,
    Attack,
    Decay,
    Sustain,
    Release,
    Volume,
    Peak,
}

fn on_button_down_press(inputs: &Inputs) {
    unsafe {
        if inputs.is_button2_pressed() {
            TIMERS.run_action_debounced(ActionId::Play, Duration::from_millis(200), || {
                TRACKER.toggle_play(PlayMode::Pattern)
            })
        } else if inputs.is_button1_pressed() {
            TIMERS.run_action_debounced(
                ActionId::InstrumentValueDownActionId,
                Duration::from_millis(200),
                || {
                    let selected_instrument = TRACKER.selected_instrument_mut();

                    match TRACKER.instrument_focus() {
                        InstrumentInput::Attack => {
                            selected_instrument.update_attack(|a| a.saturating_sub(0x10))
                        }
                        InstrumentInput::Decay => {
                            selected_instrument.update_decay(|a| a.saturating_sub(0x10))
                        }
                        InstrumentInput::Sustain => {
                            selected_instrument.update_sustain(|a| a.saturating_sub(0x10))
                        }
                        InstrumentInput::Release => {
                            selected_instrument.update_release(|a| a.saturating_sub(0x10))
                        }
                        InstrumentInput::Volume => {
                            selected_instrument.update_volume(|a| a.saturating_sub(0x10))
                        }
                        InstrumentInput::Peak => {
                            selected_instrument.update_peak(|a| a.saturating_sub(0x10))
                        }
                        InstrumentInput::DutyCycle => {}
                    }
                },
            )
        } else {
            TIMERS.run_action_debounced(
                ActionId::InstrumentNextInputActionId,
                Duration::from_millis(200),
                || TRACKER.instrument_focus_next(),
            )
        }
    }
}

fn on_button_up_press(inputs: &Inputs) {
    unsafe {
        if inputs.is_button1_pressed() {
            TIMERS.run_action_debounced(
                ActionId::InstrumentValueUpActionId,
                Duration::from_millis(200),
                || {
                    let selected_instrument = TRACKER.selected_instrument_mut();

                    match TRACKER.instrument_focus() {
                        InstrumentInput::Attack => {
                            selected_instrument.update_attack(|a| a.saturating_add(0x10))
                        }
                        InstrumentInput::Decay => {
                            selected_instrument.update_decay(|a| a.saturating_add(0x10))
                        }
                        InstrumentInput::Sustain => {
                            selected_instrument.update_sustain(|a| a.saturating_add(0x10))
                        }
                        InstrumentInput::Release => {
                            selected_instrument.update_release(|a| a.saturating_add(0x10))
                        }
                        InstrumentInput::Volume => {
                            selected_instrument.update_volume(|a| a.saturating_add(0x10))
                        }
                        InstrumentInput::Peak => {
                            selected_instrument.update_peak(|a| a.saturating_add(0x10))
                        }
                        InstrumentInput::DutyCycle => {}
                    }
                },
            )
        } else if inputs.is_button2_pressed() {
            TIMERS.run_action_debounced(ActionId::Persist, Duration::from_millis(1000), || {
                TRACKER.persist();
            })
        } else {
            TIMERS.run_action_debounced(
                ActionId::InstrumentPrevInputActionId,
                Duration::from_millis(200),
                || TRACKER.instrument_focus_prev(),
            )
        }
    }
}

fn on_button_left_press(inputs: &Inputs) {
    unsafe {
        if inputs.is_button2_pressed() {
            TIMERS.run_action_debounced(
                ActionId::NavPrevScreen,
                Duration::from_millis(200),
                || go_to_pattern_screen(Screen::Instrument),
            );
        } else if inputs.is_button1_pressed() {
            TIMERS.run_action_debounced(
                ActionId::InstrumentValuePrevActionId,
                Duration::from_millis(200),
                || {
                    let selected_instrument = TRACKER.selected_instrument_mut();

                    match TRACKER.instrument_focus() {
                        InstrumentInput::DutyCycle => {
                            selected_instrument.update_duty_cycle(|a| a.prev())
                        }
                        InstrumentInput::Attack => {
                            selected_instrument.update_attack(|a| a.saturating_sub(1))
                        }
                        InstrumentInput::Decay => {
                            selected_instrument.update_decay(|a| a.saturating_sub(1))
                        }
                        InstrumentInput::Sustain => {
                            selected_instrument.update_sustain(|a| a.saturating_sub(1))
                        }
                        InstrumentInput::Release => {
                            selected_instrument.update_release(|a| a.saturating_sub(1))
                        }
                        InstrumentInput::Volume => {
                            selected_instrument.update_volume(|a| a.saturating_sub(1))
                        }
                        InstrumentInput::Peak => {
                            selected_instrument.update_peak(|a| a.saturating_sub(1))
                        }
                    }
                },
            )
        }
    }
}

fn on_button_right_press(inputs: &Inputs) {
    unsafe {
        if inputs.is_button1_pressed() {
            TIMERS.run_action_debounced(
                ActionId::InstrumentValueNextActionId,
                Duration::from_millis(200),
                || {
                    let selected_instrument = TRACKER.selected_instrument_mut();

                    match TRACKER.instrument_focus() {
                        InstrumentInput::DutyCycle => {
                            selected_instrument.update_duty_cycle(|a| a.next())
                        }
                        InstrumentInput::Attack => {
                            selected_instrument.update_attack(|a| a.saturating_add(1))
                        }
                        InstrumentInput::Decay => {
                            selected_instrument.update_decay(|a| a.saturating_add(1))
                        }
                        InstrumentInput::Sustain => {
                            selected_instrument.update_sustain(|a| a.saturating_add(1))
                        }
                        InstrumentInput::Release => {
                            selected_instrument.update_release(|a| a.saturating_add(1))
                        }
                        InstrumentInput::Volume => {
                            selected_instrument.update_volume(|a| a.saturating_add(1))
                        }
                        InstrumentInput::Peak => {
                            selected_instrument.update_peak(|a| a.saturating_add(1))
                        }
                    }
                },
            )
        }
    }
}

pub fn add_instrument_screen_handlers(inputs: &mut Inputs) {
    inputs
        .listen(InputEvent::ButtonDownPress, on_button_down_press)
        .listen(InputEvent::ButtonUpPress, on_button_up_press)
        .listen(InputEvent::ButtonLeftPress, on_button_left_press)
        .listen(InputEvent::ButtonRightPress, on_button_right_press);
}
