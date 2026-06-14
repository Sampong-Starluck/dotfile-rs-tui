#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppFocus {
    Section,
    Apps,
    CustomInput,
    /// Live search panel (replaces the apps list body)
    Search,
    /// Installed packages list (replaces the apps list body)
    Installed,
    Installing,
    SudoConfirm,
    PmPicker,
}