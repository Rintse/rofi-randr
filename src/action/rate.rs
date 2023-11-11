use super::{Action, ParseCtx, ParseResult};
use crate::AppError;
use crate::{backend::DisplayBackend, err::ParseError};
use std::str::FromStr;

pub type Rate = f64;

pub fn parse(
    backend: &mut Box<dyn DisplayBackend>,
    ctx: ParseCtx,
) -> Result<ParseResult<Action>, AppError> {
    let ParseCtx { output, mut args } = ctx;

    let result = if let Some(rate_s) = args.pop_front() {
        // Strip the " Hz" that was printed in the menu
        // see: From<&RateEntry> for ListItem
        let rate_stripped = &rate_s[..rate_s.len() - 3];

        let rate = f64::from_str(rate_stripped)
            .map_err(|_| ParseError::Rate(rate_s.to_string()))?;

        ParseResult::rate(output, rate)
    } else {
        ParseResult::rate_list(backend, &output)?
    };

    Ok(result)
}
