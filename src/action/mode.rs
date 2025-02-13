use crate::{
    backend::DisplayBackend,
    err::{AppError, ParseError},
};
use std::{cmp::Ordering, str::FromStr};

use super::{Action, ParseCtx, ParseResult};

// Usually i want to pick resolutions and rates separately
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Mode {
    pub width: u32,
    pub height: u32,
    pub rate: f64,
}

impl Eq for Mode {}

impl PartialOrd for Mode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(Self::cmp(self, other))
    }
}

impl Ord for Mode {
    // Sort on total pixels, then width, then rate.
    // No need for a height comparison, because heights must be equal if
    // both px count and width are equal
    fn cmp(&self, other: &Self) -> Ordering {
        let px_count_ord = u32::cmp(
            &(self.width * self.height),
            &(other.width * other.height),
        );
        let width_ord = u32::cmp(&self.width, &other.width);
        let rate_ord = f64::total_cmp(&self.rate, &other.rate);

        px_count_ord.then(width_ord).then(rate_ord)
    }
}

impl FromStr for Mode {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let err = Self::Err::Resolution(s.to_string());

        let mut rate_split = s.split('@');
        let resolution_s = rate_split.next().ok_or(err.clone())?;
        let rate_s = rate_split.next().ok_or(err.clone())?;
        // Strip the " Hz" that was printed in the menu
        // see: From<&RateEntry> for ListItem
        let rate_stripped = &rate_s[..rate_s.len() - 2];
        let rate = f64::from_str(rate_stripped)
            .map_err(|_| ParseError::Rate(rate_s.to_string()))?;

        let mut resolution_split = resolution_s.split('x');
        let width = resolution_split
            .next()
            .ok_or(err.clone())?
            .parse::<u32>()
            .map_err(|_| err.clone())?;
        let height = resolution_split
            .next()
            .ok_or(err.clone())?
            .parse::<u32>()
            .map_err(|_| err.clone())?;

        Ok(Mode {
            width,
            height,
            rate,
        })
    }
}

impl From<&xrandr::Mode> for Mode {
    fn from(m: &xrandr::Mode) -> Self {
        Mode {
            width: m.width,
            height: m.height,
            rate: m.rate,
        }
    }
}

impl Mode {
    pub fn parse(
        backend: &mut Box<dyn DisplayBackend>,
        ctx: ParseCtx,
    ) -> Result<ParseResult<Action>, AppError> {
        let ParseCtx { output, mut args } = ctx;

        Ok(match args.pop_front() {
            None => ParseResult::mode_list(backend, &output)?,
            Some(res_s) => {
                let mode = Mode::from_str(&res_s)?;
                ParseResult::mode(output, mode)
            }
        })
    }
}
