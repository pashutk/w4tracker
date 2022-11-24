use std::{
    collections::HashMap,
    sync::atomic::{AtomicUsize, Ordering},
    time::Duration,
};

use crate::wtime::Winstant;

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

fn get_unique_usize() -> usize {
    static VALUE: AtomicUsize = AtomicUsize::new(0);
    VALUE.fetch_add(1, Ordering::Relaxed)
}

struct StoredInterval {
    id: usize,
    thunk: Box<dyn Fn()>,
    interval: usize,
}

pub struct Timers {
    last_calls: Option<HashMap<ActionId, Winstant>>,
    intervals: Vec<StoredInterval>,
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

    pub fn set_interval<F>(&mut self, action: F, interval: usize) -> usize
    where
        F: Fn() + 'static,
    {
        let id = get_unique_usize();
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
}
