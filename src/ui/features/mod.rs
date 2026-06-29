mod home_tab;
mod app_tab;
mod script_tab;

pub use app_tab::{app_render, handle_key as app_handle_key};
pub use home_tab::{home_render, handle_key as home_handle_key};
pub use script_tab::{script_render, handle_key as script_handle_key};