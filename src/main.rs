mod action;
mod icon;
mod err;
mod rofi;
mod backend;

use action::{Action, ParseResult};
use err::AppError;

use std::{env, collections::VecDeque};
use itertools::Itertools;

fn get_args() -> VecDeque<String> {
    // ROFI_DATA env var contains the chosen arguments to the script so far
    let mut rofi_data : VecDeque<String> = match env::var("ROFI_DATA") {
        Err(_) => VecDeque::new(), // no args yet
        Ok(data_s) => data_s.split(':').map(String::from).collect(),
    };

    // The latest chosen argument is passed as arg to this program 
    let arg = env::args().nth(1);
    if let Some(a) = arg { 
        // Split on start of first pango tag: 
        // - only the comments have markup, so all that comes before is unput
        // Unwrap: first element of a split always exists
        rofi_data.push_back(a.split('<').next().unwrap().trim().to_string()); 
    }
        
    // Store choices made for next iteration
    if !rofi_data.is_empty() {
        println!("\0data\x1f{}", rofi_data.iter().join(":")); 
    }

    rofi_data
}

fn main() -> Result<(), AppError> {
    // Allow override of automatic backend trough env var
    let mut backend = match env::var("DISPLAY_SERVER_OVERRIDE") {
        Ok(name) => backend::from_name(&name)?,
        Err(_) => backend::determine()?,
    };

    match Action::parse(&mut backend, get_args())? {
        // Still something missing, list next set of options
        ParseResult::Next(options) => options.rofi_print(),
        // We have a full action, apply it
        ParseResult::Done(action) => action.apply(backend)?,
    }
    
    Ok(())
}
