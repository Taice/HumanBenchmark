mod mode;

use super::{Filed, Game, savestate::SaveState};
use mode::Mode;

use rand::{Rng, rng};
use ratatui::style::Stylize;
use ratatui::text::Span;
use std::io;
use std::time::{Duration, Instant, SystemTime};

use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, KeyCode, MouseEventKind},
    layout::{Constraint, Direction, Layout},
    style::{Color, Style, Styled},
    symbols::border,
    widgets::{Block, Paragraph, Widget},
};

const FILE_NAME: &str = "ReactionTime";

#[derive(Default)]
pub struct ReactionTime {
    exit: bool,
    curr: Option<SystemTime>,
    times: Vec<Duration>,
    savestate: SaveState,
    mode: Mode,
}

impl ReactionTime {
    fn waiting_input(&mut self) -> io::Result<()> {
        let start = Instant::now();
        let dur = Duration::from_millis(rng().random_range(3000..6000));

        while start.elapsed() < dur {
            let remaining = dur.checked_sub(start.elapsed()).unwrap_or(Duration::ZERO);
            if event::poll(remaining)? {
                let event = event::read()?;
                match event {
                    event::Event::Key(key) => {
                        match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => self.exit = true,
                            _ => self.mode = Mode::TooEarly,
                        }
                        return Ok(());
                    }
                    event::Event::Mouse(mouse) => {
                        if let MouseEventKind::Down(_) = mouse.kind {
                            self.mode = Mode::TooEarly;
                            return Ok(());
                        }
                    }
                    _ => {} // Ignore other events and continue waiting
                }
            }
        }

        self.mode = Mode::Clicking;
        self.curr = Some(SystemTime::now());
        Ok(())
    }

    // lol what is this shit
    fn get_avg_time(&self) -> u64 {
        (self.savestate.avg_score.round() as u64 * self.savestate.num_entries as u64
            + self
                .times
                .iter()
                .fold(0u64, |acc, x| acc + x.as_millis() as u64))
            / (self.savestate.num_entries as u64 + self.times.len() as u64)
    }
}

impl Game for ReactionTime {
    fn run(terminal: &mut DefaultTerminal) -> io::Result<()> {
        let mut game = Self::load().unwrap_or_default();

        while !game.exit {
            terminal.draw(|frame| game.draw(frame))?;
            game.handle_input(terminal)?;
        }

        game.save();
        Ok(())
    }

    fn handle_input(&mut self, _: &mut DefaultTerminal) -> io::Result<()> {
        match self.mode {
            Mode::Waiting => {
                self.waiting_input()?;
            }
            Mode::Clicking => {
                if event::poll(Duration::from_secs(10))? {
                    let event = event::read()?;
                    match event {
                        event::Event::Key(key) => match key.code {
                            KeyCode::Esc | KeyCode::Char('q') => self.exit = true,
                            _ => {
                                self.times.push(self.curr.unwrap().elapsed().unwrap());
                                self.mode = Mode::Results;
                            }
                        },
                        event::Event::Mouse(mouse) => {
                            if let MouseEventKind::Down(_) = mouse.kind {
                                self.times.push(self.curr.unwrap().elapsed().unwrap());
                                self.mode = Mode::Results;
                            }
                        }
                        _ => (),
                    }
                } else {
                    self.mode = Mode::TimeOut;
                }
            }
            Mode::Results | Mode::TimeOut | Mode::TooEarly => {
                if event::poll(Duration::MAX)? {
                    let event = event::read()?;
                    match event {
                        event::Event::Key(key) => match key.code {
                            KeyCode::Esc | KeyCode::Char('q') => self.exit = true,
                            KeyCode::Enter | KeyCode::Char('r') => self.mode = Mode::Waiting,
                            _ => (),
                        },
                        event::Event::Mouse(mouse) => {
                            if let MouseEventKind::Down(_) = mouse.kind {
                                self.mode = Mode::Waiting;
                            }
                        }
                        _ => (),
                    }
                }
            }
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }
}

impl Filed<'_> for ReactionTime {
    const NAME: &'static str = FILE_NAME;
    type SaveState = SaveState;

    fn get_savestate(&self) -> Self::SaveState {
        SaveState {
            avg_score: self.get_avg_time() as f32,
            num_entries: self.savestate.num_entries + self.times.len() as u32,
        }
    }

    fn from_savestate(savestate: Self::SaveState) -> Self {
        Self {
            savestate,
            ..Default::default()
        }
    }
}

impl Widget for &ReactionTime {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let vert = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Min(0),    // Body
            ])
            .split(area);

        Paragraph::new(Span::from("Reaction Time Test").fg(Color::Red))
            .centered()
            .block(Block::bordered().border_set(border::DOUBLE))
            .render(vert[0], buf);

        Block::bordered()
            .border_set(border::DOUBLE)
            .title("╡ Playing field ╞")
            .render(vert[1], buf);

        let main = vert[1].inner(ratatui::layout::Margin {
            horizontal: 1,
            vertical: 1,
        });

        let center = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Min(0),
                Constraint::Length(1),
            ])
            .split(main);

        match self.mode {
            Mode::Waiting => {
                Block::new()
                    .style(Style::default().bg(Color::Red))
                    .render(main, buf);
                Paragraph::new("Waiting...")
                    .centered()
                    .set_style(Color::Black)
                    .render(center[1], buf);

                Paragraph::new("Esc/'q' to quit")
                    .centered()
                    .set_style(Color::Black)
                    .render(center[4], buf);
            }
            Mode::TooEarly => {
                Block::new()
                    .style(Style::default().bg(Color::DarkGray))
                    .render(main, buf);
                Paragraph::new("Too early you loser fuck you early clicker dumbass")
                    .centered()
                    .render(center[1], buf);

                Paragraph::new("'r' to restart and Esc/'q' to quit")
                    .centered()
                    .render(center[4], buf);
            }
            Mode::Clicking => {
                Block::new()
                    .style(Style::default().bg(Color::Green))
                    .render(main, buf);
                Paragraph::new("CLICK NOW FAST OR ELSE YOU'LL DIE NOW CLICK FAST")
                    .centered()
                    .set_style(Color::Black)
                    .render(center[1], buf);

                Paragraph::new("Esc/'q' to quit")
                    .centered()
                    .set_style(Color::Black)
                    .render(center[4], buf);
            }
            Mode::TimeOut => {
                Paragraph::new("You're so slow I literally timed out.")
                    .centered()
                    .render(center[1], buf);

                Paragraph::new("'r' to restart and Esc/'q' to quit")
                    .centered()
                    .render(center[4], buf);
            }
            Mode::Results => {
                Paragraph::new(format!(
                    "Your time was: {}ms",
                    self.times.last().unwrap().as_millis()
                ))
                .centered()
                .render(center[1], buf);
                Paragraph::new(format!(
                    "Avg. time across the board: {}ms",
                    self.get_avg_time()
                ))
                .centered()
                .render(center[3], buf);

                Paragraph::new("'r' to restart and Esc/'q' to quit")
                    .centered()
                    .render(center[4], buf);
            }
        }
    }
}
