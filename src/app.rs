use std::{
    collections::HashSet,
    sync::mpsc::Receiver
};
use std::sync::mpsc::Sender;
use crate::{
    enumerate::AppFocus,
    models::{Apps, OperatingSystem, PackageManager, TabModel}
};

pub struct App {
    pub active_tab:       TabModel,
    pub os:               OperatingSystem,
    pub package_managers: Vec<PackageManager>,
    pub selected_pm:      usize,
    pub command_scroll:   u16,      // ← add this
    pub running:          bool,

    // app tab
    pub apps: Option<Apps>,
    pub app_selected_section: usize,      // which section is highlighted in sidebar
    pub app_selected_app: usize,          // which app is highlighted in body
    pub app_focus: AppFocus,
    pub app_selected_ids: HashSet<String>,   // selected app ids
    pub app_custom_input: String,            // custom input buffer
    pub app_install_log: Vec<String>,        // install output lines
    pub app_installing: bool,
    pub install_rx: Option<Receiver<String>>, // receives log lines from install thread
    pub pkg_mgr:      String,   // active pkg manager for install + filtering
    pub app_sudo_pending: bool,        // waiting for sudo confirmation
    pub app_sudo_command: Vec<String>, // commands queued pending confirmation
    pub app_sudo_password: String,  // sudo password input buffer
    pub install_tx:       Option<Sender<String>>,   // app → process stdin
    pub install_input:    String,                   // current input buffer
}

impl App {
    pub fn new() -> Self {
        let os = OperatingSystem::detect();
        let package_managers = PackageManager::detect();
        Self {
            active_tab: TabModel::Home,
            os,
            package_managers,
            selected_pm: 0,
            command_scroll: 0,   // ← add this
            running: true,
            apps: None,
            app_selected_section: 0,
            app_selected_app: 0,
            app_focus: AppFocus::Section,
            app_selected_ids: HashSet::new(),
            app_custom_input: String::new(),
            app_install_log: Vec::new(),
            app_installing: false,
            install_rx: None,
            pkg_mgr: String::new(),
            app_sudo_pending: false,
            app_sudo_command: Vec::new(),
            app_sudo_password: String::new(),
            install_tx:    None,
            install_input: String::new()
        }
    }
    /// Cycle to the next available package manager and invalidate app cache
    // Derives active package manager name from selected_pm index
    pub fn active_package_manager(&self) -> &str {
        self.package_managers
            .get(self.selected_pm)
            .map(|pm| pm.binary())
            .unwrap_or("unknown")
    }

    pub fn set_package_manager(&mut self, mgr: String) {
        // Find the index of the matching package manager
        if let Some(idx) = self.package_managers
            .iter()
            .position(|pm| pm.binary() == mgr.as_str())
        {
            self.selected_pm = idx;
        }
        self.apps                 = None; // invalidate cache
        self.app_selected_section = 0;
        self.app_selected_app     = 0;
    }

    /// Derives the preferred manager from the already-detected package_managers vec
    /// so we don't re-probe on every call
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

        // Find next available one in the list
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
        // saturating_sub prevents underflow below 0
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
            self.selected_pm = self.package_managers.len() - 1; // wrap to last
        } else {
            self.selected_pm -= 1;
        }
        self.reset_scroll();
    }
}