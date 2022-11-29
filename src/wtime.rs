use std::{
    ops::{Add, Sub},
    time::Duration,
};

#[derive(PartialEq, PartialOrd, Clone, Copy)]
pub struct Winstant(u32);

static mut NOW: u32 = 0;

impl Winstant {
    pub fn now() -> Self {
        let now: u32;
        unsafe {
            now = NOW;
        }
        Winstant(now)
    }

    pub fn tick() {
        unsafe { NOW += 1 }
    }

    pub fn duration_since(&self, earlier: Winstant) -> Duration {
        Duration::from_frames(self.0 - earlier.0)
    }
}

const EXPECTED_FRAME_DURATION_MS: u32 = 1000 / 60;

impl Add<Duration> for Winstant {
    type Output = Self;

    fn add(self, rhs: Duration) -> Self::Output {
        let add_ms: u32 = rhs.as_millis() as u32 / EXPECTED_FRAME_DURATION_MS;
        Winstant(self.0 + add_ms)
    }
}

pub trait FromFrames {
    fn from_frames(x: u32) -> Self;
}

impl FromFrames for Duration {
    fn from_frames(x: u32) -> Self {
        Duration::from_millis((EXPECTED_FRAME_DURATION_MS * x).into())
    }
}
