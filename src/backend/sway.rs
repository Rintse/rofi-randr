use crate::action::mode::Mode;
use crate::action::position::Relation;
use crate::action::rotate::Rotation;
use crate::action::{position::Position, Operation};
use crate::backend::Error as BackendError;
use crate::backend_call as backend_call_err;
use swayipc::Connection;

use super::{ModeEntry, OutputEntry};

pub struct Backend {
    conn: Connection,
}

impl Backend {
    pub fn new() -> Result<Self, BackendError> {
        let conn =
            swayipc::Connection::new().map_err(|_| BackendError::GetBackend)?;

        Ok(Self { conn })
    }
}

// swayipc rates are frames per 1000 seconds with roughly 4 significant digits.
// Any two rates with less than `RATE_EPSILON` difference are considered to be
// equivalent
const RATE_EPSILON: f64 = 0.01;

// Helper function to deal with unwrapping the various layers of errors
// that result from swayipc's run_command() function. Maps all the errors we
// find to some backend error using the `err_f` function.
fn run_sway_cmd(
    conn: &mut swayipc::Connection,
    cmd: String,
    err_f: fn(swayipc::Error) -> BackendError,
) -> Result<(), BackendError> {
    let res = conn.run_command(cmd);

    // This first result seems to be whether we could even interface
    // with sway to execute the command
    let res = res.map_err(err_f)?;

    // This is a vector of results for each command. For now just fail if any of
    // them fail. Probably should log what fails
    let res: Result<Vec<_>, _> = res.into_iter().collect();
    res.map(|_| ()).map_err(err_f)
}

// Normalizes all output's positions such that the top left is at (0,0)
fn normalize_all_outputs(outputs: &[&swayipc::Output]) -> Vec<swayipc::Output> {
    let (left, top): (i32, i32) = outputs
        .iter()
        .map(|o| (o.rect.x, o.rect.y))
        .reduce(|(x1, y1), (x2, y2)| (i32::min(x1, x2), i32::min(y1, y2)))
        .expect("There should always be at least one output");

    let offset_position = |o: &&swayipc::Output| {
        let mut output = (*o).clone();
        output.rect.x -= left;
        output.rect.y -= top;
        output
    };

    outputs.iter().map(offset_position).collect()
}

