#[derive(Debug, Default, Clone, Copy)]
pub enum Mode {
    #[default]
    Waiting,
    Playing,
    Results,
}
