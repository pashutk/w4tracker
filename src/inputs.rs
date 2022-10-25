use crate::wasm4::{BUTTON_2, BUTTON_DOWN, BUTTON_UP, GAMEPAD1};

pub enum InputEvent {
    Button2Press,
    ButtonUpPress,
    ButtonDownPress,
}

struct StoredHandler {
    handler: Box<dyn Fn()>,
    event: InputEvent,
}

pub struct Inputs {
    handlers: Vec<StoredHandler>,
}

impl Inputs {
    pub const fn new() -> Self {
        Inputs { handlers: vec![] }
    }

    pub fn listen<F>(&mut self, event: InputEvent, handler: F) -> &mut Self
    where
        F: Fn() + 'static,
    {
        self.handlers.push(StoredHandler {
            handler: Box::new(handler),
            event,
        });
        self
    }

    pub fn tick(&self) {
        let gamepad = unsafe { *GAMEPAD1 };
        for StoredHandler { ref handler, event } in &self.handlers {
            match event {
                InputEvent::Button2Press if gamepad & BUTTON_2 != 0 => handler(),
                InputEvent::ButtonDownPress if gamepad & BUTTON_DOWN != 0 => handler(),
                InputEvent::ButtonUpPress if gamepad & BUTTON_UP != 0 => handler(),
                _ => {}
            }
        }
    }
}
