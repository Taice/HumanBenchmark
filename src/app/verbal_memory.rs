mod mode;
mod words;

use std::{collections::HashSet, time::Duration};
use words::WORDS;

use mode::Mode;

use rand::{Rng, rng};
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, KeyCode, KeyEvent, MouseEvent, MouseEventKind},
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Style, Stylize},
    symbols::border,
    text::{Line, Span},
    widgets::{Block, Paragraph, Widget},
};

use super::{Filed, Game, savestate::SaveState};

const FILE_NAME: &str = "VerbalMemory";
const CHANCE: u32 = 5;
const LIVES: u32 = 3;

#[derive(Debug, Clone)]
pub struct VerbalMemory {
    exit: bool,
    score: u32,
    lives: u32,

    mode: Mode,
    current: usize,
    set: HashSet<usize>,
    savestate: SaveState,
}

impl Default for VerbalMemory {
    fn default() -> Self {
        Self {
            exit: false,
            score: 0,
            lives: LIVES,
            mode: Mode::default(),
            current: 0,
            set: HashSet::new(),
            savestate: SaveState::default(),
        }
    }
}

impl VerbalMemory {
    fn reset(&mut self) {
        let new = Self {
            savestate: self.savestate,
            ..Default::default()
        };
        *self = new;
    }

    fn key_event(&mut self, key: KeyEvent) {
        match self.mode {
            Mode::Waiting => match key.code {
                KeyCode::Esc | KeyCode::Char('q') => self.exit = true,
                KeyCode::Char('r') => self.reset(),
                KeyCode::Enter | KeyCode::Char(' ') => {
                    self.mode = Mode::Playing;
                    self.new_word()
                }
                _ => (),
            },
            Mode::Playing => match key.code {
                KeyCode::Esc | KeyCode::Char('q') => self.exit = true,
                KeyCode::Char('r') => self.reset(),
                KeyCode::Char('s') => self.submit_seen(),
                KeyCode::Char('n') => self.submit_new(),
                _ => (),
            },
            Mode::Results => match key.code {
                KeyCode::Esc | KeyCode::Char('q') => self.exit = true,
                KeyCode::Enter | KeyCode::Char('r') => self.reset(),
                _ => (),
            },
        }
    }

    fn mouse_event(&mut self, mouse: MouseEvent, terminal: &mut DefaultTerminal) {
        match self.mode {
            Mode::Waiting => {
                if let MouseEventKind::Down(_) = mouse.kind {
                    self.mode = Mode::Playing;
                    self.new_word();
                }
            }
            Mode::Playing => {
                if let MouseEventKind::Down(_) = mouse.kind {
                } else {
                    return;
                }

                let main = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Length(3), Constraint::Min(0)])
                    .split(terminal.get_frame().area())[1]
                    .inner(Margin {
                        horizontal: 1,
                        vertical: 1,
                    });

                let constraints = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Min(0),
                        Constraint::Length(1),
                        Constraint::Min(0),
                    ])
                    .split(main);

                let buttons_vert = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Percentage(20),
                        Constraint::Length(3),
                        Constraint::Min(0),
                    ])
                    .split(constraints[2])[1];

                let buttons = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Min(0),
                        Constraint::Length(6),
                        Constraint::Percentage(20),
                        Constraint::Length(6),
                        Constraint::Min(0),
                    ])
                    .split(buttons_vert);

                let mouse_rect = Rect::new(mouse.column, mouse.row, 1, 1);
                if mouse_rect.intersects(buttons[1]) {
                    self.submit_seen();
                } else if mouse_rect.intersects(buttons[3]) {
                    self.submit_new();
                }
            }
            Mode::Results => {
                if let MouseEventKind::Down(_) = mouse.kind {
                    self.reset();
                }
            }
        }
    }

    fn new_word(&mut self) {
        let mut rng = rng();
        if rng.random_range(0..CHANCE) == 1 && !self.set.is_empty() {
            self.current = *self
                .set
                .iter()
                .nth(rng.random_range(0..self.set.len()))
                .unwrap_or(&rng.random_range(0..WORDS.len()));
        } else {
            self.current = rng.random_range(0..WORDS.len());
        }
    }

    fn decrease_lives(&mut self) {
        if self.lives > 0 {
            self.lives -= 1;
        } else {
            self.mode = Mode::Results;
            self.savestate.update(self.score as f32);
        }
    }

    fn submit_new(&mut self) {
        if !self.set.insert(self.current) {
            self.decrease_lives();
        } else {
            self.score += 1;
        }

        self.new_word();
    }
    fn submit_seen(&mut self) {
        if self.set.insert(self.current) {
            self.decrease_lives();
        } else {
            self.score += 1;
        }

        self.new_word();
    }
}

