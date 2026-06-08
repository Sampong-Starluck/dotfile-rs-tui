mod home_tab;
mod app_tab;
mod shell_tab;

pub use app_tab::{app_render, handle_key as app_handle_key, SearchResult};
pub use home_tab::{home_render, handle_key as home_handle_key};
pub use shell_tab::{shell_render, handle_key as shell_handle_key};