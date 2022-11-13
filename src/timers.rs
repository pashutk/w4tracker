use std::{collections::HashMap, time::Duration};

use crate::wtime::Winstant;

pub static mut TIMERS: Timers = Timers { last_calls: None };

#[derive(Eq, Hash, PartialEq)]
pub enum ActionId {
    Play,
    Persist,
    NavNextScreen,
    NavPrevScreen,

    InstrumentValueDownActionId,
    InstrumentNextInputActionId,
    InstrumentValueUpActionId,
    InstrumentPrevInputActionId,
    InstrumentValuePrevActionId,
    InstrumentValueNextActionId,

    PatternPitchOctaveDown,
    PatternNavDown,
    PatternPitchOctaveUp,
    PatternNavUp,
    PatternPitchUp,
    PatternInstrumentNext,
    PatternPitchDown,
    PatternInstrumentPrev,

    SongNextRow,
    SongPrevRow,
    SongDecrementPattern,
    SongPrevChannel,
    SongIncrementPattern,
    SongNextChannel,
    SongAddPattern,
}

pub struct Timers {
    last_calls: Option<HashMap<ActionId, Winstant>>,
}

impl Timers {
    pub fn init(&mut self) {
        self.last_calls = Some(HashMap::new());
    }

    fn run_action<F>(&mut self, action_id: ActionId, action: F)
    where
        F: FnOnce(),
    {
        let now = Winstant::now();
        let map = self
            .last_calls
            .as_mut()
            .expect("Timers should be initialized");
        map.insert(action_id, now);
        action()
    }

    pub fn run_action_debounced<F>(&mut self, action_id: ActionId, t: Duration, action: F)
    where
        F: FnOnce(),
    {
        let now = Winstant::now();
        let map = self
            .last_calls
            .as_ref()
            .expect("Timers should be initialized");
        let last_call = map.get(&action_id);
        match last_call {
            Some(last_call) if now > *last_call + t => self.run_action(action_id, action),
            None => self.run_action(action_id, action),
            _ => {}
        }
    }
}
