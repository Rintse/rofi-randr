use crate::action::ParseCtx;
use crate::ParseResult;
use crate::Action;
use crate::backend::DisplayBackend;
use std::{str::FromStr, fmt};
use crate::err::{ParseError, AppError};

#[derive(Debug,Default)]
pub enum Relation {
    #[default] SameAs,
    LeftOf,
    RightOf,
    Above,
    Below,
}

impl From<&Relation> for xrandr::Relation {
    fn from(relation: &Relation) -> Self {
        match relation {
            Relation::LeftOf    => xrandr::Relation::LeftOf,
            Relation::RightOf   => xrandr::Relation::RightOf,
            Relation::Above     => xrandr::Relation::Above,
            Relation::Below     => xrandr::Relation::Below,
            Relation::SameAs    => xrandr::Relation::SameAs,
        }
    }
}

impl fmt::Display for Relation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let pos_s = match self {
            Relation::LeftOf    => "To the left of",
            Relation::RightOf   => "To the right of",
            Relation::Above     => "Above",
            Relation::Below     => "Below",
            Relation::SameAs    => "Mirroring"
        };

        write!(f, "{pos_s} ")
    }
}

impl FromStr for Relation {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "To the left of"    => Ok(Relation::LeftOf),
            "To the right of"   => Ok(Relation::RightOf),
            "Above"             => Ok(Relation::Above),
            "Below"             => Ok(Relation::Below),
            "Mirroring"         => Ok(Relation::SameAs),
            _ => Err(Self::Err::Relation(s.to_string()))
        }
    }
}

#[derive(Debug,Default)]
pub struct Position {
    pub relation : Relation,
    pub output_s : String
}

impl FromStr for Position {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let data : Vec<&str> = s.split(' ').collect();
        if data.len() != 2 { 
            return Err(Self::Err::Position(s.to_string())); 
        }

        Ok(Position { 
            relation: Relation::from_str(data[0])?,
            output_s: data[1].to_string()
        })
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.relation, self.output_s)
    }
}

impl Position {
    pub fn parse(backend: &mut Box<dyn DisplayBackend>, ctx: ParseCtx) 
    -> Result<ParseResult<Action>, AppError> 
    {
        let ParseCtx { output, mut args } = ctx;

        let relation = match args.pop_front() {
            None => return Ok(ParseResult::relation_list(backend)),
            Some(rel_s) => Relation::from_str(&rel_s)
        }?;

        Ok(match args.pop_front() {
            None => return ParseResult::relatives_list(backend, &output, &relation),
            Some(o2) => ParseResult::position(output, relation, &o2)
        })
    }
}
