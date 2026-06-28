pub mod app_service;
pub mod install_service;
pub mod search_service;
pub mod system_service;
pub mod script_service;

pub use app_service::{read_apps_json, filter_apps_by_platform};
pub use install_service::{install_command, get_install_command, remove_command};
pub use search_service::{search_command, decode_search_output, parse_search_output, search_hint, list_command, parse_list_output, SearchResult};
pub use system_service::{is_root, requires_sudo, requires_interactive};