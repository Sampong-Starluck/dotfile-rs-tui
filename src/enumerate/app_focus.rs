#[derive(PartialEq, Clone)]
pub enum AppFocus {
    Section,
    Apps,
    CustomInput,
    Installing,
    SudoConfirm,  // ← add this
}