impl Game for VerbalMemory {
    fn run(terminal: &mut ratatui::DefaultTerminal) -> std::io::Result<()> {
        let mut game = Self::load().unwrap_or_default();

        while !game.exit {
            terminal.draw(|frame| game.draw(frame))?;
            game.handle_input(terminal)?;
        }

        game.save();
        Ok(())
    }

    fn handle_input(&mut self, terminal: &mut ratatui::DefaultTerminal) -> std::io::Result<()> {
        if event::poll(Duration::MAX)? {
            match event::read()? {
                event::Event::Key(key) => self.key_event(key),
                event::Event::Mouse(mouse) => self.mouse_event(mouse, terminal),
                _ => (),
            }
        }

        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }
}

impl Filed<'_> for VerbalMemory {
    type SaveState = SaveState;
    const NAME: &'static str = FILE_NAME;

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

impl Widget for &VerbalMemory {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let vert = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(area);

        Paragraph::new(Span::from("Verbal Memory Test").fg(Color::Red))
            .centered()
            .block(Block::bordered().border_set(border::DOUBLE))
            .render(vert[0], buf);

        let block = Block::bordered().border_set(border::DOUBLE);

        let main = vert[1].inner(Margin {
            horizontal: 1,
            vertical: 1,
        });

        match self.mode {
            Mode::Waiting => {
                block.title("╡ Playing field ╞").render(vert[1], buf);

                let constraints = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Min(0),
                        Constraint::Length(1),
                        Constraint::Min(0),
                    ])
                    .split(main);

                Paragraph::new("Click/Enter to start playing")
                    .centered()
                    .render(constraints[1], buf);
            }
            Mode::Playing => {
                block.title("╡ Playing ╞").render(vert[1], buf);
                let constraints = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Min(0),
                        Constraint::Length(1),
                        Constraint::Min(0),
                    ])
                    .split(main);

                let top_constraints = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Min(0),
                        Constraint::Length(1),
                        Constraint::Percentage(20),
                    ])
                    .split(constraints[0]);

                let percentages = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Min(0),
                        Constraint::Percentage(10),
                        Constraint::Percentage(20),
                        Constraint::Percentage(10),
                        Constraint::Min(0),
                    ])
                    .split(top_constraints[1]);

                Paragraph::new(format!("Score: {}", self.score))
                    .centered()
                    .render(percentages[1], buf);
                Paragraph::new(format!("Lives: {}", self.lives))
                    .centered()
                    .render(percentages[3], buf);

                Paragraph::new(WORDS[self.current])
                    .centered()
                    .render(constraints[1], buf);

                let buttons_vert = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Percentage(20),
                        Constraint::Length(3),
                        Constraint::Min(0),
                    ])
                    .split(constraints[2])[1];

                let buttons = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Min(0),
                        Constraint::Length(6),
                        Constraint::Percentage(20),
                        Constraint::Length(6),
                        Constraint::Min(0),
                    ])
                    .split(buttons_vert);

                Paragraph::new(Line::from(vec![
                    Span::styled("S", Style::default().underlined()),
                    Span::raw("EEN"),
                ]))
                .block(Block::bordered().border_set(border::DOUBLE))
                .render(buttons[1], buf);

                Paragraph::new(Line::from(vec![
                    Span::styled("N", Style::default().underlined()),
                    Span::raw("EW"),
                ]))
                .block(Block::bordered().border_set(border::DOUBLE))
                .render(buttons[3], buf);
            }
            Mode::Results => {
                block.title("╡ Results ╞").render(vert[1], buf);

                let results = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Min(0),
                        Constraint::Length(1),
                        Constraint::Length(1),
                        Constraint::Min(0),
                    ])
                    .split(main);

                Paragraph::new(format!("Your score is: {}", self.score))
                    .centered()
                    .render(results[1], buf);

                Paragraph::new(format!(
                    "Your avg score is: {:.0}",
                    self.savestate.avg_score
                ))
                .centered()
                .render(results[2], buf);
            }
        }
    }
}
