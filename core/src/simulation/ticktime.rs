pub struct TickTime(pub std::time::Instant);

impl Default for TickTime {
    fn default() -> TickTime {
        TickTime(std::time::Instant::now())
    }
}