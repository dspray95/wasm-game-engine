pub struct DifficultyCurve {
    pub base: f32,
    pub cap: f32,
    /// Score at which the curve begins rising (flat at `base` before this).
    pub warmup_score: f32,
    /// Score at which `cap` is reached.
    pub full_scale_score: f32,
    /// Power applied to the normalised progress value. < 1.0 gives a fast
    /// initial rise that slows as it approaches `cap` (e.g. 0.5 = square root).
    pub exponent: f32,
}

impl DifficultyCurve {
    pub fn value(&self, score: i32) -> f32 {
        let span = self.full_scale_score - self.warmup_score;
        let progress = if span <= 0.0 {
            1.0
        } else {
            ((score as f32 - self.warmup_score) / span).clamp(0.0, 1.0)
        };
        let eased = progress.powf(self.exponent);
        self.base + (self.cap - self.base) * eased
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_at_base_during_warmup() {
        let curve = DifficultyCurve {
            base: 10.0,
            cap: 50.0,
            warmup_score: 50.0,
            full_scale_score: 1500.0,
            exponent: 0.6,
        };
        assert!((curve.value(0) - 10.0).abs() < 1e-4);
        assert!((curve.value(50) - 10.0).abs() < 1e-4);
    }

    #[test]
    fn reaches_cap_at_full_scale_score() {
        let curve = DifficultyCurve {
            base: 10.0,
            cap: 50.0,
            warmup_score: 50.0,
            full_scale_score: 1500.0,
            exponent: 0.6,
        };
        assert!((curve.value(1500) - 50.0).abs() < 1e-3);
        assert!((curve.value(100_000) - 50.0).abs() < 1e-3);
    }

    #[test]
    fn decreasing_curve_reaches_cap() {
        let curve = DifficultyCurve {
            base: 3.0,
            cap: 1.0,
            warmup_score: 50.0,
            full_scale_score: 1000.0,
            exponent: 0.5,
        };
        assert!((curve.value(0) - 3.0).abs() < 1e-4);
        assert!((curve.value(1000) - 1.0).abs() < 1e-3);
        assert!((curve.value(100_000) - 1.0).abs() < 1e-3);
    }
}
