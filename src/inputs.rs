use std::{collections::HashMap, time::Duration};

use crate::{
    wasm4::{BUTTON_1, BUTTON_2, BUTTON_DOWN, BUTTON_LEFT, BUTTON_RIGHT, BUTTON_UP, GAMEPAD1},
    wtime::{FromFrames, Winstant},
};

#[derive(Eq, Hash, PartialEq)]
pub enum InputEvent {
    Button1Press,
    Button1DoublePress,
    Button2Press,
    ButtonUpPress,
    ButtonDownPress,
    ButtonLeftPress,
    ButtonRightPress,
}

struct StoredHandler {
    handler: Box<dyn Fn()>,
    event: InputEvent,
}

pub struct Inputs {
    initialized: bool,
    handlers: Vec<StoredHandler>,
    last_fire: Option<HashMap<InputEvent, Winstant>>,
    double_press_activated: bool,
}

impl Inputs {
    pub const fn new() -> Self {
        Inputs {
            initialized: false,
            handlers: vec![],
            last_fire: None,
            double_press_activated: false,
        }
    }

    fn init(&mut self) {
        self.last_fire = Some(HashMap::new());
        self.initialized = true;
    }

    pub fn listen(&mut self, event: InputEvent, handler: impl Fn() + 'static) -> &mut Self {
        self.handlers.push(StoredHandler {
            handler: Box::new(handler),
            event,
        });
        self
    }

    pub fn unlisten(&mut self) {
        self.handlers.clear();
    }

    fn get_last_fire(&self, event: InputEvent) -> Option<&Winstant> {
        let map = self.last_fire.as_ref()?;
        map.get(&event)
    }

    pub fn tick(&mut self) {
        if !self.initialized {
            self.init()
        }
        let gamepad = unsafe { *GAMEPAD1 };
        for StoredHandler { ref handler, event } in &self.handlers {
            match event {
                InputEvent::Button1Press
                    if self.is_button1_pressed() && !self.double_press_activated =>
                {
                    handler()
                }
                InputEvent::Button1DoublePress if self.is_button1_pressed() => {
                    if let Some(last) = self.get_last_fire(InputEvent::Button1Press) {
                        let now = Winstant::now();
                        let max = *last + Duration::from_millis(200);
                        let min = *last + Duration::from_frames(2);
                        if now < max && now > min {
                            handler();
                            self.double_press_activated = true;
                        }
                    }
                }
                InputEvent::Button2Press if self.is_button2_pressed() => handler(),
                InputEvent::ButtonDownPress if gamepad & BUTTON_DOWN != 0 => handler(),
                InputEvent::ButtonUpPress if gamepad & BUTTON_UP != 0 => handler(),
                InputEvent::ButtonLeftPress if gamepad & BUTTON_LEFT != 0 => handler(),
                InputEvent::ButtonRightPress if gamepad & BUTTON_RIGHT != 0 => handler(),
                _ => {}
            }
        }
        let now = Winstant::now();
        match self.get_last_fire(InputEvent::Button1Press) {
            None => {
                if self.is_button1_pressed() {
                    let map = self.last_fire.as_mut().unwrap();
                    map.insert(InputEvent::Button1Press, now);
                }
            }
            Some(last) => {
                if self.double_press_activated && *last + Duration::from_millis(200) <= now {
                    self.double_press_activated = false;
                }
                if self.is_button1_pressed() {
                    let map = self.last_fire.as_mut().unwrap();
                    map.insert(InputEvent::Button1Press, now);
                }
            }
        }

        // if *last + Duration::from_frames(2) >= now {

        // }
    }

    pub fn is_button2_pressed(&self) -> bool {
        let gamepad = unsafe { *GAMEPAD1 };
        gamepad & BUTTON_2 != 0
    }

    pub fn is_button1_pressed(&self) -> bool {
        let gamepad = unsafe { *GAMEPAD1 };
        gamepad & BUTTON_1 != 0
    }
}
