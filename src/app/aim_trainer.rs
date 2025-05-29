mod mode;

use mode::Mode;
use std::{
    io,
    time::{Duration, Instant},
};

use rand::{Rng, rng};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    crossterm::event::{self, KeyCode, MouseEvent, MouseEventKind},
    layout::{Constraint, Direction, Layout, Position, Rect},
    style::{Color, Style, Styled, Stylize},
    symbols::border,
    text::Span,
    widgets::{Block, Paragraph, Widget},
};

use super::{Filed, Game, savestate::SaveState};

const FILE_NAME: &str = "AimTrainer";
const TARGET_AMOUNT: u64 = 30;
const TARGET_SIZE: u16 = 3;
const PF_WIDTH: u16 = 100;
const PF_HEIGHT: u16 = 20;

#[derive(Default)]
pub struct AimTrainer {
    exit: bool,

    mode: Mode,
    target: Position,
    instant: Option<Instant>,
    times: SaveState,
    savestate: SaveState,
}

impl AimTrainer {
    fn mouse_input(&mut self, terminal: &mut DefaultTerminal, mouse: MouseEvent) {
        match self.mode {
            Mode::Waiting => {
                if let MouseEventKind::Down(_) = mouse.kind {
                } else {
                    return;
                }

                let area = terminal.get_frame().area();
                let vert = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Length(3), Constraint::Min(0)])
                    .split(area);

                let main_rec = vert[1].inner(ratatui::layout::Margin {
                    horizontal: 1,
                    vertical: 1,
                });

                let main = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Min(0),
                        Constraint::Length(1),
                        Constraint::Length(TARGET_SIZE),
                        Constraint::Length(1),
                        Constraint::Min(0),
                    ])
                    .split(main_rec);

                let rect = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Min(0),
                        Constraint::Length(TARGET_SIZE * 2),
                        Constraint::Min(0),
                    ])
                    .split(main[2]);

                let mouse_rect = Rect::new(mouse.column, mouse.row, 1, 1);

                if mouse_rect.intersects(rect[1]) {
                    self.mode = Mode::Playing;
                    self.instant = Some(Instant::now());
                    self.new_target();
                }
            }
            Mode::Playing => {
                if let MouseEventKind::Down(_) = mouse.kind {
                } else {
                    return;
                }

                let vert = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Length(3), Constraint::Min(0)])
                    .split(terminal.get_frame().area());
                let main = vert[1].inner(ratatui::layout::Margin {
                    horizontal: 1,
                    vertical: 1,
                });

                let pf_vert = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Min(0),
                        Constraint::Length(PF_HEIGHT),
                        Constraint::Min(0),
                    ])
                    .split(main)[1];

                let pf = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Min(0),
                        Constraint::Length(PF_WIDTH),
                        Constraint::Min(0),
                    ])
                    .split(pf_vert)[1];

                let target_vert = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(self.target.y),
                        Constraint::Length(TARGET_SIZE),
                    ])
                    .split(pf)[1];

                let target = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Length(self.target.x),
                        Constraint::Length(TARGET_SIZE * 2),
                    ])
                    .split(target_vert)[1];

                let mouse_rect = Rect::new(mouse.column, mouse.row, 1, 1);

                if mouse_rect.intersects(target) {
                    self.update_times();
                    self.new_target();
                }
            }
            Mode::Results => {
                if let MouseEventKind::Down(event::MouseButton::Left) = mouse.kind {
                    self.reset();
                }
            }
        }
    }

    fn new_target(&mut self) {
        self.instant = Some(Instant::now());
        let mut rng = rng();
        self.target = Position {
            x: rng.random_range(0..(PF_WIDTH - TARGET_SIZE * 2)),
            y: rng.random_range(0..(PF_HEIGHT - TARGET_SIZE)),
        };
    }

    fn update_times(&mut self) {
        if let Some(val) = self.instant {
            self.times.update(val.elapsed().as_millis() as f32);
            if self.times.num_entries as u64 >= TARGET_AMOUNT {
                self.mode = Mode::Results;
                self.savestate.update(self.times.avg_score);
            }
        }
    }

    fn reset(&mut self) {
        let df = Self {
            savestate: self.savestate,
            ..Default::default()
        };
        *self = df;
    }
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
        if event::poll(Duration::from_secs(10))? {
            let event = event::read()?;
            match event {
                event::Event::Key(key) => match key.code {
                    KeyCode::Esc | KeyCode::Char('q') => self.exit = true,
                    KeyCode::Char('r') => self.reset(),
                    _ => {}
                },
                event::Event::Mouse(mouse) => {
                    self.mouse_input(terminal, mouse);
                }
                _ => (),
            }
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }
}

