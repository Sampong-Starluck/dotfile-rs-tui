mod tab_model;
mod app_area;
mod os;
mod package_manager;
mod apps;
mod shell;

pub use app_area::AppArea;
pub use os::OperatingSystem;
pub use package_manager::PackageManager;
pub use tab_model::TabModel;
pub use apps::*;
pub use shell::{ShellConfig, ShellEntry};