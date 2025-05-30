#[derive(Default)]
pub enum Mode {
    #[default]
    Waiting,
    TooEarly,
    Clicking,
    TimeOut,
    Results,
}
