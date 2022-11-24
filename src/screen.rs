#[derive(PartialEq, Clone, Copy)]
pub enum Screen {
    Song,
    Chain,
    Pattern,
    Instrument,
}

pub enum Screens {
    Single(Screen),
    Transition(Screen, Screen),
}
