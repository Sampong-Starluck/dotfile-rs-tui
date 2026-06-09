#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppFocus {
    Section,
    Apps,
    CustomInput,
    /// Winget live search panel (replaces the apps list body)
    Search,
    Installing,
    SudoConfirm,
    PmPicker,   // ← add
}