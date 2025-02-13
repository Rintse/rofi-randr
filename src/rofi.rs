// Defines data structures and methods to construct and
// print lists in the format that rofi understands.
use strum::IntoEnumIterator;

use crate::action::mode::Mode;
use crate::action::{
    position::Relation, rotate::Rotation, Action, Operation, ParseResult,
};
use crate::backend::{DisplayBackend, ModeEntry, OutputEntry};
use crate::err::AppError;
use crate::icon::Icon;

#[derive(Debug, Default)]
pub struct ListItem {
    pub text: String,
    pub comments: Vec<String>,
    pub icon: Option<Icon>,
    pub meta: Option<String>,
    pub non_selectable: bool,
    pub info: Option<String>,
}

impl ListItem {
    pub fn rofi_print(&self) {
        let mut mods: Vec<String> = Vec::new();
        mods.push(format!("nonselectable\x1f{}", self.non_selectable));

        if let Some(icon) = &self.icon {
            mods.push(format!("icon\x1f{}", icon.name()));
        }
        if let Some(meta) = &self.meta {
            mods.push(format!("meta\x1f{meta}"));
        }
        if let Some(info) = &self.info {
            mods.push(format!("info\x1f{info}"));
        }
        let cmt = if self.comments.is_empty() {
            String::new()
        } else {
            let cmt_str = self.comments.join(", ");
            format!(" <span style='italic' size='small'>({cmt_str})</span>")
        };

        println!("{}{}\0{}", self.text, cmt, mods.join("\x1f"),);
    }

    pub fn back() -> Self {
        Self {
            text: "Back".into(),
            comments: vec!["previous menu".into()],
            icon: Some(Icon::Back),
            ..Default::default()
        }
    }
}

// List of options to show next
#[derive(Debug, Default)]
pub struct List {
    pub prompt: Option<String>,
    pub message: Option<String>,
    // Inverted from rofi-script due to more sensible `Default`
    pub allow_custom: bool,
    pub keep_selection: bool,
    pub no_markup: bool,
    pub items: Vec<ListItem>,
    // Do not print a back entry in the list
    pub no_back: bool,
}

impl List {
    pub fn rofi_print(&self) {
        if let Some(prompt) = &self.prompt {
            println!("\0prompt\x1f{prompt}");
        }

        if let Some(msg) = &self.message {
            println!("\0message\x1f{msg}");
        } else {
            // This needs to be reset between lists
            println!("\0message\x1f");
        };

        println!("\0no-custom\x1f{}", !self.allow_custom);
        println!("\0keep-selection\x1f{}", self.keep_selection);
        println!("\0markup-rows\x1f{}", !self.no_markup);

        self.items.iter().for_each(ListItem::rofi_print);
        if !self.no_back {
            ListItem::back().rofi_print();
        }
    }

    pub fn error(msg: &str) -> Self {
        Self {
            prompt: Some("ERROR".into()),
            message: Some(msg.to_string()),
            ..Default::default()
        }
    }
}

// TODO: lots of duplication here
impl From<&OutputEntry> for ListItem {
    fn from(output: &OutputEntry) -> Self {
        let (icon, comments) = match (output.connected, output.enabled) {
            (false, _) => {
                (Icon::Disconnected, vec!["disconnected".to_string()])
            }
            (_, false) => (Icon::Disabled, vec!["disabled".to_string()]),
            _ => (Icon::Connected, Vec::new()),
        };

        ListItem {
            text: output.name.clone(),
            comments,
            icon: Some(icon),
            non_selectable: !output.connected,
            ..Default::default()
        }
    }
}

impl From<Operation> for ListItem {
    fn from(op: Operation) -> Self {
        ListItem {
            text: op.to_string(),
            icon: Some(Icon::from(op)),
            ..Default::default()
        }
    }
}

impl From<Relation> for ListItem {
    fn from(dir: Relation) -> Self {
        ListItem {
            text: dir.to_string(),
            icon: Some(Icon::from(dir)),
            ..Default::default()
        }
    }
}

impl From<Rotation> for ListItem {
    fn from(rot: Rotation) -> Self {
        ListItem {
            text: rot.to_string(),
            comments: vec![rot.explain()],
            icon: Some(Icon::from(rot)),
            ..Default::default()
        }
    }
}

