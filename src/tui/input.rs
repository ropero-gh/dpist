use crossterm::event::KeyCode;

use crate::techniques::{DelayConfig, ModifierConfig};
use crate::tui::app::{App, Focus, MenuItem, save};

pub enum DispatchResult {
    Continue,
    Exit,
}

pub fn handle_input(app: &mut App, key: KeyCode) -> DispatchResult {
    if app.focus == Focus::Editing {
        handle_editing(app, key);
        return DispatchResult::Continue;
    }

    match key {
        KeyCode::Char('q') => return DispatchResult::Exit,
        KeyCode::Char('s') => {
            save(app);
            return DispatchResult::Continue;
        }
        _ => {}
    }

    match app.focus {
        Focus::Menu => handle_menu(app, key),
        Focus::Config => handle_config(app, key),
        Focus::Editing => {}
    }

    DispatchResult::Continue
}

fn handle_menu(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Char('j') | KeyCode::Down => {
            app.selected_menu = app.selected_menu.next(&app.modifier_ui);
            app.selected_field = 0;
        }

        KeyCode::Char('k') | KeyCode::Up => {
            app.selected_menu = app.selected_menu.prev(&app.modifier_ui);
            app.selected_field = 0;
        }

        KeyCode::Char('l') | KeyCode::Right => {
            app.focus = Focus::Config;
            app.selected_field = 0;
        }

        KeyCode::Tab => toggle_modifier(app),

        _ => {}
    }
}

fn toggle_modifier(app: &mut App) {
    if let MenuItem::Modifier(idx) = app.selected_menu {
        if let Some(entry) = app.modifier_ui.get_mut(idx) {
            entry.enabled = !entry.enabled;
        }
    }
}

fn field_count(app: &App) -> usize {
    match app.selected_menu {
        MenuItem::General => 2,

        MenuItem::Modifier(idx) => {
            let Some(entry) = app.modifier_ui.get(idx) else {
                return 0;
            };

            match &entry.config {
                ModifierConfig::DropEveryNth { .. } => 1,

                ModifierConfig::TcpSegmentation { .. } => 1,
                ModifierConfig::TlsClientHelloFragmentation { .. } => 1,

                ModifierConfig::Delay(cfg) => match cfg {
                    DelayConfig::Fixed { .. } => 1,
                    DelayConfig::PacketPacing { .. } => 1,
                    DelayConfig::FlowRateLimit { .. } => 1,
                    DelayConfig::Jitter { .. } => 2,
                    DelayConfig::Burst { .. } => 2,
                },
                ModifierConfig::TcpOutOfOrder { .. } => 1,
                ModifierConfig::HttpHeaderFragmentation { .. } => 1,
            }
        }
    }
}

fn handle_config(app: &mut App, key: KeyCode) {
    let max_fields = field_count(app);

    match key {
        KeyCode::Char('h') | KeyCode::Left => {
            app.focus = Focus::Menu;
            return;
        }

        KeyCode::Char('j') | KeyCode::Down => {
            if max_fields == 0 {
                if let MenuItem::Modifier(_) = app.selected_menu {
                    app.selected_field = 0;
                }
                return;
            }

            app.selected_field = (app.selected_field + 1).min(max_fields.saturating_sub(1));
        }

        KeyCode::Char('k') | KeyCode::Up => {
            app.selected_field = app.selected_field.saturating_sub(1);
        }

        KeyCode::Tab => toggle_modifier(app),

        KeyCode::Enter => start_edit(app),

        _ => {}
    }
}

