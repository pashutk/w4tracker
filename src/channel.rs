#[derive(Clone, Copy, PartialEq)]
pub enum Channel {
    Pulse1,
    Pulse2,
    Triangle,
    Noise,
}

impl Channel {
    pub fn iterator() -> impl Iterator<Item = Channel> {
        [
            Channel::Pulse1,
            Channel::Pulse2,
            Channel::Triangle,
            Channel::Noise,
        ]
        .iter()
        .copied()
    }

    pub fn next(&self) -> Self {
        match self {
            Channel::Pulse1 => Channel::Pulse2,
            Channel::Pulse2 => Channel::Triangle,
            Channel::Triangle => Channel::Noise,
            &noise @ Channel::Noise => noise,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            &pulse1 @ Channel::Pulse1 => pulse1,
            Channel::Pulse2 => Channel::Pulse1,
            Channel::Triangle => Channel::Pulse2,
            Channel::Noise => Channel::Triangle,
        }
    }
}
