mod action;
mod backend;
mod err;
mod icon;
mod rofi;

use action::{Action, ParseResult};
use err::AppError;

use itertools::Itertools;
use rofi::List;
use std::{collections::VecDeque, env};

fn get_args() -> VecDeque<String> {
    // ROFI_DATA env var contains the chosen arguments to the script so far
    let mut rofi_data: VecDeque<String> = match env::var("ROFI_DATA") {
        Err(_) => VecDeque::new(), // no args yet
        Ok(data_s) => data_s
            .split(':')
            .filter(|s| !s.is_empty())
            .map(String::from).collect(),
    };

    // The latest chosen argument is passed as arg to this program
    let arg = env::args().nth(1);
    if let Some(a) = arg {
        // Split on start of first pango tag:
        // - only comments have markup, so all that comes before is unput
        // Unwrap: first element of a split always exists
        let input = a.split('<').next().unwrap().trim().to_string();

        // If the user chose back, keep the data as it was the before
        if input == "Back" {
            rofi_data.pop_back();
        } else {
            rofi_data.push_back(input);
        }
    }


    // Store choices made for next iteration
    if rofi_data.is_empty() {
        println!("\0data\x1f"); // Reset in case of `Back`
    } else {
        println!("\0data\x1f{}", rofi_data.iter().join(":"));
    }
    
    rofi_data
}

fn run() -> Result<(), AppError> {
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

fn main() {
    match run() {
        Ok(()) => { std::process::exit(0); }
        Err(e) => {
            List::error(&format!("{e}")).rofi_print();
            std::process::exit(1)
        }
    }
}