impl Filed<'_> for AimTrainer {
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

impl Widget for &AimTrainer {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        if area.width < PF_WIDTH + 2 || area.height < PF_HEIGHT + 2 + 2 + 1 {
            let area = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(0),
                    Constraint::Length(1),
                    Constraint::Min(0),
                ])
                .split(area);
            Paragraph::new("Please make the window bigger.")
                .centered()
                .render(area[1], buf);
            return;
        }

        let vert = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(area);

        Paragraph::new(Span::from("Aim Trainer Test").fg(Color::Red))
            .centered()
            .block(Block::bordered().border_set(border::DOUBLE))
            .render(vert[0], buf);

        let main = vert[1].inner(ratatui::layout::Margin {
            horizontal: 1,
            vertical: 1,
        });

        match self.mode {
            Mode::Waiting => {
                Block::bordered()
                    .border_set(border::DOUBLE)
                    .title("╡ Playing field ╞")
                    .render(vert[1], buf);

                let main_vert = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Min(0),
                        Constraint::Length(1),
                        Constraint::Length(TARGET_SIZE),
                        Constraint::Length(1),
                        Constraint::Min(0),
                    ])
                    .split(main);

                let rect = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Min(0),
                        Constraint::Length(TARGET_SIZE * 2),
                        Constraint::Min(0),
                    ])
                    .split(main_vert[2]);

                render_target(rect[1], buf);

                Paragraph::new("Hit 30 targets in as short a time as possible")
                    .set_style(Color::DarkGray)
                    .italic()
                    .centered()
                    .render(main_vert[1], buf);
            }
            Mode::Playing => {
                Block::bordered()
                    .border_set(border::DOUBLE)
                    .title(format!(
                        "╡ Playing field {}, {}, {} ╞",
                        self.times.avg_score,
                        self.times.num_entries,
                        self.instant.unwrap().elapsed().as_millis()
                    ))
                    .render(vert[1], buf);

                let pf_vert = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Min(0),
                        Constraint::Length(PF_HEIGHT),
                        Constraint::Min(0),
                    ])
                    .split(main)[1];

                let pf = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Min(0),
                        Constraint::Length(PF_WIDTH),
                        Constraint::Min(0),
                    ])
                    .split(pf_vert)[1];

                let target_vert = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(self.target.y),
                        Constraint::Length(TARGET_SIZE),
                    ])
                    .split(pf)[1];

                let target = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Length(self.target.x),
                        Constraint::Length(TARGET_SIZE * 2),
                    ])
                    .split(target_vert)[1];

                render_target(target, buf);
            }
            Mode::Results => {
                Block::bordered()
                    .border_set(border::DOUBLE)
                    .title("╡ Results ╞")
                    .render(vert[1], buf);

                let constraint = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Min(0),
                        Constraint::Length(1),
                        Constraint::Length(1),
                        Constraint::Min(0),
                    ])
                    .split(main);

                Paragraph::new(format!("Your score is: {}", self.times.avg_score))
                    .centered()
                    .render(constraint[1], buf);

                Paragraph::new(format!(
                    "Your avg score overall is: {}",
                    self.savestate.avg_score
                ))
                .centered()
                .render(constraint[2], buf);
            }
        }
    }
}

fn render_target(rect: Rect, buf: &mut Buffer) {
    // target
    Block::bordered()
        .border_set(border::QUADRANT_OUTSIDE)
        .style(Style::default().bg(Color::Rgb(50, 50, 50)))
        .render(rect, buf);
}
