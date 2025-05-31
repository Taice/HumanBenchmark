#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Waiting,
    Playing,
    Failed,
    Results,
}
