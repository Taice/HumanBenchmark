use std::time::Instant;

#[derive(PartialEq, Eq)]
pub enum Mode {
    Menu,
    Watching(u32),
    Waiting(Instant),
    Clicking,
    Results,
}

impl Default for Mode {
    fn default() -> Self {
        Self::Watching(0)
    }
}
