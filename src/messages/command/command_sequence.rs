use std::{fmt::Display, str::FromStr};

use crate::prelude::*;

#[derive(Clone, Debug, serde::Deserialize, Default)]
#[serde(transparent)]
pub struct CommandSequence {
    pub commands: Vec<Command>,
}

impl From<Command> for CommandSequence {
    fn from(single_command: Command) -> Self {
        Self {
            commands: vec![single_command],
        }
    }
}

impl From<Vec<Command>> for CommandSequence {
    fn from(commands: Vec<Command>) -> Self {
        Self { commands }
    }
}

impl<const N: usize> From<[Command; N]> for CommandSequence {
    fn from(value: [Command; N]) -> Self {
        value.into_iter().collect::<Vec<Command>>().into()
    }
}

impl<const N: usize> From<[&str; N]> for CommandSequence {
    fn from(value: [&str; N]) -> Self {
        value
            .iter()
            .map(|s| Command::from_str(s).unwrap()) // <- if this fails it should fail hard
            .collect::<Vec<Command>>()
            .into()
    }
}

impl From<&str> for CommandSequence {
    fn from(value: &str) -> Self {
        Command::from_str(value).unwrap().into() // <- if this fails it should fail hard
    }
}

impl Display for CommandSequence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut first = true;
        for command in self.commands.iter() {
            if !first {
                f.write_str(",")?;
            }
            command.fmt(f)?;

            first = false;
        }

        Ok(())
    }
}
