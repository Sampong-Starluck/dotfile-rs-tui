#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TabModel {
    Home, Application, Shell
}

impl TabModel {
    pub const ALL: &'static [TabModel] = &[
        TabModel::Home,
        TabModel::Application,
        TabModel::Shell
    ];

    pub fn title(&self) -> &'static str {
        match self {
            TabModel::Home => "Home",
            TabModel::Application => "Applications",
            TabModel::Shell => "Shell"
        }
    }

    pub fn next(self) -> Self {
        let idx = Self::ALL.iter().position(|t| *t == self).unwrap_or(0);
        Self::ALL[(idx + 1) % Self::ALL.len()]
    }

    pub fn prev(self) -> Self {
        let idx = Self::ALL.iter().position(|t| *t == self).unwrap_or(0);
        Self::ALL[(idx + Self::ALL.len() - 1) % Self::ALL.len()]
    }
}