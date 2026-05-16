pub struct DifficultyCurve {
    pub base: f32,
    pub cap: f32,
    pub sensitivity: f32,
}

impl DifficultyCurve {
    pub fn value(&self, score: i32) -> f32 {
        let raw = self.base + (score as f32) * self.sensitivity;
        if self.cap >= self.base {
            raw.min(self.cap)
        } else {
            raw.max(self.cap)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ramps_up_and_caps() {
        let curve = DifficultyCurve { base: 10.0, cap: 50.0, sensitivity: 0.00267 };
        assert!((curve.value(0) - 10.0).abs() < 1e-4);
        assert!((curve.value(15_000) - 50.0).abs() < 0.05);
        assert!((curve.value(100_000) - 50.0).abs() < 1e-4);
    }

    #[test]
    fn ramps_down_and_floors() {
        let curve = DifficultyCurve { base: 3.0, cap: 1.0, sensitivity: -0.000667 };
        assert!((curve.value(0) - 3.0).abs() < 1e-4);
        assert!((curve.value(3_000) - 1.0).abs() < 0.01);
        assert!((curve.value(100_000) - 1.0).abs() < 1e-4);
    }
}
