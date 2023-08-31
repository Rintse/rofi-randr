use crate::action::Operation;
use crate::action::position::Relation;
use crate::action::position::Position;
use crate::action::rate::Rate;
use crate::action::rotate::Rotation;
use crate::action::resolution::Resolution;
use crate::backend_call as backend_call_err;
use crate::backend::Error as BackendError;
use xrandr::XHandle;
use xrandr::ScreenResources;

use super::{OutputEntry, RateEntry, ResolutionEntry};

pub struct Backend { handle: XHandle, res: ScreenResources }

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

    fn get_outputs(&mut self) -> Result<Vec<OutputEntry>, BackendError> {
        let outputs = self.res.outputs(&mut self.handle)
            .map_err(|e| backend_call_err!(GetOutputs, LibXrandr, e))?;

        let entries = outputs.iter()
            .map(|o| OutputEntry { 
                name: o.name.clone(), 
                connected: o.connected, 
                enabled: o.current_mode.is_some()})
            .collect();

        Ok(entries)
    }
    
    fn get_resolutions(&mut self, output: &str) 
    -> Result<Vec<ResolutionEntry>, BackendError> {
        let outputs = self.res.outputs(&mut self.handle)
            .map_err(|e| backend_call_err!(GetResolutions, LibXrandr, e))?;

        let output = outputs.iter()
            .find(|o| o.name == output)
            .ok_or(super::err::GetResolutions::NoOutput(output.to_string()))?;

        eprintln!("output: {}", output.name);
        eprintln!("modes: {:?}", output.modes);

        let current_mode_id = output.current_mode
            .ok_or(super::err::GetResolutions::GetCurrent)?;

        let current_mode = self.res.mode(current_mode_id)
            .map_err(|_|super::err::GetResolutions::GetCurrent)?;

        let mut entries = self.res.modes().iter()
            .filter(|m| output.modes.contains(&m.xid))
            .map(|m| ResolutionEntry { 
                val: Resolution { width: m.width, height: m.height }, 
                current: m.width == current_mode.width 
                    && m.height == current_mode.height })
            .collect::<Vec<ResolutionEntry>>();

        entries.sort_by( |a,b| u32::cmp( 
            &(b.val.width * b.val.height),
            &(a.val.width * a.val.height)));
        entries.dedup();
        Ok(entries)
    }
    
    fn set_resolution(&mut self, output_name: &str, res: &Resolution) 
    -> Result<(), BackendError> {
        let outputs = self.res.outputs(&mut self.handle)
            .map_err(|e| backend_call_err!(SetResolution, LibXrandr, e))?;

        let output = outputs.iter()
            .find(|o| o.name == output_name)
            .ok_or(super::err::SetResolution::NoOutput(output_name.to_string()))?;

        let target_mode = self.res.modes.iter()
            .filter(|m| output.modes.contains(&m.xid))
            .find(|m| m.width == res.width && m.height == res.height)
            .ok_or(super::err::SetResolution::NoMode(res.clone()))?;

        self.handle.set_mode(output, target_mode)
            .map_err(|e| backend_call_err!(GetResolutions, LibXrandr, e))?;

        Ok(())
    }

    fn get_rates(&mut self, output_name: &str) 
    -> Result<Vec<RateEntry>, BackendError> {
        let outputs = self.res.outputs(&mut self.handle)
            .map_err(|e| backend_call_err!(GetRates, LibXrandr, e))?;

        let output = outputs.iter()
            .find(|o| o.name == output_name)
            .ok_or(super::err::GetRates::NoOutput(output_name.to_string()))?;

        let current_mode_id = output.current_mode
            .ok_or(super::err::GetRates::GetCurrent)?;
        let current_mode = self.res.mode(current_mode_id)
            .map_err(|_|super::err::GetRates::GetCurrent)?;

        let entries = self.res.modes().iter()
            .filter(|m| output.modes.contains(&m.xid))
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
    -> Result<(), BackendError> {
        let outputs = self.res.outputs(&mut self.handle)
            .map_err(|e| backend_call_err!(SetRate, LibXrandr, e))?;
        let output = outputs.iter()
            .find(|o| o.name == output_name)
            .ok_or(super::err::SetRate::NoOutput(output_name.to_string()))?;

        let current_mode_id = output.current_mode
            .ok_or(super::err::SetRate::NoMode(output.name.clone()))?;
        let current_mode = self.res.mode(current_mode_id)
            .map_err(|_|super::err::SetRate::NoMode(output.name.clone()))?;

        let target_mode = self.res.modes.iter()
            .filter(|m| output.modes.contains(&m.xid))
            .find(|m| m.width == current_mode.width 
                && m.height == current_mode.height
                && (m.rate - rate).abs() < RATE_EPSILON)
            .ok_or(super::err::SetRate::NoRate(rate))?;

        self.handle.set_mode(output, target_mode)
            .map_err(|e| backend_call_err!(SetRate, LibXrandr, e))?;

        Ok(())
    }

    fn set_rotation(&mut self, output_name: &str, rotation: &Rotation)
    -> Result<(), BackendError> {
        let outputs = self.res.outputs(&mut self.handle)
            .map_err(|e| backend_call_err!(SetRotation, LibXrandr, e))?;

        let output = outputs.iter()
            .find(|o| o.name == output_name)
            .ok_or(super::err::SetRotation::NoOutput(output_name.to_string()))?;

        self.handle.set_rotation(output, &xrandr::Rotation::from(rotation))
            .map_err(|e| backend_call_err!(SetRotation, LibXrandr, e))?;

        Ok(())
    }
    
    fn set_position(&mut self, output_name: &str, pos: &Position)
    -> Result<(), BackendError> {
        let Position { output_s: rel_output, relation, ..} = pos;
        
        let outputs = self.res.outputs(&mut self.handle)
            .map_err(|e| backend_call_err!(SetPosition, LibXrandr, e))?;

        let output = outputs.iter()
            .find(|o| o.name == output_name)
            .ok_or(super::err::SetPosition::NoOutput(output_name.to_string()))?;

        let rel_output = outputs.iter()
            .find(|o| &o.name == rel_output)
            .ok_or(super::err::SetPosition::NoOutput(output_name.to_string()))?;

        assert!(output.name != rel_output.name, "UI should prohibit this");

        let xrel = &xrandr::Relation::from(relation);
        self.handle.set_position(output, xrel, rel_output)
            .map_err(|e| backend_call_err!(SetPosition, LibXrandr, e))?;

        Ok(())
    }
    
    fn set_primary(&mut self, output_name: &str) -> Result<(), BackendError> {
        let outputs = self.res.outputs(&mut self.handle)
            .map_err(|e| backend_call_err!(SetPrimary, LibXrandr, e))?;

        let output = outputs.iter()
            .find(|o| o.name == output_name)
            .ok_or(super::err::SetPrimary::NoOutput(output_name.to_string()))?;

        self.handle.set_primary(output);
        Ok(())
    }
    
    fn enable(&mut self, output_name: &str) -> Result<(), BackendError> {
        let outputs = self.res.outputs(&mut self.handle)
            .map_err(|e| backend_call_err!(Enable, LibXrandr, e))?;

        let output = outputs.iter()
            .find(|o| o.name == output_name)
            .ok_or(super::err::Enable::NoOutput(output_name.to_string()))?;
        
        self.handle.enable(output)
            .map_err(|e| backend_call_err!(Enable, LibXrandr, e))?;

        Ok(())
    }
    
    fn disable(&mut self, output_name: &str) -> Result<(), BackendError> {
        let outputs = self.res.outputs(&mut self.handle)
            .map_err(|e| backend_call_err!(Disable, LibXrandr, e))?;

        let output = outputs.iter()
            .find(|o| o.name == output_name)
            .ok_or(super::err::Disable::NoOutput(output_name.to_string()))?;
        
        self.handle.disable(output)
            .map_err(|e| backend_call_err!(Disable, LibXrandr, e))?;

        Ok(())
    }
}