fn start_edit(app: &mut App) {
    app.focus = Focus::Editing;

    app.edit_buffer = match app.selected_menu {
        MenuItem::General => match app.selected_field {
            0 => app.config.input.display().to_string(),
            1 => app.config.output.display().to_string(),
            _ => String::new(),
        },

        MenuItem::Modifier(idx) => {
            let entry = &app.modifier_ui[idx];

            match &entry.config {
                ModifierConfig::DropEveryNth { n } => n.to_string(),

                ModifierConfig::TcpSegmentation { segment_size } => segment_size.to_string(),

                ModifierConfig::TlsClientHelloFragmentation { fragment_size } => {
                    fragment_size.to_string()
                }

                ModifierConfig::TcpOutOfOrder { window } => window.to_string(),
                ModifierConfig::HttpHeaderFragmentation { fragment_size } => {
                    fragment_size.to_string()
                }

                ModifierConfig::Delay(cfg) => match (cfg, app.selected_field) {
                    (DelayConfig::Fixed { millis }, 0) => millis.to_string(),

                    (DelayConfig::PacketPacing { millis }, 0) => millis.to_string(),

                    (DelayConfig::FlowRateLimit { bytes_per_second }, 0) => {
                        bytes_per_second.to_string()
                    }

                    (DelayConfig::Jitter { min_ms, max_ms: _ }, 0) => min_ms.to_string(),
                    (DelayConfig::Jitter { min_ms: _, max_ms }, 1) => max_ms.to_string(),

                    (
                        DelayConfig::Burst {
                            active_ms,
                            pause_ms: _,
                        },
                        0,
                    ) => active_ms.to_string(),

                    (
                        DelayConfig::Burst {
                            active_ms: _,
                            pause_ms,
                        },
                        1,
                    ) => pause_ms.to_string(),

                    _ => String::new(),
                },
            }
        }
    };
}

fn handle_editing(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Esc => {
            app.edit_buffer.clear();
            app.focus = Focus::Config;
        }

        KeyCode::Enter => {
            commit_edit(app);
            app.focus = Focus::Config;
        }

        KeyCode::Backspace => {
            app.edit_buffer.pop();
        }

        KeyCode::Char(c) => {
            app.edit_buffer.push(c);
        }

        _ => {}
    }
}

fn commit_edit(app: &mut App) {
    match app.selected_menu {
        MenuItem::General => match app.selected_field {
            0 => app.config.input = app.edit_buffer.clone().into(),
            1 => app.config.output = app.edit_buffer.clone().into(),
            _ => {}
        },

        MenuItem::Modifier(idx) => {
            let entry = &mut app.modifier_ui[idx];

            match &mut entry.config {
                ModifierConfig::DropEveryNth { n } => {
                    if let Ok(v) = app.edit_buffer.parse::<u64>() {
                        *n = v;
                    }
                }

                ModifierConfig::TcpSegmentation { segment_size } => {
                    if let Ok(v) = app.edit_buffer.parse::<usize>() {
                        *segment_size = v;
                    }
                }

                ModifierConfig::TlsClientHelloFragmentation { fragment_size } => {
                    if let Ok(v) = app.edit_buffer.parse::<usize>() {
                        *fragment_size = v;
                    }
                }

                ModifierConfig::TcpOutOfOrder { window } => {
                    if let Ok(v) = app.edit_buffer.parse::<usize>() {
                        *window = v;
                    }
                }

                ModifierConfig::HttpHeaderFragmentation { fragment_size } => {
                    if let Ok(v) = app.edit_buffer.parse::<usize>() {
                        *fragment_size = v;
                    }
                }

                ModifierConfig::Delay(cfg) => match cfg {
                    DelayConfig::Fixed { millis } => {
                        if let Ok(v) = app.edit_buffer.parse::<u64>() {
                            *millis = v;
                        }
                    }

                    DelayConfig::PacketPacing { millis } => {
                        if let Ok(v) = app.edit_buffer.parse::<u64>() {
                            *millis = v;
                        }
                    }

                    DelayConfig::FlowRateLimit { bytes_per_second } => {
                        if let Ok(v) = app.edit_buffer.parse::<u64>() {
                            *bytes_per_second = v;
                        }
                    }

                    DelayConfig::Jitter { min_ms, max_ms: _ } => {
                        if let Ok(v) = app.edit_buffer.parse::<u64>() {
                            *min_ms = v;
                        }
                    }

                    DelayConfig::Burst {
                        active_ms,
                        pause_ms: _,
                    } => {
                        if let Ok(v) = app.edit_buffer.parse::<u64>() {
                            *active_ms = v;
                        }
                    }
                },
            }
        }
    }

    app.edit_buffer.clear();
}
