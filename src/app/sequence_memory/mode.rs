use std::time::Instant;

#[derive(PartialEq, Eq)]
pub enum Mode {
    Waiting,
    Watching(u32),
    Pause(Instant),
    Clicking,
    Results,
}

impl Default for Mode {
    fn default() -> Self {
        Self::Watching(0)
    }
}
