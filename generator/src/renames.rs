//! This module contains all data regarding renames of input code categories and documentation to attach
//! to each category.

pub const RENAMES: &[CategoryRename] = &[
    CategoryRename {
        name: "Input",
        rename_to: Some("InputQuirk"),
        documentation: None,
    },
    CategoryRename {
        name: "Ev",
        rename_to: Some("EventType"),
        documentation: None,
    },
    CategoryRename {
        name: "Syn",
        rename_to: Some("SynchronizationEvent"),
        documentation: None,
    },
    CategoryRename {
        name: "Key",
        rename_to: None,
        documentation: None,
    },
    CategoryRename {
        name: "Btn",
        rename_to: Some("Button"),
        documentation: None,
    },
    CategoryRename {
        name: "Rel",
        rename_to: Some("RelativeAxis"),
        documentation: None,
    },
    CategoryRename {
        name: "Abs",
        rename_to: Some("AbsoluteAxis"),
        documentation: None,
    },
    CategoryRename {
        name: "Sw",
        rename_to: Some("SwitchEvent"),
        documentation: None,
    },
    CategoryRename {
        name: "Msc",
        rename_to: Some("MiscEvent"),
        documentation: None,
    },
    CategoryRename {
        name: "Rep",
        rename_to: Some("AutoRepeat"),
        documentation: None,
    },
    CategoryRename {
        name: "Snd",
        rename_to: Some("Sound"),
        documentation: None,
    },
];

/// Describes extra data that should be attached to a category's constants.
pub struct CategoryRename {
    /// The generated name of the category.
    pub name: &'static str,

    /// The changed category name.
    pub rename_to: Option<&'static str>,

    /// The documentation to associate with the category.
    ///
    /// This is hard to capture and does not present all the info we would like.
    pub documentation: Option<&'static str>,
}
