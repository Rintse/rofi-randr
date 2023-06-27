use crate::action::{
    Operation, 
    position::Relation, 
    rotate::Rotation };

#[derive(Debug, Default)]
pub enum Icon {
    Connected, Disabled, Disconnected,

    Primary,
    Disable,
    Rotate, Upright, RotLeft, RotRight, Flipped,
    Rate,
    Mode, Fitsize,
    Position, Left, Right, Above, Below, Duplicate,

    Apply, Cancel,
    #[default] None,
}

impl Icon {
    pub fn name(&self) -> String {
        match self {
            Self::Connected     => "desktopconnected",
            Self::Disabled      => "desktoptrusted",
            Self::Disconnected  => "desktopdisconnected",

            Self::Primary   => "video-single-display-symbolic",
            Self::Disable   => "error",
            Self::Rate      => "backup",
            
            // Rotation related
            Self::Rotate    => "rotation-allowed-symbolic",
            Self::Upright   => "draw-triangle3",
            Self::RotLeft   => "draw-triangle1",
            Self::RotRight  => "draw-triangle2",
            Self::Flipped   => "draw-triangle4",

            // Mode related
            Self::Mode      => "node-transform",
            Self::Fitsize   => "fitsize",

            // Positioning related
            Self::Position  => "fitbest",
            Self::Left      => "gtk-goto-first-ltr",
            Self::Right     => "gtk-goto-first-rtl",
            Self::Above     => "gtk-goto-top",
            Self::Below     => "gtk-goto-bottom",
            Self::Duplicate => "video-joined-displays-symbolic",

            // Confirmation
            Self::Apply     => "dialog-apply",
            Self::Cancel    => "dialog-cancel",
            Self::None          => return String::new(),
        }.to_string()
    }

}

impl From<Relation> for Icon {
    fn from(dir : Relation) -> Self {
        match dir {
            Relation::SameAs   => Icon::Duplicate,
            Relation::LeftOf   => Icon::Left,
            Relation::RightOf  => Icon::Right,
            Relation::Above    => Icon::Above,
            Relation::Below    => Icon::Below,
        }
    }
}

impl From<Rotation> for Icon {
    fn from(rot : Rotation) -> Self {
        match rot {
            Rotation::Normal      => Icon::Upright,
            Rotation::Left        => Icon::RotLeft,
            Rotation::Right       => Icon::RotRight,
            Rotation::Inverted    => Icon::Flipped,
        }
    }
}

impl From<Operation> for Icon {
    fn from(op : Operation) -> Self {
        match op {
            Operation::Enable           => Icon::Connected,
            Operation::Disable          => Icon::Disable,
            Operation::SetPrimary       => Icon::Primary,
            Operation::ChangeRes(_)     => Icon::Mode,
            Operation::Position(_)      => Icon::Position,
            Operation::ChangeRate(..)   => Icon::Rate,
            Operation::Rotate(_)        => Icon::Rotate,
        }
    }
}
