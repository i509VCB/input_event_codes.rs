//! This module contains all data to be attached to categories.

pub const DATA: &[CategoryData] = &[
    CategoryData {
        name: "Input",
        rename_to: Some("InputQuirk"),
        documentation: None,
    },
    CategoryData {
        name: "Ev",
        rename_to: Some("EventType"),
        documentation: None,
    },
    CategoryData {
        name: "Syn",
        rename_to: Some("SynchronizationEvent"),
        documentation: None,
    },
    CategoryData {
        name: "Key",
        rename_to: None,
        documentation: None,
    },
    CategoryData {
        name: "Btn",
        rename_to: Some("Button"),
        documentation: None,
    },
    CategoryData {
        name: "Rel",
        rename_to: Some("RelativeAxis"),
        documentation: None,
    },
    CategoryData {
        name: "Abs",
        rename_to: Some("AbsoluteAxis"),
        documentation: None,
    },
    CategoryData {
        name: "Sw",
        rename_to: Some("SwitchEvent"),
        documentation: None,
    },
    CategoryData {
        name: "Msc",
        rename_to: Some("MiscEvent"),
        documentation: None,
    },
    CategoryData {
        name: "Rep",
        rename_to: Some("AutoRepeat"),
        documentation: None,
    },
    CategoryData {
        name: "Snd",
        rename_to: Some("Sound"),
        documentation: None,
    },
];

/// Describes extra data that should be attached to a category's constants.
pub struct CategoryData {
    /// The generated name of the category.
    pub name: &'static str,

    /// The changed category name.
    pub rename_to: Option<&'static str>,

    /// The documentation to associate with the category.
    ///
    /// This is hard to capture and does not present all the info we would like.
    pub documentation: Option<&'static str>,
}
