mod mode;

use super::render_graph;
use super::{Filed, Game, savestate::SaveState};
use mode::Mode;

use rand::{Rng, rng};
use ratatui::style::Stylize;
use ratatui::symbols::Marker;
use ratatui::text::Span;
use ratatui::widgets::{Dataset, GraphType};
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
    time: f32,
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
                                self.savestate.update(
                                    self.curr.unwrap().elapsed().unwrap().as_millis() as f32,
                                );
                                self.time =
                                    self.curr.unwrap().elapsed().unwrap().as_millis() as f32;
                                self.mode = Mode::Results;
                            }
                        },
                        event::Event::Mouse(mouse) => {
                            if let MouseEventKind::Down(_) = mouse.kind {
                                self.savestate.update(
                                    self.curr.unwrap().elapsed().unwrap().as_millis() as f32,
                                );
                                self.time =
                                    self.curr.unwrap().elapsed().unwrap().as_millis() as f32;
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
        self.savestate
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

        let block = Block::bordered().border_set(border::DOUBLE);

        match self.mode {
            Mode::Waiting => {
                block.title("╡ Game ╞").render(vert[1], buf);
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
                block.title("╡ Too early ╞").render(vert[1], buf);
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
                block.title("╡ Clicking ╞").render(vert[1], buf);
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
                block.title("╡ Timed out ╞").render(vert[1], buf);
                Paragraph::new("You're so slow I literally timed out.")
                    .centered()
                    .render(center[1], buf);

                Paragraph::new("'r' to restart and Esc/'q' to quit")
                    .centered()
                    .render(center[4], buf);
            }
            Mode::Results => {
                block.title("╡ Results ╞").render(vert[1], buf);

                let dataset = Dataset::default()
                    .marker(Marker::Braille)
                    .graph_type(GraphType::Line)
                    .cyan()
                    .data(&[
                        (0.0, (0.0 / 270.0)),
                        (25.0, (0.0 / 270.0)),
                        (50.0, (0.0 / 270.0)),
                        (75.0, (0.0 / 270.0)),
                        (100.0, (0.0 / 270.0)),
                        (115.0, (5.0 / 270.0)),
                        (125.0, (14.0 / 270.0)),
                        (150.0, (78.0 / 270.0)),
                        (175.0, (205.0 / 270.0)),
                        (200.0, (250.0 / 270.0)),
                        (225.0, (230.0 / 270.0)),
                        (250.0, (160.0 / 270.0)),
                        (275.0, (90.0 / 270.0)),
                        (300.0, (50.0 / 270.0)),
                        (325.0, (30.0 / 270.0)),
                        (350.0, (17.0 / 270.0)),
                        (375.0, (10.0 / 270.0)),
                        (400.0, (8.0 / 270.0)),
                        (425.0, (6.0 / 270.0)),
                        (450.0, (5.0 / 270.0)),
                        (475.0, (3.0 / 270.0)),
                        (500.0, (3.0 / 270.0)),
                    ]);

                render_graph(
                    self.savestate.avg_score as f64,
                    self.time as f64,
                    dataset,
                    [0.0, 500.0],
                    main,
                    buf,
                );
            }
        }
    }
}