impl From<&ModeEntry> for ListItem {
    fn from(mode_entry: &ModeEntry) -> Self {
        let comments = if mode_entry.current {
            vec!["Current".to_string()]
        } else {
            Vec::new()
        };

        ListItem {
            text: format!(
                "{}x{}@{:.2}Hz",
                mode_entry.val.width,
                mode_entry.val.height,
                mode_entry.val.rate
            ),
            icon: Some(Icon::Fitsize),
            comments,
            ..Default::default()
        }
    }
}

impl ParseResult<Action> {
    // All outputs on the system (enabled+disabled+disconnected)
    pub fn output_list(
        backend: &mut Box<dyn DisplayBackend>,
    ) -> Result<Self, AppError> {
        let mut outputs = backend.get_outputs()?;

        // List connected outputs first
        outputs.sort_by(|a, b| bool::cmp(&b.connected, &a.connected));

        Ok(Self::Next(List {
            prompt: Some("Select output".to_string()),
            items: outputs.iter().map(ListItem::from).collect(),
            no_back: true,
            ..Default::default()
        }))
    }

    // left/right/above/below
    pub fn relation_list(backend: &mut Box<dyn DisplayBackend>) -> Self {
        let list = backend
            .supported_relations()
            .into_iter()
            .map(ListItem::from)
            .collect();

        Self::Next(List {
            prompt: Some("Select position".to_string()),
            items: list,
            ..Default::default()
        })
    }

    // left/right/normal/inverted
    pub fn rotation_list() -> Self {
        Self::Next(List {
            prompt: Some("Select rotation".to_string()),
            items: Rotation::iter().map(ListItem::from).collect(),
            ..Default::default()
        })
    }

    // Confirm menu to avoid accidentally disabling the last display
    pub fn confirm_disable_list() -> Self {
        Self::Next(List {
            prompt: Some("Disable last active output?".to_string()),
            items: vec![ListItem {
                text: "Yes".to_string(),
                icon: Some(Icon::Apply),
                ..Default::default()
            }],
            ..Default::default()
        })
    }

    // Available resolutions for the given output
    pub fn mode_list(
        backend: &mut Box<dyn DisplayBackend>,
        output: &str,
    ) -> Result<Self, AppError> {
        let mut modes = backend.get_modes(output)?;
        modes.sort_by(|a, b| Mode::cmp(&a.val, &b.val));

        Ok(Self::Next(List {
            prompt: Some("Select resolution ".to_string()),
            message: Some(output.to_string()),
            items: modes.iter().map(ListItem::from).collect(),
            ..Default::default()
        }))
    }

    // list_outputs not equal to `output`
    pub fn relatives_list(
        backend: &mut Box<dyn DisplayBackend>,
        output: &str,
        relation: &Relation,
    ) -> Result<Self, AppError> {
        let outputs = backend.get_outputs()?;
        let mut others: Vec<&OutputEntry> =
            outputs.iter().filter(|o| o.name != output).collect();

        // List connected outputs first
        others.sort_by(|a, b| bool::cmp(&b.connected, &a.connected));

        let mut list = others
            .iter()
            .copied()
            .map(ListItem::from)
            .collect::<Vec<ListItem>>();

        // In this menu, you should only be able to select enabled displays
        for (item, output) in list.iter_mut().zip(others.iter()) {
            if !output.enabled {
                item.non_selectable = true;
            }
        }

        Ok(Self::Next(List {
            prompt: Some("Select output".to_string()),
            message: Some(format!("{output} ({relation}...)")),
            items: list,
            ..Default::default()
        }))
    }

    // Enabled displays have all options except enable
    pub fn operation_list(
        backend: &mut Box<dyn DisplayBackend>,
        output: &OutputEntry,
    ) -> Self {
        let supported_ops = backend.supported_operations(output);
        let op_list = supported_ops.into_iter().map(ListItem::from).collect();

        Self::Next(List {
            prompt: Some("Select operation".to_string()),
            message: Some(output.name.clone()),
            items: op_list,
            ..Default::default()
        })
    }
}
