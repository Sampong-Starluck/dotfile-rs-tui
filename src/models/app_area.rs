use ratatui::layout::Rect;

pub struct AppArea {
    // pub header: Rect,
    pub tabs: Rect,
    pub sidebar: Rect,
    pub body: Rect,
    pub status: Rect
}

impl AppArea {
    /// Returns the full content area spanning sidebar + body combined.
    /// Use this for features that don't need a sidebar split.
    pub fn full_content(&self) -> Rect {
        Rect {
            x:      self.sidebar.x,
            y:      self.sidebar.y,
            width:  self.sidebar.width + self.body.width,
            height: self.sidebar.height,
        }
    }
}