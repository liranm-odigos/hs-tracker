//! Sleep until a deadline without giving up the last couple of milliseconds
//! to the scheduler. Skill cooldowns are long; the short spin is only there
//! so the key is not late by a full timer tick.

use std::thread;
use std::time::{Duration, Instant};

/// Returns true when `should_stop` fires before the deadline.
pub fn wait_until(deadline: Instant, should_stop: impl Fn() -> bool) -> bool {
    loop {
        if should_stop() {
            return true;
        }
        let now = Instant::now();
        if now >= deadline {
            return false;
        }
        let left = deadline.saturating_duration_since(now);
        if left > Duration::from_millis(2) {
            let coarse = (left - Duration::from_millis(1)).min(Duration::from_millis(5));
            thread::sleep(coarse);
        } else {
            std::hint::spin_loop();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finishes_a_short_wait() {
        let start = Instant::now();
        let stopped = wait_until(start + Duration::from_millis(40), || false);
        assert!(!stopped);
        let elapsed = start.elapsed();
        assert!(elapsed >= Duration::from_millis(35), "elapsed {elapsed:?}");
        assert!(elapsed < Duration::from_millis(200), "elapsed {elapsed:?}");
    }

    #[test]
    fn returns_immediately_when_already_stopped() {
        let start = Instant::now();
        let stopped = wait_until(start + Duration::from_secs(30), || true);
        assert!(stopped);
        assert!(start.elapsed() < Duration::from_millis(50));
    }
}
