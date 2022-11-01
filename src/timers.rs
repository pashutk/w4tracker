use std::{collections::HashMap, time::Duration};

use crate::wtime::Winstant;

pub static mut TIMERS: Timers = Timers { last_calls: None };

pub struct Timers {
    last_calls: Option<HashMap<String, Winstant>>,
}

impl Timers {
    pub fn init(&mut self) {
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

    pub fn run_action_debounced<F>(&mut self, key: String, t: Duration, action: F)
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
