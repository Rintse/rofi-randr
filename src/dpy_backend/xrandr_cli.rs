use std::collections::VecDeque;
use std::io::BufRead;

use crate::action::Operation;
use crate::action::position::Relation;
use crate::action::position::Position;
use crate::action::rate::Rate;
use crate::action::rotate::Rotation;
use crate::action::resolution::Resolution;
use crate::backend_call as backend_call_err;
use crate::dpy_backend::err::DpyServerError;

use super::{OutputEntry, RateEntry, ResolutionEntry};

// Structs to parse the xrandr output into
#[derive(Debug,Clone)]
struct Mode {
    width: u32,
    height: u32,
    rate: f64,
    current: bool,
}
#[derive(Debug,Clone)]
struct Output {
    name: String,
    connected: bool,
    enabled: bool,
    modes: Vec<Mode>,
}

/// **NOTE:** this is an experimental backend for testing and is not
/// fit for everyday use. The parser it relies on is very unga bunga.
struct XrandrState { outputs: Vec<Output> }

// The modes are not space-separated, since the preferred marker can be 
// separated from the mode by a space. We must therefore read numeric chars
// until we have read a space, and then continue reading until we find the 
// next numeric character, which should be the start of the next mode
fn parse_mode_line(line: &str) -> (&str, Vec<&str>) {
    fn is_num(c: char) -> bool { c == '.' || c.is_ascii_digit() }

    let mut rates: Vec<&str> = Vec::new();
    let line = line.trim();
    let first_space = line.find(' ').unwrap();
    let res = line.get(0..first_space).unwrap();
    let line = line.get(first_space..).unwrap().trim();
    
    let mut start = 0;
    let mut i = 0;

    while i < line.len() {
        while i < line.len() && is_num(line.chars().nth(i).unwrap()) { i+=1 }

        while i < line.len() && !is_num(line.chars().nth(i).unwrap()) { i+=1 }
        let end = if i == line.len() { i } else { i-1 };
        rates.push(line.get(start..end).unwrap().trim());
        start = i;
    }

    (res, rates)
}

impl XrandrState {
    // The new() constructor calls `xrandr` and parses the result
    // TODO: this is very rough for now, should have many more checks
    fn new() -> Result<Self, DpyServerError> {
        let mut cmd = std::process::Command::new("xrandr");
        let res = cmd.output()
            .map_err(|e| backend_call_err!(GetOutputs, XrandrCLI, e.to_string()))?;

        let mut lines = res.stdout.lines().collect::<Result<VecDeque<String>,_>>()
            .unwrap(); // unrwap: error if not utf-8, should never happen

        let mut outputs: Vec<Output> = Vec::new();
        loop {
            let line = lines.pop_front();
            if line.is_none() { break }
            let line = line.unwrap(); // see above

            if line.get(..6) == Some("Screen") { continue }

            let mut words = line.split(' ').collect::<VecDeque<&str>>();
            let name = words.pop_front().unwrap().to_string();
            let connected = words.pop_front() == Some("connected");

            let mut enabled = false;
            let mut modes: Vec<Mode> = Vec::new();

            while !lines.is_empty() 
            && lines.front().unwrap().get(..3) == Some("   ") {
                let mode_line = lines.pop_front().unwrap();
                let (res, rates) = parse_mode_line(&mode_line);

                let width: u32 = res.split('x').next().unwrap().parse().unwrap();
                let height: u32 = res.split('x').nth(1).unwrap().parse().unwrap();

                for rate_s in rates {
                    let rate_stripped = rate_s.replace(&['*','+', ' '][..], "");
                    let rate: f64 = rate_stripped.parse().unwrap();
                    let current = rate_s.contains('*');
                    if current { enabled = true; }

                    modes.push(Mode { width, height, rate, current });
                }
            }
            outputs.push(Output { name, connected, enabled, modes });
        }
        Ok(XrandrState { outputs })
    }
}

