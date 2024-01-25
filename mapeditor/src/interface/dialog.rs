pub enum DialogType {
    TypeInformation,
    TypeWarning,
    TypeConfirmation,
    TypeInput,
}

pub struct Dialog {
    is_open: bool,
    dialog_type: DialogType,
}