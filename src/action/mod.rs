pub mod rotate;
pub mod resolution;
pub mod position;
pub mod rate;

use crate::backend::DisplayBackend;
use crate::backend::OutputEntry;
use crate::rofi::List as RofiList;
use std::collections::VecDeque;
use itertools::Itertools;
use std::fmt;

use crate::err::ParseError;
use crate::action::position::Relation;
use crate::action::position::Position;
use crate::action::rotate::Rotation;
use crate::action::rate::parse as parse_rate;
use crate::action::resolution::Resolution;
use crate::err::AppError;


#[derive(Debug)]
pub enum Operation {
    Enable,
    Disable,
    SetPrimary,
    ChangeRes(Resolution),
    Position(Position),
    ChangeRate(f64),
    Rotate(Rotation),
}

#[derive(Debug)]
pub struct Action {
    output : String,
    op : Operation,
}

// To list the possible operations
impl fmt::Display for Operation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let op_s = match self {
            Operation::Enable           => "Enable",
            Operation::Disable          => "Disable",
            Operation::SetPrimary       => "Make primary",
            Operation::ChangeRes(_)     => "Change resolution",
            Operation::ChangeRate(..)   => "Change rate",
            Operation::Position(_)      => "Position",
            Operation::Rotate(_)        => "Rotate",
        };
        write!(f, "{op_s} ")
    }
}

// Apply the action: just constructs and calls a command
impl Action {
    pub fn apply(&self, mut backend: Box<dyn DisplayBackend>) 
    -> Result<(), AppError> {
        let output = &self.output;

        Ok(match &self.op {
            Operation::Enable => backend.enable(output),
            Operation::Disable => backend.disable(output),
            Operation::SetPrimary => backend.set_primary(output),
            Operation::ChangeRes(res) => backend.set_resolution(output, res),
            Operation::ChangeRate(rate) => backend.set_rate(output, *rate),
            Operation::Rotate(r) => backend.set_rotation(output, r),
            Operation::Position(p) => backend.set_position(output, p),
        }?)
    }
}

// A partial parse can result in two things:
// - There is still some missing argument
//     > Give a list of the possible values for next arg
// - The object is parsed completely.
// - (Or we can encounter an error parsing ofc: PartParseError)
#[derive(Debug)]
pub enum ParseResult<A> {
    Done(A),
    Next(RofiList)
}

// Shorthand constructors for readability in the parser function
// TODO: is there a better way to do this?
impl ParseResult<Action> {
    // Constructors. lots of duplication here..
    fn enable(output : String) -> Self {
        Self::Done( Action { output, op: Operation::Enable } )
    }

    fn disable(output : String) -> Self {
        Self::Done(Action { output, op: Operation::Disable })
    }
    
    fn primary(output : String) -> Self {
        Self::Done(Action { output, op: Operation::SetPrimary })
    }
    
    fn resolution(output: String, m : Resolution) -> Self {
        Self::Done(Action { output, op: Operation::ChangeRes(m) })
    }

    // TODO: we currently need the cur_res because of the apply backend 
    // that just calls xrandr. Ideally we just contact the xrandr backend
    fn rate(output: String, rate: f64) -> Self {
        Self::Done(Action { 
            output, op: Operation::ChangeRate(rate)
        })
    }

    fn rotate(output: String, r : Rotation) -> Self {
        Self::Done(Action { output, op : Operation::Rotate(r) })
    }

    fn position(output: String, rel : Relation, o2 : &str) -> Self {
        Self::Done(Action { output, op : Operation::Position(
            Position { relation: rel, output_s: o2.to_string() }
        )})
    }
}
    
// xrandr lets you disable your last display, leaving your system in a
// hard to recover state. This function prompts you on whether you really
// want to disable your last display.
fn confirm_last_display_disable(
    outputs : &[OutputEntry], mut ctx: ParseCtx) 
-> Result<ParseResult<Action>, AppError> {
    if let Some(confirmation) = ctx.args.pop_front() {
        return match confirmation.as_str() {
            "Yes" => Ok(ParseResult::disable(ctx.output)),
            _ => Err(AppError::Cancel), // TODO: this is not really an error
        };
    }

    // There are no other displays that are connected: prompt to confirm
    if !outputs.iter().any(|o| o.name != ctx.output && o.enabled) {
        return Ok(ParseResult::confirm_disable_list())
    }

    // Otherwise, immediately disable.
    Ok(ParseResult::enable(ctx.output))
       
}

#[derive(Debug)]
pub struct ParseCtx {
    output: String,
    args: VecDeque<String>,
}


impl Action {
    // Parse needed arguments for an action, and returns the 
    // generated action If not all arguments are present yet, 
    // a list of options for the next argument is returned instead
    pub fn parse(backend: &mut Box<dyn DisplayBackend>, mut args : VecDeque<String>) 
    -> Result<ParseResult<Self>, AppError> 
    {
        let outputs = backend.get_outputs()?;

        // First argument should be the output
        let output = match args.pop_front() {
            None => return ParseResult::output_list(backend),
            Some(name) => outputs.iter()
                .find(|o| o.name == name)
                .ok_or(AppError::NoOuput(name))?,
        };

        // No arguments further args, list possible operations on the output
        let op_str = match args.pop_front() {
            None => return Ok(ParseResult::operation_list(backend, output)),
            Some(op_s) => op_s,
        };

        // Operation provided, parse its arguments
        // Clone to be able to print the input in case of error
        let ctx = ParseCtx { output: output.name.clone(), args: args.clone() };

        let action_p : ParseResult<Self> = match op_str.as_str() {
            // Nullary actions, return the action
            "Enable"        => ParseResult::enable(ctx.output),
            "Disable"       => confirm_last_display_disable(&outputs, ctx)?,
            "Make primary"  => ParseResult::primary(ctx.output),

            // Unary/binary, parse further
            "Change resolution"     => Resolution::parse(backend, ctx)?,
            "Rotate"                => Rotation::parse(ctx)?,
            "Change rate"           => parse_rate(backend, ctx)?,
            "Position"              => Position::parse(backend, ctx)?,

            // If not handled now, this is an invalid action
            _ => return Err(ParseError::Operation(
                format!("{} ({})", op_str, args.iter().join(", "))
            ))?
        };

        Ok(action_p)
    }
}

