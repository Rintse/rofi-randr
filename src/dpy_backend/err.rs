use crate::action::{rate::Rate, resolution::Resolution};

#[derive(thiserror::Error, Debug)]
pub enum BackendCall { 
    #[error("xrandr CLI")]
    XrandrCLI(String),

    #[error("libxrandr")]
    LibXrandr(#[from] xrandr::XrandrError),
    
    #[error("swayipc")]
    SwayIPC(#[from] swayipc::Error),
    
    #[error("wayland-client")]
    WaylandClient(#[from] wayland_client::ConnectError),
}

#[derive(thiserror::Error, Debug)]
pub enum GetHandle { 
    #[error("Call in display backend failed:\n{0}")]
    BackendCall( #[from] BackendCall ),
}

#[derive(thiserror::Error, Debug)]
pub enum GetOutputs {
    #[error("Call in display backend failed:\n{0}")]
    BackendCall( #[from] BackendCall ),

    #[error("xrandr CLI")]
    XrandrCLI(String),
}

#[derive(thiserror::Error, Debug)]
pub enum GetResolutions { 
    #[error("Call in display backend failed:\n{0}")]
    BackendCall( #[from] BackendCall ),

    #[error("Could not find requested output ({0})")]
    NoOutput(String),

    #[error("Could not determine the current mode")]
    GetCurrent,
}

#[derive(thiserror::Error, Debug)]
pub enum SetResolution { 
    #[error("Call in display backend failed:\n{0}")]
    BackendCall( #[from] BackendCall ),

    #[error("Could not find requested output ({0})")]
    NoOutput(String),

    #[error("Could not find mode with requested resolution ({0:?})")]
    NoMode(Resolution),
}

#[derive(thiserror::Error, Debug)]
pub enum GetRates { 
    #[error("Call in display backend failed:\n{0}")]
    BackendCall( #[from] BackendCall ),

    #[error("Could not find requested output ({0})")]
    NoOutput(String),

    #[error("Could not determine the current mode")]
    GetCurrent,
}

#[derive(thiserror::Error, Debug)]
pub enum SetRate { 
    #[error("Call in display backend failed:\n{0}")]
    BackendCall( #[from] BackendCall ),

    #[error("Could not find requested output ({0})")]
    NoOutput(String),

    #[error("Output '{0}' has no current mode")]
    NoMode(String),

    #[error("Could not find requested rate ({0})")]
    NoRate(Rate),
}

#[derive(thiserror::Error, Debug)]
pub enum SetRotation { 
    #[error("Call in display backend failed:\n{0}")]
    BackendCall( #[from] BackendCall ),

    #[error("Could not find requested output ({0})")]
    NoOutput(String),
}

#[derive(thiserror::Error, Debug)]
pub enum SetPosition { 
    #[error("Call in display backend failed:\n{0}")]
    BackendCall( #[from] BackendCall ),

    #[error("Could not find requested output ({0})")]
    NoOutput(String),
}

#[derive(thiserror::Error, Debug)]
pub enum SetPrimary { 
    #[error("Call in display backend failed:\n{0}")]
    BackendCall( #[from] BackendCall ),

    #[error("Could not find requested output ({0})")]
    NoOutput(String),
}

#[derive(thiserror::Error, Debug)]
pub enum Enable { 
    #[error("Call in display backend failed:\n{0}")]
    BackendCall( #[from] BackendCall ),

    #[error("Could not find requested output ({0})")]
    NoOutput(String),
}

#[derive(thiserror::Error, Debug)]
pub enum Disable { 
    #[error("Call in display backend failed:\n{0}")]
    BackendCall( #[from] BackendCall ),

    #[error("Could not find requested output ({0})")]
    NoOutput(String),
}

/// Helps keep error propegation in the backend short
/// # Arguments
/// * `err_type` - the error that should be built from the backend error,
///     e.g. `GetResolutions`.
/// * `backend ` - The backend from which the error came, e.g. `XrandrCLI`.
/// * `args` - Potential arguments to the `backend` error type.
#[macro_export]
macro_rules! backend_call {
    ( $err_type:ident, $backend:ident, $( $args:expr ),*) => {
        super::err::DpyServerError::$err_type(
            super::err::$err_type::BackendCall(
                super::err::BackendCall::$backend($($args)*)))
    };
}

#[derive(thiserror::Error, Debug)]
pub enum DpyServerError { 
    #[error("Could not find fitting display server")]
    GetBackend,
    
    #[error("Could not open a connection to the display server ({0})")]
    GetHandle (#[from] GetHandle ),

    #[error("Could not get outputs from the display server:\n{0}")]
    GetOutputs( #[from] GetOutputs ),

    #[error("Could not get resolutions from the display server\n{0}")]
    GetResolutions( #[from] GetResolutions ),
    
    #[error("Could not set resolution in the display server\n{0}")]
    SetResolution( #[from] SetResolution ),
    
    #[error("Could not get rates from the display server\n{0}")]
    GetRates( #[from] GetRates ),
    
    #[error("Could not set rate:\n{0}")]
    SetRate( #[from] SetRate ),

    #[error("Could not set rate:\n{0}")]
    SetRotation( #[from] SetRotation ),

    #[error("Could not set position:\n{0}")]
    SetPosition( #[from] SetPosition ),
    
    #[error("Could not set display as primary:\n{0}")]
    SetPrimary( #[from] SetPrimary ),

    #[error("Could not enable display")]
    Enable( #[from] Enable ),

    #[error("Could not disable display")]
    Disable( #[from] Disable ),
}
