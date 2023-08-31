use crate::{err::{ParseError, AppError}, backend::DisplayBackend};
use std::str::FromStr;

use super::{ParseResult, Action, ParseCtx};

// Usually i want to pick resolutions and rates separately
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Resolution {
    pub width: u32,
    pub height: u32,
}

impl FromStr for Resolution {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let e = Self::Err::Resolution(s.to_string());

        let data : Vec<&str> = s.split('x').collect();
        if data.len() != 2 { return Err(e); }

        let size_res: Result<Vec<u32>,_> = 
            data.iter().map(|s| s.parse::<u32>()).collect();

        let size = size_res.map_err(|_|e)?;
        let (width, height) = (size[0], size[1]);

        Ok( Resolution { width, height } )
    }
}

impl From<&xrandr::Mode> for Resolution {
    fn from(m: &xrandr::Mode) -> Self {
        Resolution { width : m.width, height : m.height }
    }
}

impl Resolution {
    pub fn parse(backend: &mut Box<dyn DisplayBackend>, ctx: ParseCtx) 
    -> Result<ParseResult<Action>, AppError> 
    {
        let ParseCtx { output, mut args } = ctx;

        Ok(match args.pop_front() {
            None => ParseResult::resolution_list(backend, &output)?,
            Some(res_s) => {
                let mode = Resolution::from_str(&res_s)?;
                ParseResult::resolution(output, mode)
            }
        })
    }
}
