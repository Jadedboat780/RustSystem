use super::{duration_to_ticks, ticks, ticks_to_duration};
use core::{
    ops::{Add, AddAssign, Sub},
    time::Duration,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Instant {
    ticks: u64,
}

impl Instant {
    #[inline]
    pub fn now() -> Self {
        Self { ticks: ticks() }
    }

    #[inline]
    pub fn duration_since(&self, earlier: Instant) -> Duration {
        ticks_to_duration(self.ticks.saturating_sub(earlier.ticks))
    }

    #[inline]
    pub fn elapsed(&self) -> Duration {
        Instant::now().duration_since(*self)
    }

    #[inline]
    pub fn checked_add(&self, duration: Duration) -> Option<Instant> {
        self.ticks.checked_add(duration_to_ticks(duration)).map(|ticks| Self { ticks })
    }
}

impl Add<Duration> for Instant {
    type Output = Instant;

    fn add(self, duration: Duration) -> Instant {
        Self {
            ticks: self.ticks.saturating_add(duration_to_ticks(duration)),
        }
    }
}

impl AddAssign<Duration> for Instant {
    fn add_assign(&mut self, duration: Duration) {
        *self = *self + duration;
    }
}

impl Sub for Instant {
    type Output = Duration;

    fn sub(self, earlier: Instant) -> Duration {
        self.duration_since(earlier)
    }
}
