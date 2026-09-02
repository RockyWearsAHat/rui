//! Standard animation curves and spring physics.
//!
//! R2 Motion Kit: Easing functions, spring dynamics, and transitions that
//! animate properties over time without reading a wall clock.
//!
//! Enter/exit transitions animate elements appearing or disappearing using
//! fade, slide, scale, or custom curves. Transitions are declared on elements
//! and automatically managed by Memory, which tracks animation progress.

/// Standard animation easing curves.
///
/// Each easing function maps t in [0, 1] to progress in [0, 1] with different
/// acceleration/deceleration profiles.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Easing {
    /// No acceleration; linear interpolation.
    Linear,
    /// Slow start, fast end (quadratic ease-in).
    EaseIn,
    /// Fast start, slow end (quadratic ease-out).
    EaseOut,
    /// Slow start and end, fast middle (cubic ease-in-out).
    EaseInOut,
    /// Custom cubic Bézier curve.
    CubicBezier {
        /// First control point x-coordinate.
        x1: f32,
        /// First control point y-coordinate.
        y1: f32,
        /// Second control point x-coordinate.
        x2: f32,
        /// Second control point y-coordinate.
        y2: f32,
    },
}

impl Easing {
    /// Interpolate a value from 0 to 1 using this easing curve.
    ///
    /// Takes `t` in [0, 1] and returns progress in [0, 1].
    pub fn interpolate(self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Easing::Linear => t,
            Easing::EaseIn => t * t,
            Easing::EaseOut => t * (2.0 - t),
            Easing::EaseInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    -1.0 + (4.0 - 2.0 * t) * t
                }
            }
            Easing::CubicBezier { x1, y1, x2, y2 } => cubic_bezier(t, x1, y1, x2, y2),
        }
    }
}

/// Cubic Bézier interpolation using standard parametric form.
fn cubic_bezier(t: f32, x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    // Bezier curve: B(s) = (1-s)³P0 + 3(1-s)²sP1 + 3(1-s)s²P2 + s³P3
    // where P0=(0,0), P1=(x1,y1), P2=(x2,y2), P3=(1,1)
    // Find s such that x-coordinate equals t, then return y-coordinate

    let mut s = t; // Start with s = t as initial guess
    for _ in 0..5 {
        // x(s) = 3(1-s)²s*x1 + 3(1-s)s²*x2 + s³
        let mt = 1.0 - s;
        let x = 3.0 * mt * mt * s * x1 + 3.0 * mt * s * s * x2 + s * s * s;
        let dx = 3.0 * (mt * mt * (x1 - 0.0) + 2.0 * mt * s * (x2 - x1) + s * s * (1.0 - x2));

        if (x - t).abs() < 0.0001 || dx.abs() < 0.0001 {
            break;
        }
        s -= (x - t) / dx;
    }

    s = s.clamp(0.0, 1.0);
    let mt = 1.0 - s;
    // y(s) = 3(1-s)²s*y1 + 3(1-s)s²*y2 + s³
    3.0 * mt * mt * s * y1 + 3.0 * mt * s * s * y2 + s * s * s
}

/// Spring physics for natural-feeling animations.
///
/// Models a mass on a spring with configurable stiffness and damping.
/// Springs settle smoothly toward their target without the artificial
/// precision of easing functions.
#[derive(Debug, Clone)]
pub struct Spring {
    position: f32,
    velocity: f32,
    stiffness: f32,
    damping: f32,
    mass: f32,
}

impl Spring {
    /// Create a spring with stiffness, damping, and mass.
    ///
    /// - stiffness: How hard the spring pulls (typical: 100–400)
    /// - damping: How much resistance (typical: 10–30)
    /// - mass: Inertia of the moving object (typical: 1.0)
    pub fn new(stiffness: f32, damping: f32, mass: f32) -> Self {
        Self {
            position: 0.0,
            velocity: 0.0,
            stiffness,
            damping,
            mass,
        }
    }

    /// Spring preset for gentle, leisurely motion (bouncy).
    pub fn gentle() -> Self {
        Self::new(80.0, 20.0, 1.0)
    }

    /// Spring preset for normal, responsive motion.
    pub fn normal() -> Self {
        Self::new(150.0, 15.0, 1.0)
    }

    /// Spring preset for snappy, energetic motion (tight).
    pub fn snappy() -> Self {
        Self::new(300.0, 10.0, 1.0)
    }

    /// Advance the spring by one time step toward target = 1.0.
    ///
    /// Returns (position, velocity).
    pub fn tick(&mut self, dt: f32) -> (f32, f32) {
        let target = 1.0;
        let distance = target - self.position;
        let force = self.stiffness * distance - self.damping * self.velocity;
        let acceleration = force / self.mass;
        self.velocity += acceleration * dt;
        self.position += self.velocity * dt;

        // Settle when very close to target and moving slowly
        if distance.abs() < 0.001 && self.velocity.abs() < 0.001 {
            self.position = target;
            self.velocity = 0.0;
        }

        (self.position, self.velocity)
    }

    /// The damping coefficient.
    pub fn damping(&self) -> f32 {
        self.damping
    }
}