pub struct Backend { state: XrandrState }

impl Backend {
    pub fn new() -> Result<Self, DpyServerError> { 
        Ok(Self{ state: XrandrState::new()? })
    }
}

// Tranform to a string that can be understood by xrandrs CLI
pub trait Xcl { fn xcl(&self) -> String; }

impl Xcl for Resolution {
    fn xcl(&self) -> String {
        format!("{}x{}", self.width, self.height)
    }
}

impl Xcl for Rotation {
    fn xcl(&self) -> String {
        match self {
            Rotation::Normal      => String::from("normal"),
            Rotation::Left        => String::from("left"),
            Rotation::Right       => String::from("right"),
            Rotation::Inverted    => String::from("inverted")
        }
    }
}

impl Xcl for Relation {
    fn xcl(&self) -> String {
        match self {
            Relation::LeftOf    => String::from("--left-of"),
            Relation::RightOf   => String::from("--right-of"),
            Relation::Above     => String::from("--above"),
            Relation::Below     => String::from("--below"),
            Relation::SameAs    => String::from("--same-as"),
        }
    }
}

const RATE_EPSILON: f64 = 0.01; // xrandr rates are rounded to 2 decimals


impl super::DisplayBackend for Backend {
    fn supported_operations(&mut self, output: &OutputEntry) -> Vec<Operation> {
        match (output.connected, output.enabled) {
            // If the output is not connected, just give the option
            // to disable/enable it. (X allows you to unplug an output 
            // while still having it as active)
            (false, _) => vec![ Operation::Disable ],

            // If the output is connected but disabled, only show enable option
            (_, false) => vec![ Operation::Enable ],

            // Otherwise, list all except enable
            _ => vec![
                Operation::Disable,
                Operation::SetPrimary,
                Operation::ChangeRes(Resolution::default()),
                Operation::Position(Position::default()),
                Operation::ChangeRate(Rate::default()),
                Operation::Rotate(Rotation::default()),
            ],
        }
    }

    fn supported_relations(&mut self) -> Vec<Relation> {
        vec![ 
            Relation::LeftOf, 
            Relation::RightOf, 
            Relation::Below, 
            Relation::Above,
            Relation::SameAs, 
        ]
    }

    fn get_outputs(&mut self) -> Result<Vec<OutputEntry>, DpyServerError> {
        let entries = self.state.outputs.iter()
            .map(|o| OutputEntry { 
                name: o.name.clone(),
                connected: o.connected,
                enabled: o.enabled })
            .collect();

        Ok(entries)
    }
    
    fn get_resolutions(&mut self, output_name: &str) 
    -> Result<Vec<ResolutionEntry>, DpyServerError> {
        let output = self.state.outputs.iter()
            .find(|o| o.name == output_name)
            .ok_or(super::err::GetResolutions::NoOutput(
                output_name.to_string()))?;

        let mut entries = output.modes.iter()
            .map(|m| ResolutionEntry {
                val: Resolution { width: m.width, height: m.height },
                current: m.current })
            .collect::<Vec<ResolutionEntry>>();

        entries.dedup_by(|a,b| 
            a.val.width == b.val.width && a.val.height == b.val.height);

        Ok(entries)
    }
    
    fn set_resolution(&mut self, output_name: &str, res: &Resolution) 
    -> Result<(), DpyServerError> {
        let mut cmd = std::process::Command::new("xrandr");
        let cmd = cmd.args([
            "--output", output_name,
            "--mode", &res.xcl(), 
        ]);

        let err_f = |s: String| backend_call_err!(SetResolution, XrandrCLI, s);
        run_and_check(cmd, err_f)
    }

