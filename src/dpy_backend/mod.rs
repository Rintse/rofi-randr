use crate::action::Operation;
use crate::action::position::{Position, Relation};
use crate::action::rate::Rate;
use crate::action::rotate::Rotation;
use crate::action::resolution::Resolution;
use self::err::DpyServerError;
use std::env;

pub mod err;
mod libxrandr;
mod xrandr_cli;
mod sway;

pub(crate) fn backend_from_name(name: &str) 
-> Result<Box<dyn DisplayBackend>, DpyServerError> {
    match name {
        "libxrandr" => Ok(Box::new(libxrandr::Backend::new()?)),
        "xrandr_cli" => Ok(Box::new(xrandr_cli::Backend::new()?)),
        "swayipc" => Ok(Box::new(sway::Backend::new()?)),
        _ => Err(DpyServerError::GetBackend)
    }
}

// TODO: this is a bit hacky atm
/// Gets the appropriate backend based on environment variables
pub(crate) fn determine_backend() 
-> Result<Box<dyn DisplayBackend>, DpyServerError> {
    match env::var("XDG_SESSION_TYPE") {
        Ok(name) => match name.as_str() {
            "x11" => backend_from_name("libxrandr"),
            "wayland" => match env::var("SWAYSOCK") {
                Ok(_) => backend_from_name("swayipc"),
                Err(_) => Err(DpyServerError::GetBackend),
            }
            _ => Err(DpyServerError::GetBackend),
        },
        Err(_) => Err(DpyServerError::GetBackend),
    }
}

// Defines the API that this application wants from the dpy server
pub trait DisplayBackend {
    // The supported operations for this backend
    // Takes output as argument because ops might change depending on its state
    fn supported_operations(&mut self, output: &OutputEntry) -> Vec<Operation>;
    
    // This is needed because sway does not really support mirroring
    fn supported_relations(&mut self) -> Vec<Relation>;

    fn get_outputs(&mut self) -> Result<Vec<OutputEntry>, DpyServerError>;

    fn get_resolutions(&mut self, output_name: &str) 
    -> Result<Vec<ResolutionEntry>, DpyServerError>;
    
    fn set_resolution(&mut self, output_name: &str, res: &Resolution) 
    -> Result<(), DpyServerError>;

    fn get_rates(&mut self, output_name: &str) 
    -> Result<Vec<RateEntry>, DpyServerError>;
    
    fn set_rate(&mut self, output_name: &str, rate: Rate) 
    -> Result<(), DpyServerError>;
    
    fn set_rotation(&mut self, output_name: &str, rotation: &Rotation)
    -> Result<(), DpyServerError>;
    
    fn set_position(&mut self, output_name: &str, pos: &Position)
    -> Result<(), DpyServerError>;
    
    fn set_primary(&mut self, output_name: &str) -> Result<(), DpyServerError>;
    
    fn enable(&mut self, output_name: &str) -> Result<(), DpyServerError>;
    
    fn disable(&mut self, output_name: &str) -> Result<(), DpyServerError>;
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
