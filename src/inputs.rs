use crate::wasm4::{
    BUTTON_1, BUTTON_2, BUTTON_DOWN, BUTTON_LEFT, BUTTON_RIGHT, BUTTON_UP, GAMEPAD1,
};

pub enum InputEvent {
    Button1Press,
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
                InputEvent::Button1Press if self.is_button1_pressed() => handler(),
                InputEvent::Button2Press if self.is_button2_pressed() => handler(),
                InputEvent::ButtonDownPress if gamepad & BUTTON_DOWN != 0 => handler(),
                InputEvent::ButtonUpPress if gamepad & BUTTON_UP != 0 => handler(),
                InputEvent::ButtonLeftPress if gamepad & BUTTON_LEFT != 0 => handler(),
                InputEvent::ButtonRightPress if gamepad & BUTTON_RIGHT != 0 => handler(),
                _ => {}
            }
        }
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
