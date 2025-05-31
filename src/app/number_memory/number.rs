use std::{
    fmt::Display,
    ops::{Deref, DerefMut},
};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Number(Vec<char>);

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
