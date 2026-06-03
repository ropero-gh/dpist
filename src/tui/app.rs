use std::path::PathBuf;

use crate::config::{Config, ModifierUi, save_config, validate};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Menu,
    Config,
    Editing,
}

#[derive(Debug, derive_more::Display, Clone, Copy, PartialEq, Eq)]
pub enum MenuItem {
    General,
    Modifier(usize),
}

impl MenuItem {
    pub fn next(self, ui: &Vec<ModifierUi>) -> Self {
        match self {
            MenuItem::General => {
                if ui.is_empty() {
                    MenuItem::General
                } else {
                    MenuItem::Modifier(0)
                }
            }
            MenuItem::Modifier(i) => {
                if i + 1 < ui.len() {
                    MenuItem::Modifier(i + 1)
                } else {
                    MenuItem::General
                }
            }
        }
    }

    pub fn prev(self, ui: &Vec<ModifierUi>) -> Self {
        match self {
            MenuItem::General => {
                if ui.is_empty() {
                    MenuItem::General
                } else {
                    MenuItem::Modifier(ui.len() - 1)
                }
            }
            MenuItem::Modifier(i) => {
                if i == 0 {
                    MenuItem::General
                } else {
                    MenuItem::Modifier(i - 1)
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct App {
    pub config: Config,
    pub modifier_ui: Vec<ModifierUi>,

    pub focus: Focus,
    pub selected_menu: MenuItem,
    pub selected_field: usize,
    pub edit_buffer: String,

    pub status: String,
    pub config_path: PathBuf,
}

impl App {
    pub fn new(config: Config, modifier_ui: Vec<ModifierUi>, path: impl Into<PathBuf>) -> Self {
        Self {
            config,
            modifier_ui,

            focus: Focus::Menu,
            selected_menu: MenuItem::General,
            selected_field: 0,

            edit_buffer: String::new(),
            status: "Ready".into(),

            config_path: path.into(),
        }
    }

    pub fn sync_to_config(&mut self) {
        self.config.modifiers = self
            .modifier_ui
            .iter()
            .filter(|m| m.enabled)
            .map(|m| m.config)
            .collect();
    }

    pub fn current_menu(&self) -> MenuItem {
        self.selected_menu
    }
}

pub fn save(app: &mut App) {
    app.config.modifiers = app
        .modifier_ui
        .iter()
        .filter(|m| m.enabled)
        .map(|m| m.config)
        .collect();

    match validate(&app.config) {
        Ok(()) => match save_config(&app.config_path, &app.config) {
            Ok(()) => {
                app.status = format!("Saved {}", app.config_path.display());
            }
            Err(err) => {
                app.status = format!("Save failed: {err}");
            }
        },
        Err(err) => {
            app.status = format!("Invalid config: {err}");
        }
    }
}
