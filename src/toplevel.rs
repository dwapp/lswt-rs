use serde::Serialize;

#[derive(Debug, Clone, Default, Serialize)]
pub struct ToplevelState {
    pub fullscreen: bool,
    pub activated: bool,
    pub maximized: bool,
    pub minimized: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct Toplevel {
    pub id: usize,
    pub title: Option<String>,
    pub app_id: Option<String>,
    pub identifier: Option<String>,
    pub state: ToplevelState,
    #[serde(skip)]
    pub listed: bool,
}

impl Toplevel {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            title: None,
            app_id: None,
            identifier: None,
            state: ToplevelState::default(),
            listed: false,
        }
    }

    pub fn set_title(&mut self, title: String) {
        self.title = Some(title);
    }

    pub fn set_app_id(&mut self, app_id: String) {
        self.app_id = Some(app_id);
    }

    pub fn set_identifier(&mut self, identifier: String) {
        self.identifier = Some(identifier);
    }

    pub fn set_state(&mut self, state: ToplevelState) {
        self.state = state;
    }

    pub fn mark_listed(&mut self) {
        self.listed = true;
    }

    pub fn title_str(&self) -> &str {
        self.title.as_deref().unwrap_or("<NULL>")
    }

    pub fn app_id_str(&self) -> &str {
        self.app_id.as_deref().unwrap_or("<NULL>")
    }

    pub fn identifier_str(&self) -> &str {
        self.identifier.as_deref().unwrap_or("<NULL>")
    }
}
