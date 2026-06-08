#[derive(PartialEq, Clone, Debug)]
pub enum AppFocus {
    Section,
    Apps,
    CustomInput,
    Installing,
    SudoConfirm,  // ← add this
}