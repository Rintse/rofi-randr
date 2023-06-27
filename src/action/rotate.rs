use crate::AppError;
use core::fmt;
use crate::err::ParseError;
use std::str::FromStr;
use strum_macros::EnumIter;
use super::{ParseResult, Action, ParseCtx};

#[derive(Debug,Default,EnumIter)]
pub enum Rotation {
    #[default] Normal,
    Left,       // Counterclockwise
    Right,      // Clockwise
    Inverted    // Upside down
}

impl From<&Rotation> for xrandr::Rotation {
    fn from(r : &Rotation) -> Self {
        match r {
            Rotation::Normal    => xrandr::Rotation::Normal,
            Rotation::Left      => xrandr::Rotation::Left,
            Rotation::Right     => xrandr::Rotation::Right,
            Rotation::Inverted  => xrandr::Rotation::Inverted,
        }
    }
}

impl fmt::Display for Rotation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let pos_s = match self {
            Rotation::Normal    => "Normal",
            Rotation::Left      => "Left",
            Rotation::Right     => "Right",
            Rotation::Inverted  => "Inverted",
        };

        write!(f, "{pos_s} ")
    }
}

impl Rotation {
    // Alternative phrasings for clarity
    pub fn explain(&self) -> String { 
        match self {
            Rotation::Normal    => String::from("Upright"),
            Rotation::Left      => String::from("Counterclockwise"),
            Rotation::Right     => String::from("Clockwise"),
            Rotation::Inverted  => String::from("upside down"),
        }
    }

    pub fn parse(ctx: ParseCtx) 
    -> Result<ParseResult<Action>, AppError> 
    {
        let ParseCtx { output, mut args } = ctx;
        
        Ok(match args.pop_front() {
            None => ParseResult::rotation_list(),
            Some(rot_s) => {
                let rotation = Rotation::from_str(&rot_s)?;
                ParseResult::rotate(output, rotation)
            }
        })
    }
}

impl FromStr for Rotation {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Normal"    => Ok(Rotation::Normal),
            "Left"      => Ok(Rotation::Left),
            "Right"     => Ok(Rotation::Right),
            "Inverted"  => Ok(Rotation::Inverted),
            _           => Err(Self::Err::Rotation(s.to_string()))
        }
    }
}
