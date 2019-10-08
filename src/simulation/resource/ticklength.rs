#[derive(Default)]
pub struct TickLength(pub std::time::Duration);

impl TickLength {
    pub fn scale_to(&self, value: f64) -> f64 {
        return value / 1000.0 * self.0.as_millis() as f64;
    }
}