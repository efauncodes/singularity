use std::time::Duration;

use bevy::prelude::*;

#[derive(Resource, Debug)]
pub struct SimulationClock {
    pub elapsed_units: u64,
    paused: bool,
    remainder: Duration,
}

impl Default for SimulationClock {
    fn default() -> Self {
        Self {
            elapsed_units: 0,
            paused: true,
            remainder: Duration::ZERO,
        }
    }
}

impl SimulationClock {
    pub fn is_paused(&self) -> bool {
        self.paused
    }

    pub(crate) fn set_paused(&mut self, paused: bool) {
        self.paused = paused;
    }

    pub(crate) fn advance(&mut self, delta: Duration, millis_per_unit: u32, stop_at: Option<u64>) {
        if self.paused {
            return;
        }

        let unit_nanos = u128::from(millis_per_unit) * 1_000_000;
        let budget = self.remainder.as_nanos() + delta.as_nanos();
        let units = budget / unit_nanos;
        let remaining = stop_at
            .unwrap_or(u64::MAX)
            .saturating_sub(self.elapsed_units);

        // Discard excess real time at an event boundary, so resume cannot catch up past it.
        if units >= u128::from(remaining) {
            self.elapsed_units += remaining;
            self.remainder = Duration::ZERO;
            return;
        }

        self.elapsed_units +=
            u64::try_from(units).expect("units are bounded by remaining u64 time");
        self.remainder = Duration::from_nanos(
            u64::try_from(budget % unit_nanos)
                .expect("a fractional u32 millisecond unit fits in u64"),
        );
    }
}

#[derive(Message, Clone, Copy, Debug)]
pub enum TimeControl {
    Start,
    Pause,
}

#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SimulationSet {
    Controls,
    Trade,
    TravelOrders,
    Advance,
    Arrivals,
    Notifications,
    Presentation,
}
