pub struct LinearTrend {
    pub last_level: f32,
    pub last_trend: f32,
    smoothing_param: f32,
}

// Holt’s Linear Trend Model
// Applicable to seasonal data with a linear trend but not dependent on the level of the series.
// 
//   Level: l_t = α⋅y_t + (1−α)⋅(l_{t−1} + b_{t−1})
//   Trend: b_t = β⋅(l_t − l_{t−1}) + (1−β)⋅b_{t−1}
//   Forecast: F_{t+m} = l_t + m⋅b_t
// 
impl LinearTrend {
    pub fn new() -> LinearTrend {
        LinearTrend {
            last_level: 0.,
            last_trend: 0.,
            smoothing_param: 0.5,
        }
    }

    fn _fit(&mut self, tau_t: f32) -> (f32, f32) {
        let alpha = self.smoothing_param;
        let beta = &alpha;

        let level = alpha * tau_t + (1.0 - alpha) * (self.last_level + alpha * self.last_trend);
        let trend = beta * (level - self.last_level) + (1.0 - beta) * beta * self.last_trend;

        self.last_level = level;
        self.last_trend = trend;

        (level, trend)
    }

    pub fn forecast(&mut self, tau_t: f32) -> f32 {
        let (level, trend) = self._fit(tau_t);
        level + trend
    }
}