    fn get_rates(&mut self, output_name: &str) 
    -> Result<Vec<RateEntry>, DpyServerError> {
        let output = self.state.outputs.iter()
            .find(|o| o.name == output_name)
            .ok_or(super::err::GetRates::NoOutput(output_name.to_string()))?;

        let current_mode = output.modes.iter()
            .find(|m| m.current)
            .ok_or(super::err::GetRates::GetCurrent)?;

        let entries = output.modes.iter()
            .filter(
                |m| m.height == current_mode.height
                && m.width == current_mode.width)
            .map(|m| RateEntry { 
                val: m.rate, 
                current: (m.rate-current_mode.rate).abs() < RATE_EPSILON })
            .collect();

        Ok(entries)
    }

    fn set_rate(&mut self, output_name: &str, rate: Rate) 
    -> Result<(), DpyServerError> {
        let mut cmd = std::process::Command::new("xrandr");
        let cur_res = self.state.outputs.iter()
            .find(|o| o.name == output_name)
            .ok_or(super::err::SetRate::NoOutput(output_name.to_string()))?
            .modes.iter()
            .find(|m| m.current)
            .ok_or(super::err::SetRate::NoMode(output_name.to_string()))?;

        let cmd = cmd.args([
            "--output", output_name,
            "--mode", &format!("{}x{}", cur_res.width, cur_res.height), 
            "--rate", &rate.to_string() 
        ]);
        
        let err_f = |s: String| backend_call_err!(SetRate, XrandrCLI, s);
        run_and_check(cmd, err_f)
    }

    fn set_rotation(&mut self, output_name: &str, rotation: &Rotation)
    -> Result<(), DpyServerError> {
        let mut cmd = std::process::Command::new("xrandr");
        let cmd = cmd.args([
            "--output", output_name, 
            "--rotate", &rotation.xcl(),
        ]);

        let err_f = |s: String| backend_call_err!(SetRotation, XrandrCLI, s);
        run_and_check(cmd, err_f)
    }
    
    fn set_position(&mut self, output_name: &str, pos: &Position)
    -> Result<(), DpyServerError> {
        let mut cmd = std::process::Command::new("xrandr");
        let cmd = cmd.args([
            "--output", output_name,
            &pos.relation.xcl(),
            &pos.output_s,
        ]);
        
        let err_f = |s: String| backend_call_err!(SetPosition, XrandrCLI, s);
        run_and_check(cmd, err_f)
    }
    
    fn set_primary(&mut self, output_name: &str) -> Result<(), DpyServerError> {
        let mut cmd = std::process::Command::new("xrandr");
        let cmd = cmd.args(["--output", output_name, "--primary"]);

        let err_f = |s: String| backend_call_err!(SetPrimary, XrandrCLI, s);
        run_and_check(cmd, err_f)
    }
    
    fn enable(&mut self, output_name: &str) -> Result<(), DpyServerError> {
        let mut cmd = std::process::Command::new("xrandr");
        let cmd = cmd.args(["--output", output_name, "--auto"]);

        let err_f = |s: String| backend_call_err!(Enable, XrandrCLI, s);
        run_and_check(cmd, err_f)
    }
    
    fn disable(&mut self, output_name: &str) -> Result<(), DpyServerError> {
        let mut cmd = std::process::Command::new("xrandr");
        let cmd = cmd.args(["--output", output_name, "--off"]);

        let err_f = |s: String| backend_call_err!(Disable, XrandrCLI, s);
        run_and_check(cmd, err_f)
    }
}

// Helper function to improve the readibility of the error handling in the
// interface functions above. Relies on the fact that we only put strings
// inside the errors for this backend.
fn run_and_check(
    cmd: &mut std::process::Command, 
    err_f: fn(s: String)->DpyServerError
) -> Result<(),DpyServerError>
{
    let res = cmd.output()
        .map_err(|_| err_f("Could not execute command".to_string()))?;

    if res.status.success() { Ok(()) } 
    else {
        let stderr = String::from_utf8(res.stderr)
            .map_err(|_| err_f("Unknown error".to_string()))?;
        Err(err_f(stderr))
    }
}

