use std::{
    collections::HashSet,
    sync::mpsc::Receiver
};
use std::sync::mpsc::Sender;
use crate::{
    enumerate::AppFocus,
    models::{Apps, OperatingSystem, PackageManager, TabModel}
};
use crate::ui::features::SearchResult;

pub struct App {
    pub active_tab:       TabModel,
    pub os:               OperatingSystem,
    pub package_managers: Vec<PackageManager>,
    pub selected_pm:      usize,
    pub command_scroll:   u16,
    pub running:          bool,

    // app tab
    pub apps:                 Option<Apps>,
    pub app_selected_section: usize,
    pub app_selected_app:     usize,
    pub app_focus:            AppFocus,
    pub app_selected_ids:     HashSet<String>,
    pub app_custom_input:     String,
    pub app_install_log:      Vec<String>,
    pub app_installing:       bool,
    pub install_rx:           Option<Receiver<String>>,
    pub pkg_mgr:              String,
    pub app_sudo_pending:     bool,
    pub app_sudo_command:     Vec<String>,
    pub app_sudo_password:    String,
    pub install_tx:           Option<Sender<String>>,
    pub install_input:        String,

    // winget search
    pub search_query:    String,
    pub search_results:  Vec<SearchResult>,
    pub search_selected: usize,
    pub search_loading:  bool,
    pub search_rx:       Option<Receiver<Vec<SearchResult>>>,
}

impl App {
    pub fn new() -> Self {
        let os = OperatingSystem::detect();
        let package_managers = PackageManager::detect();
        Self {
            active_tab:           TabModel::Home,
            os,
            package_managers,
            selected_pm:          0,
            command_scroll:       0,
            running:              true,
            apps:                 None,
            app_selected_section: 0,
            app_selected_app:     0,
            app_focus:            AppFocus::Section,
            app_selected_ids:     HashSet::new(),
            app_custom_input:     String::new(),
            app_install_log:      Vec::new(),
            app_installing:       false,
            install_rx:           None,
            pkg_mgr:              String::new(),
            app_sudo_pending:     false,
            app_sudo_command:     Vec::new(),
            app_sudo_password:    String::new(),
            install_tx:           None,
            install_input:        String::new(),
            // winget search
            search_query:         String::new(),
            search_results:       Vec::new(),
            search_selected:      0,
            search_loading:       false,
            search_rx:            None,
        }
    }

    pub fn active_package_manager(&self) -> &str {
        self.package_managers
            .get(self.selected_pm)
            .map(|pm| pm.binary())
            .unwrap_or("unknown")
    }

    pub fn set_package_manager(&mut self, mgr: String) {
        if let Some(idx) = self.package_managers
            .iter()
            .position(|pm| pm.binary() == mgr.as_str())
        {
            self.selected_pm = idx;
        }
        self.apps                 = None;
        self.app_selected_section = 0;
        self.app_selected_app     = 0;
    }

    pub fn cycle_package_manager(&mut self) {
        let candidates: Vec<&str> = if cfg!(target_os = "windows") {
            vec!["winget", "scoop", "choco"]
        } else if cfg!(target_os = "macos") {
            vec!["brew"]
        } else {
            vec!["pacman", "apt", "dnf", "xbps-install"]
        };

        let current = candidates
            .iter()
            .position(|m| *m == self.pkg_mgr.as_str())
            .unwrap_or(0);

        let next = candidates
            .iter()
            .cycle()
            .skip(current + 1)
            .take(candidates.len())
            .find(|mgr| {
                std::process::Command::new(*mgr)
                    .arg("--version")
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .status()
                    .is_ok()
            })
            .map(|m| m.to_string());

        if let Some(mgr) = next {
            self.set_package_manager(mgr);
        }
    }

    pub fn scroll_down(&mut self) {
        self.command_scroll = self.command_scroll.saturating_add(1);
    }

    pub fn scroll_up(&mut self) {
        self.command_scroll = self.command_scroll.saturating_sub(1);
    }

    pub fn reset_scroll(&mut self) {
        self.command_scroll = 0;
    }

    pub fn selected_manager(&self) -> Option<&PackageManager> {
        self.package_managers.get(self.selected_pm)
    }

    pub fn next_pkg(&mut self) {
        if self.package_managers.is_empty() { return; }
        self.selected_pm = (self.selected_pm + 1) % self.package_managers.len();
        self.reset_scroll();
    }

    pub fn previous_pkg(&mut self) {
        if self.package_managers.is_empty() { return; }
        if self.selected_pm == 0 {
            self.selected_pm = self.package_managers.len() - 1;
        } else {
            self.selected_pm -= 1;
        }
        self.reset_scroll();
    }
}