impl super::DisplayBackend for Backend {
    fn supported_operations(&mut self, output: &OutputEntry) -> Vec<Operation> {
        match (output.connected, output.enabled) {
            (false, _) => {
                unreachable!("SwayIPC does not list disconnected outputs")
            }

            // If the output is connected but disabled, only show enable option
            (_, false) => vec![Operation::Enable],

            _ => vec![
                Operation::Disable,
                Operation::ChangeMode(Mode::default()),
                Operation::Position(Position::default()),
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
        ]
    }

    fn get_outputs(&mut self) -> Result<Vec<OutputEntry>, BackendError> {
        let sway_outputs = self
            .conn
            .get_outputs()
            .map_err(|e| backend_call_err!(GetOutputs, SwayIPC, e))?;

        let entries = sway_outputs
            .iter()
            .map(|o| OutputEntry {
                name: o.name.clone(),
                connected: true, // swayipc only lists connected outputs
                enabled: o.current_mode.is_some(),
            })
            .collect();

        Ok(entries)
    }

    fn get_modes(
        &mut self,
        output_name: &str,
    ) -> Result<Vec<ModeEntry>, BackendError> {
        let outputs = self
            .conn
            .get_outputs()
            .map_err(|e| backend_call_err!(GetResolutions, SwayIPC, e))?;
        let output = outputs.iter().find(|o| o.name == output_name).ok_or(
            super::err::GetResolutions::NoOutput(output_name.to_string()),
        )?;

        let current_mode = output
            .current_mode
            .ok_or(super::err::GetResolutions::GetCurrent)?;

        let mut entries = output
            .modes
            .iter()
            .map(|m| ModeEntry {
                val: Mode {
                    width: m.width as u32,
                    height: m.height as u32,
                    rate: f64::from(m.refresh),
                },
                current: m.width == current_mode.width
                    && m.height == current_mode.height,
            })
            .collect::<Vec<_>>();

        entries.sort_by(|a, b| Mode::cmp(&a.val, &b.val));
        entries.dedup();
        Ok(entries)
    }

    fn set_mode(
        &mut self,
        output_name: &str,
        mode: &Mode,
    ) -> Result<(), BackendError> {
        let outputs = self
            .conn
            .get_outputs()
            .map_err(|e| backend_call_err!(SetResolution, SwayIPC, e))?;
        let output = outputs.iter().find(|o| o.name == output_name).ok_or(
            super::err::SetResolution::NoOutput(output_name.to_string()),
        )?;

        let target_mode = output
            .modes
            .iter()
            .find(|m| {
                (f64::from(m.refresh) - mode.rate).abs() < RATE_EPSILON
                && m.width as u32 == mode.width
                    && m.height as u32 == mode.height
            })
            .ok_or(super::err::SetResolution::NoMode(mode.clone()))?;

        let mode_str = format!(
            "{}x{}@{}Hz",
            target_mode.width,
            target_mode.height,
            f64::from(target_mode.refresh) / 1000.0
        );

        let cmd = format!("output {} mode {}", output.name, mode_str);
        let mut res = self
            .conn
            .run_command(cmd)
            .map_err(|e| backend_call_err!(SetResolution, SwayIPC, e))?;
        res.pop()
            .unwrap()
            .map_err(|e| backend_call_err!(SetResolution, SwayIPC, e))?;

        Ok(())
    }

    fn set_rotation(
        &mut self,
        output_name: &str,
        rotation: &Rotation,
    ) -> Result<(), BackendError> {
        let angle_str = match rotation {
            Rotation::Normal => "0",
            Rotation::Left => "90",
            Rotation::Inverted => "180",
            Rotation::Right => "270",
        };

        let outputs = self
            .conn
            .get_outputs()
            .map_err(|e| backend_call_err!(SetRotation, SwayIPC, e))?;
        let output = outputs
            .iter()
            .find(|o| o.name == output_name)
            .ok_or(super::err::Enable::NoOutput(output_name.to_string()))?;

        let err_f = |e| backend_call_err!(SetRotation, SwayIPC, e);
        let cmd = format!("output {} transform {}", output.name, angle_str);

        run_sway_cmd(&mut self.conn, cmd, err_f)
    }

    // This is not really supported in sway-output, but it can be easily
    // done through the geometry of the displays + the pos command
    fn set_position(
        &mut self,
        output_name: &str,
        pos: &Position,
    ) -> Result<(), BackendError> {
        let Position {
            output_s: rel_output,
            relation,
        } = pos;

        let outputs = self
            .conn
            .get_outputs()
            .map_err(|e| backend_call_err!(SetPosition, SwayIPC, e))?;

        let output = outputs
            .iter()
            .find(|o| o.name == output_name)
            .ok_or(super::err::Enable::NoOutput(output_name.to_string()))?;
        let rel_output = outputs
            .iter()
            .find(|o| &o.name == rel_output)
            .ok_or(super::err::Enable::NoOutput(rel_output.to_string()))?;

        let (w, h) = (output.rect.width, output.rect.height);
        let (rel_x, rel_y) = (rel_output.rect.x, rel_output.rect.y);
        let (rel_w, rel_h) = (rel_output.rect.width, rel_output.rect.height);

        let mut new_output = output.clone();
        (new_output.rect.x, new_output.rect.y) = match relation {
            Relation::LeftOf => (rel_x - w, rel_y),
            Relation::RightOf => (rel_x + rel_w, rel_y),
            Relation::Above => (rel_x, rel_y - h),
            Relation::Below => (rel_x, rel_y + rel_h),
            Relation::SameAs => (rel_x, rel_y),
        };

        // New iterator of outputs based on the old and the new output
        let new_outputs: Vec<&swayipc::Output> = outputs
            .iter()
            .filter(|o| o.name != new_output.name)
            .chain(std::iter::once(&new_output))
            .collect();

        // Always position the immediately affected output
        let normalized_outputs = normalize_all_outputs(&new_outputs);

        let cmds: Vec<String> = outputs
            .iter()
            .zip(normalized_outputs.iter())
            .filter(|(old, new)| old.rect != new.rect)
            .map(|(_, new)| {
                format!("output {} pos {} {}", new.name, new.rect.x, new.rect.y)
            })
            .collect();

        // All outputs are already in the correct position
        if cmds.is_empty() {
            return Ok(());
        }

        let err_f = |e| backend_call_err!(SetPosition, SwayIPC, e);
        let cmd = itertools::Itertools::join(&mut cmds.iter(), ";");

        run_sway_cmd(&mut self.conn, cmd, err_f)
    }

    fn set_primary(&mut self, _output_name: &str) -> Result<(), BackendError> {
        unimplemented!("Not supported in swayipc");
    }

    fn enable(&mut self, output_name: &str) -> Result<(), BackendError> {
        let outputs = self
            .conn
            .get_outputs()
            .map_err(|e| backend_call_err!(Enable, SwayIPC, e))?;
        let output = outputs
            .iter()
            .find(|o| o.name == output_name)
            .ok_or(super::err::Enable::NoOutput(output_name.to_string()))?;

        let err_f = |e| backend_call_err!(Enable, SwayIPC, e);
        let cmd = format!("output {} enable", output.name);

        run_sway_cmd(&mut self.conn, cmd, err_f)
    }

    fn disable(&mut self, output_name: &str) -> Result<(), BackendError> {
        let outputs = self
            .conn
            .get_outputs()
            .map_err(|e| backend_call_err!(Disable, SwayIPC, e))?;
        let output = outputs
            .iter()
            .find(|o| o.name == output_name)
            .ok_or(super::err::Disable::NoOutput(output_name.to_string()))?;

        let err_f = |e| backend_call_err!(Disable, SwayIPC, e);
        let cmd = format!("output {} disable", output.name);

        run_sway_cmd(&mut self.conn, cmd, err_f)
    }
}
