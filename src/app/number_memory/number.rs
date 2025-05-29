use std::{
    fmt::Display,
    ops::{Deref, DerefMut},
};

use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Number(Vec<char>);

impl Number {
    pub fn get_styled_text(&self, other: &Self) -> Line {
        if self.len() != other.len() {
            panic!("something went wroong");
        }

        let mut text = Line::default();

        for (i, &x) in self.iter().enumerate() {
            if x == other[i] {
                text += Span::raw(x.to_string());
            } else {
                text += Span::styled(x.to_string(), Style::default().bg(Color::Red));
            }
        }

        text
    }

    pub fn get_wrong_styled_text(&self, other: &Self) -> Line {
        if self.len() != other.len() {
            panic!("something went wroong");
        }

        let mut text = Line::default();

        for (i, &x) in self.iter().enumerate() {
            if x == other[i] {
                text += Span::styled(x.to_string(), Style::default().fg(Color::Green));
            } else {
                text += Span::styled(other[i].to_string(), Style::default().fg(Color::Red));
            }
        }

        text
    }
}

impl Deref for Number {
    type Target = Vec<char>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Number {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Display for Number {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = self.iter().collect::<String>();

        write!(f, "{string}")
    }
}
