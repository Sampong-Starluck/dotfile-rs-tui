use crate::models::{OperatingSystem, PackageManager, TabModel};

pub struct App {
    pub active_tab:       TabModel,
    pub os:               OperatingSystem,
    pub package_managers: Vec<PackageManager>,
    pub selected_pm:      usize,
    pub command_scroll:   u16,      // ← add this
    pub running:          bool,
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