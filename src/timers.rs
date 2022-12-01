use std::{collections::HashMap, time::Duration, vec};

use crate::{unique_usize::get_unique_usize, wasm4::trace, wtime::Winstant};

pub static mut TIMERS: Timers = Timers {
    last_calls: None,
    intervals: vec![],
};

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

struct StoredInterval<'a> {
    id: usize,
    thunk: Box<dyn Fn() + 'a>,
    interval: usize,
}

pub struct Timers<'a> {
    last_calls: Option<HashMap<ActionId, Winstant>>,
    intervals: Vec<StoredInterval<'a>>,
}

impl<'a> Timers<'a>  {
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

    pub fn set_interval<F>(&mut self, id: usize, action: F, interval: usize) -> usize
    where
        F: Fn() + 'a
    {
        self.intervals.push(StoredInterval {
            id,
            thunk: Box::new(action),
            interval,
        });
        id
    }

    pub fn cancel_interval(&mut self, interval_id: usize) {
        let mut i = 0;
        while i < self.intervals.len() {
            let x = &mut self.intervals[i];
            if x.id == interval_id {
                self.intervals.remove(i);
            } else {
                i += 1;
            }
        }
    }

    pub fn tick(&self) {
        for interval in &self.intervals {
            let x = &interval.thunk;
            x();
        }
    }
}
