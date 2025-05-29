use std::time::Instant;

#[derive(Default, Debug, Clone, Copy)]
pub enum Mode {
    #[default]
    Waiting,
    Watching(Instant),
    Playing,
    Results,
}
