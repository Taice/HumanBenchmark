use std::io;

use ratatui::{DefaultTerminal, Frame, widgets::Widget};
use serde::{Deserialize, Serialize};

use super::{Filed, Game};

const FILE_NAME: &str = "AimTrainer";
const TARGET_AMOUNT: u32 = 30;

#[derive(Default)]
pub struct AimTrainer {
    exit: bool,

    savestate: ATSaveState,
}

impl AimTrainer {
    fn update_savestate(&mut self, score: u64) {
        let st = ATSaveState {
            avg_score: (self.savestate.avg_score * self.savestate.num_entries as u64)
                / (self.savestate.num_entries + 1) as u64,
            num_entries: self.savestate.num_entries + 1,
        };

        self.savestate = st;
    }
}

#[derive(Default, Debug, Clone, Copy, Deserialize, Serialize)]
pub struct ATSaveState {
    // in millis
    avg_score: u64,
    num_entries: u32,
}

impl Game for AimTrainer {
    fn run(terminal: &mut DefaultTerminal) -> io::Result<()> {
        let mut game = Self::load().unwrap_or_default();

        while !game.exit {
            terminal.draw(|frame| game.draw(frame))?;
            game.handle_input(terminal)?;
        }

        game.save();
        Ok(())
    }

    fn handle_input(&mut self, terminal: &mut ratatui::DefaultTerminal) -> std::io::Result<()> {
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }
}

impl Filed<'_> for AimTrainer {
    const NAME: &'static str = FILE_NAME;
    type SaveState = ATSaveState;
    fn get_savestate(&self) -> Self::SaveState {
        self.savestate
    }

    fn from_savestate(savestate: Self::SaveState) -> Self {
        Self {
            savestate,
            ..Default::default()
        }
    }
}

impl Widget for &AimTrainer {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
    }
}
