use super::{ModeEntry, OutputEntry};
use crate::action::mode::Mode;
use crate::action::position::Position;
use crate::action::position::Relation;
use crate::action::rotate::Rotation;
use crate::action::Operation;
use crate::backend::Error as BackendError;
use crate::backend_call as backend_call_err;
use xrandr::ScreenResources;
use xrandr::XHandle;

pub struct Backend {
    handle: XHandle,
    res: ScreenResources,
}

impl Backend {
    pub fn new() -> Result<Self, BackendError> {
        let mut handle = XHandle::open()
            .map_err(|e| backend_call_err!(GetOutputs, LibXrandr, e))?;
        let res = ScreenResources::new(&mut handle)
            .map_err(|e| backend_call_err!(GetOutputs, LibXrandr, e))?;

        Ok(Self { handle, res })
    }
}

const RATE_EPSILON: f64 = 0.01; // xrandr rates are rounded to 2 decimals

impl super::DisplayBackend for Backend {
    fn supported_operations(&mut self, output: &OutputEntry) -> Vec<Operation> {
        match (output.connected, output.enabled) {
            // If the output is not connected, just give the option
            // to disable/enable it. (X allows you to unplug an output
            // while still having it as active)
            (false, _) => vec![Operation::Disable],

            // If the output is connected but disabled, only show enable option
            (_, false) => vec![Operation::Enable],

            // Otherwise, list all except enable
            _ => vec![
                Operation::Disable,
                Operation::SetPrimary,
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
            Relation::SameAs,
        ]
    }

    fn get_outputs(&mut self) -> Result<Vec<OutputEntry>, BackendError> {
        let outputs = self
            .res
            .outputs(&mut self.handle)
            .map_err(|e| backend_call_err!(GetOutputs, LibXrandr, e))?;

        let entries = outputs
            .iter()
            .map(|o| OutputEntry {
                name: o.name.clone(),
                connected: o.connected,
                enabled: o.current_mode.is_some(),
            })
            .collect();

        Ok(entries)
    }

    fn get_modes(
        &mut self,
        output: &str,
    ) -> Result<Vec<ModeEntry>, BackendError> {
        let outputs = self
            .res
            .outputs(&mut self.handle)
            .map_err(|e| backend_call_err!(GetResolutions, LibXrandr, e))?;

        let output = outputs
            .iter()
            .find(|o| o.name == output)
            .ok_or(super::err::GetResolutions::NoOutput(output.to_string()))?;

        let current_mode_id = output
            .current_mode
            .ok_or(super::err::GetResolutions::GetCurrent)?;

        let current_mode = self
            .res
            .mode(current_mode_id)
            .map_err(|_| super::err::GetResolutions::GetCurrent)?;

        let mut entries = self
            .res
            .modes()
            .iter()
            .filter(|m| output.modes.contains(&m.xid))
            .map(|m| ModeEntry {
                val: Mode {
                    width: m.width,
                    height: m.height,
                    rate: m.rate,
                },
                current: m.width == current_mode.width
                    && m.height == current_mode.height,
            })
            .collect::<Vec<ModeEntry>>();

        entries.sort_by(|a, b| Mode::cmp(&b.val, &a.val));
        entries.dedup();
        Ok(entries)
    }

    fn set_mode(
        &mut self,
        output_name: &str,
        mode: &Mode,
    ) -> Result<(), BackendError> {
        let outputs = self
            .res
            .outputs(&mut self.handle)
            .map_err(|e| backend_call_err!(SetResolution, LibXrandr, e))?;

        let output = outputs.iter().find(|o| o.name == output_name).ok_or(
            super::err::SetResolution::NoOutput(output_name.to_string()),
        )?;

        let target_mode = self
            .res
            .modes
            .iter()
            .filter(|m| output.modes.contains(&m.xid))
            .find(|m| {
                (m.rate - mode.rate).abs() < RATE_EPSILON
                    && m.width == mode.width
                    && m.height == mode.height
            })
            .ok_or(super::err::SetResolution::NoMode(mode.clone()))?;

        self.handle
            .set_mode(output, target_mode)
            .map_err(|e| backend_call_err!(GetResolutions, LibXrandr, e))?;

        Ok(())
    }

    fn set_rotation(
        &mut self,
        output_name: &str,
        rotation: &Rotation,
    ) -> Result<(), BackendError> {
        let outputs = self
            .res
            .outputs(&mut self.handle)
            .map_err(|e| backend_call_err!(SetRotation, LibXrandr, e))?;

        let output = outputs.iter().find(|o| o.name == output_name).ok_or(
            super::err::SetRotation::NoOutput(output_name.to_string()),
        )?;

        self.handle
            .set_rotation(output, &xrandr::Rotation::from(rotation))
            .map_err(|e| backend_call_err!(SetRotation, LibXrandr, e))?;

        Ok(())
    }

    fn set_position(
        &mut self,
        output_name: &str,
        pos: &Position,
    ) -> Result<(), BackendError> {
        let Position {
            output_s: rel_output,
            relation,
            ..
        } = pos;

        let outputs = self
            .res
            .outputs(&mut self.handle)
            .map_err(|e| backend_call_err!(SetPosition, LibXrandr, e))?;

        let output = outputs.iter().find(|o| o.name == output_name).ok_or(
            super::err::SetPosition::NoOutput(output_name.to_string()),
        )?;

        let rel_output = outputs.iter().find(|o| &o.name == rel_output).ok_or(
            super::err::SetPosition::NoOutput(output_name.to_string()),
        )?;

        assert!(output.name != rel_output.name, "UI should prohibit this");

        let xrel = &xrandr::Relation::from(relation);
        self.handle
            .set_position(output, xrel, rel_output)
            .map_err(|e| backend_call_err!(SetPosition, LibXrandr, e))?;

        Ok(())
    }

    fn set_primary(&mut self, output_name: &str) -> Result<(), BackendError> {
        let outputs = self
            .res
            .outputs(&mut self.handle)
            .map_err(|e| backend_call_err!(SetPrimary, LibXrandr, e))?;

        let output = outputs
            .iter()
            .find(|o| o.name == output_name)
            .ok_or(super::err::SetPrimary::NoOutput(output_name.to_string()))?;

        self.handle.set_primary(output);
        Ok(())
    }

    fn enable(&mut self, output_name: &str) -> Result<(), BackendError> {
        let outputs = self
            .res
            .outputs(&mut self.handle)
            .map_err(|e| backend_call_err!(Enable, LibXrandr, e))?;

        let output = outputs
            .iter()
            .find(|o| o.name == output_name)
            .ok_or(super::err::Enable::NoOutput(output_name.to_string()))?;

        self.handle
            .enable(output)
            .map_err(|e| backend_call_err!(Enable, LibXrandr, e))?;

        Ok(())
    }

    fn disable(&mut self, output_name: &str) -> Result<(), BackendError> {
        let outputs = self
            .res
            .outputs(&mut self.handle)
            .map_err(|e| backend_call_err!(Disable, LibXrandr, e))?;

        let output = outputs
            .iter()
            .find(|o| o.name == output_name)
            .ok_or(super::err::Disable::NoOutput(output_name.to_string()))?;

        self.handle
            .disable(output)
            .map_err(|e| backend_call_err!(Disable, LibXrandr, e))?;

        Ok(())
    }
}
