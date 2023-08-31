pub mod err;
mod libxrandr;
mod xrandr_cli;
mod sway;

use crate::action::Operation;
use crate::action::position::{Position, Relation};
use crate::action::rate::Rate;
use crate::action::rotate::Rotation;
use crate::action::resolution::Resolution;
pub(crate) use self::err::Error as Error;
use std::env;


pub(crate) fn from_name(name: &str) 
-> Result<Box<dyn DisplayBackend>, Error> {
    match name {
        "libxrandr" => Ok(Box::new(libxrandr::Backend::new()?)),
        "xrandr_cli" => Ok(Box::new(xrandr_cli::Backend::new()?)),
        "swayipc" => Ok(Box::new(sway::Backend::new()?)),
        _ => Err(Error::GetBackend)
    }
}

// TODO: this is a bit hacky atm
/// Gets the appropriate backend based on environment variables
pub(crate) fn determine() 
-> Result<Box<dyn DisplayBackend>, Error> {
    match env::var("XDG_SESSION_TYPE") {
        Ok(name) => match name.as_str() {
            "x11" => from_name("libxrandr"),
            "wayland" => match env::var("SWAYSOCK") {
                Ok(_) => from_name("swayipc"),
                Err(_) => Err(Error::GetBackend),
            }
            _ => Err(Error::GetBackend),
        },
        Err(_) => Err(Error::GetBackend),
    }
}

/// Defines the API that this application wants with the display server
pub trait DisplayBackend {
    // The supported operations for this backend
    // Takes output as argument because ops might change depending on its state
    fn supported_operations(&mut self, output: &OutputEntry) -> Vec<Operation>;
    
    // This is needed because sway does not really support mirroring
    fn supported_relations(&mut self) -> Vec<Relation>;

    fn get_outputs(&mut self) -> Result<Vec<OutputEntry>, Error>;

    fn get_resolutions(&mut self, output_name: &str) 
    -> Result<Vec<ResolutionEntry>, Error>;
    
    fn set_resolution(&mut self, output_name: &str, res: &Resolution) 
    -> Result<(), Error>;

    fn get_rates(&mut self, output_name: &str) 
    -> Result<Vec<RateEntry>, Error>;
    
    fn set_rate(&mut self, output_name: &str, rate: Rate) 
    -> Result<(), Error>;
    
    fn set_rotation(&mut self, output_name: &str, rotation: &Rotation)
    -> Result<(), Error>;
    
    fn set_position(&mut self, output_name: &str, pos: &Position)
    -> Result<(), Error>;
    
    fn set_primary(&mut self, output_name: &str) -> Result<(), Error>;
    
    fn enable(&mut self, output_name: &str) -> Result<(), Error>;
    
    fn disable(&mut self, output_name: &str) -> Result<(), Error>;
}

#[derive(Debug,Clone)]
pub struct OutputEntry { 
    pub name: String,
    pub connected: bool,
    pub enabled: bool,
}

#[derive(Debug,Clone,PartialEq,Eq)]
pub struct ResolutionEntry {
    pub val: Resolution,
    pub current: bool,
}

#[derive(Debug,Clone)]
pub struct RateEntry { 
    pub val: Rate,
    pub current: bool,
}
