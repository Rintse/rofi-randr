// Top level errors

use crate::dpy_backend::err::DpyServerError;
use xrandr::XrandrError;
use thiserror::Error;

#[derive(Error,Debug)]
pub enum ParseError { 
    #[error("Invalid resolution: {0}")]
    Resolution(String),

    #[error("Invalid position: {0}")]
    Position(String),

    #[error("Invalid direction: {0}")]
    Relation(String),

    #[error("Invalid rotaiton: {0}")]
    Rotation(String),

    #[error("Invalid rate: {0}")]
    Rate(String),

    #[error("Invalid operation: '{0}'") ]
    Operation(String),
}

// Global level errors
#[derive(Debug, Error)]
pub enum AppError {
    #[error("Something went wrong in the display backend:\n{source}") ]
    BackendErr { #[from] source: DpyServerError },

    #[error("Call to libxrandr failed") ]
    Lib { #[from] source: XrandrError },

    #[error("Call to libxrandr failed") ]
    Cmd,

    #[error("Parsing of rofi input failed") ]
    Parse { #[from] source: ParseError },

    #[error("No modes for requested resolution found") ]
    NoModes,
    
    #[error("No output found for the name {0}") ]
    NoOuput(String),

    #[error("Invalid operation '{0}' on disabled display") ]
    Disabled(String),

    #[error("Operation cancelled by user")]
    Cancel,
}