/// Enter/exit transition animation type.
///
/// Specifies how an element animates as it appears (enter) or disappears (exit).
/// Transitions combine a type (fade, slide, scale) with duration and easing.
#[derive(Debug, Clone)]
pub enum Transition {
    /// Fade in/out over duration with easing curve.
    Fade {
        /// Duration in seconds.
        duration: f32,
        /// Easing curve for opacity progression.
        easing: Easing,
    },
    /// Slide from/to direction over duration.
    Slide {
        /// Duration in seconds.
        duration: f32,
        /// Easing curve for position progression.
        easing: Easing,
        /// Direction: "up", "down", "left", "right".
        direction: SlideDirection,
    },
    /// Scale from/to a size over duration.
    Scale {
        /// Duration in seconds.
        duration: f32,
        /// Easing curve for scale progression.
        easing: Easing,
        /// Starting scale (0.0–1.0).
        from_scale: f32,
    },
}

/// Direction for slide transitions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SlideDirection {
    /// Slide from/to the left.
    Left,
    /// Slide from/to the right.
    Right,
    /// Slide from/to above.
    Up,
    /// Slide from/to below.
    Down,
}

impl Transition {
    /// Fade in over duration with ease-out timing (feels natural for appearance).
    pub fn fade_in(duration: f32) -> Self {
        Self::Fade {
            duration,
            easing: Easing::EaseOut,
        }
    }

    /// Fade out over duration with ease-in timing (feels natural for disappearance).
    pub fn fade_out(duration: f32) -> Self {
        Self::Fade {
            duration,
            easing: Easing::EaseIn,
        }
    }

    /// Slide in from a direction over duration.
    pub fn slide_in(direction: SlideDirection, duration: f32) -> Self {
        Self::Slide {
            duration,
            easing: Easing::EaseOut,
            direction,
        }
    }

    /// Slide out to a direction over duration.
    pub fn slide_out(direction: SlideDirection, duration: f32) -> Self {
        Self::Slide {
            duration,
            easing: Easing::EaseIn,
            direction,
        }
    }

    /// Scale in from a smaller size over duration.
    pub fn scale_in(from_scale: f32, duration: f32) -> Self {
        Self::Scale {
            duration,
            easing: Easing::EaseOut,
            from_scale,
        }
    }

    /// Scale out to a smaller size over duration.
    pub fn scale_out(to_scale: f32, duration: f32) -> Self {
        Self::Scale {
            duration,
            easing: Easing::EaseIn,
            from_scale: to_scale,
        }
    }

    /// Duration of this transition in seconds.
    pub fn duration(&self) -> f32 {
        match self {
            Self::Fade { duration, .. }
            | Self::Slide { duration, .. }
            | Self::Scale { duration, .. } => *duration,
        }
    }

    /// Easing curve for this transition.
    pub fn easing(&self) -> Easing {
        match self {
            Self::Fade { easing, .. } | Self::Slide { easing, .. } | Self::Scale { easing, .. } => {
                *easing
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linear_easing_is_identity() {
        assert_eq!(Easing::Linear.interpolate(0.0), 0.0);
        assert_eq!(Easing::Linear.interpolate(0.5), 0.5);
        assert_eq!(Easing::Linear.interpolate(1.0), 1.0);
    }

    #[test]
    fn ease_in_accelerates() {
        let mid = Easing::EaseIn.interpolate(0.5);
        assert!(mid < 0.5);
    }

    #[test]
    fn ease_out_decelerates() {
        let mid = Easing::EaseOut.interpolate(0.5);
        assert!(mid > 0.5);
    }

    #[test]
    fn spring_settles_toward_target() {
        let mut spring = Spring::new(100.0, 10.0, 1.0);
        for _ in 0..200 {
            spring.tick(0.016);
        }
        assert!((spring.position - 1.0).abs() < 0.01);
    }

    #[test]
    fn spring_presets_have_correct_damping_order() {
        let gentle = Spring::gentle();
        let snappy = Spring::snappy();
        assert!(snappy.damping() < gentle.damping());
    }

    #[test]
    fn fade_in_has_correct_duration_and_easing() {
        let trans = Transition::fade_in(0.3);
        assert_eq!(trans.duration(), 0.3);
        assert_eq!(trans.easing(), Easing::EaseOut);
    }

    #[test]
    fn fade_out_has_correct_duration_and_easing() {
        let trans = Transition::fade_out(0.3);
        assert_eq!(trans.duration(), 0.3);
        assert_eq!(trans.easing(), Easing::EaseIn);
    }

    #[test]
    fn slide_in_has_correct_duration_and_direction() {
        let trans = Transition::slide_in(SlideDirection::Left, 0.4);
        assert_eq!(trans.duration(), 0.4);
        assert_eq!(trans.easing(), Easing::EaseOut);
        if let Transition::Slide { direction, .. } = trans {
            assert_eq!(direction, SlideDirection::Left);
        } else {
            panic!("Expected Slide transition");
        }
    }

    #[test]
    fn scale_in_has_correct_duration_and_scale() {
        let trans = Transition::scale_in(0.8, 0.25);
        assert_eq!(trans.duration(), 0.25);
        assert_eq!(trans.easing(), Easing::EaseOut);
        if let Transition::Scale { from_scale, .. } = trans {
            assert_eq!(from_scale, 0.8);
        } else {
            panic!("Expected Scale transition");
        }
    }

    #[test]
    fn scale_out_has_correct_duration_and_scale() {
        let trans = Transition::scale_out(0.5, 0.25);
        assert_eq!(trans.duration(), 0.25);
        assert_eq!(trans.easing(), Easing::EaseIn);
        if let Transition::Scale { from_scale, .. } = trans {
            assert_eq!(from_scale, 0.5);
        } else {
            panic!("Expected Scale transition");
        }
    }
}
