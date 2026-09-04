//! What time it is, on a platform that can be asked and in a page that cannot.
//!
//! The loop wants one thing from a clock: how long the last frame took, so that
//! an animation advances by the time that passed rather than by a count of
//! frames. Every desktop answers that with [`Instant`](std::time::Instant).
//!
//! A browser cannot. On `wasm32-unknown-unknown` there is no clock behind the
//! type at all — `Instant::now()` is compiled from the unsupported platform and
//! panics the moment it is called — so a page reads `performance.now()`
//! instead: milliseconds since the document began, monotonic in the same way,
//! and unmoved by the wall clock being set. Nothing above here knows or needs
//! to know which of the two it got.

use std::time::Duration;

/// A moment, as this platform is able to tell one.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Moment(Reading);

/// What the platform's clock actually hands back.
#[cfg(not(target_arch = "wasm32"))]
type Reading = std::time::Instant;

/// The same, in a page: milliseconds since the document was created.
#[cfg(target_arch = "wasm32")]
type Reading = f64;

impl Moment {
    /// Now.
    pub(crate) fn now() -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        {
            Self(std::time::Instant::now())
        }
        #[cfg(target_arch = "wasm32")]
        {
            Self(page_millis())
        }
    }

    /// How long has passed since `earlier`, and never less than nothing.
    ///
    /// Saturating for the reason `Instant::saturating_duration_since` is: a
    /// clock that looks to have gone backwards should cost a frame its
    /// animation, not end the program in front of the person using it.
    pub(crate) fn since(self, earlier: Self) -> Duration {
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.0.saturating_duration_since(earlier.0)
        }
        #[cfg(target_arch = "wasm32")]
        {
            Duration::from_secs_f64(((self.0 - earlier.0) / 1000.0).max(0.0))
        }
    }
}

/// The page's own clock, or a standing zero if the page has none.
///
/// Zero rather than an error: a document with no `performance` reports that no
/// time ever passes, which leaves animation standing still. That is the worst
/// this can do, and it is better than refusing to draw.
#[cfg(target_arch = "wasm32")]
fn page_millis() -> f64 {
    web_sys::window()
        .and_then(|window| window.performance())
        .map_or(0.0, |performance| performance.now())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn moment_now_succeeds() {
        let _ = Moment::now();
    }

    #[test]
    fn moment_since_measures_elapsed_time() {
        let m1 = Moment::now();
        let m2 = Moment::now();
        let elapsed = m2.since(m1);
        // Duration is always non-negative; just verify it can be measured.
        let _ = elapsed.as_millis();
    }

    #[test]
    fn moment_since_saturates_when_clock_goes_backward() {
        let m1 = Moment::now();
        let m2 = Moment::now();
        let elapsed = m1.since(m2);
        assert_eq!(
            elapsed.as_millis(),
            0,
            "saturating since prevents negative durations"
        );
    }
}
