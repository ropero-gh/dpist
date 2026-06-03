use ratatui::{prelude::*, widgets::*};

use crate::{
    techniques::{DelayConfig, ModifierConfig},
    tui::{
        app::{App, Focus, MenuItem},
        input::{DispatchResult, handle_input},
        modifier_name,
    },
};

pub struct Field {
    pub name: &'static str,
    pub value: String,
}

pub fn run_tui(app: &mut App) -> anyhow::Result<()> {
    use crossterm::{
        event::{self, Event},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode},
    };
    use std::io;

    enable_raw_mode()?;

    let mut stdout = io::stdout();
    execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;

    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal = ratatui::Terminal::new(backend)?;

    loop {
        terminal.draw(|f| draw(f, app))?;

        if let Event::Key(key) = event::read()?
            && let DispatchResult::Exit = handle_input(app, key.code)
        {
            break;
        }
    }

    disable_raw_mode()?;

    execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen
    )?;

    Ok(())
}

fn draw(f: &mut Frame, app: &App) {
    let layout = Layout::vertical([Constraint::Min(1), Constraint::Length(3)]).split(f.area());

    let body = Layout::horizontal([Constraint::Length(38), Constraint::Min(1)]).split(layout[0]);

    draw_menu(f, app, body[0]);
    draw_config(f, app, body[1]);
    draw_status(f, app, layout[1]);
}

fn draw_menu(f: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = std::iter::once(ListItem::new("General"))
        .chain(app.modifier_ui.iter().map(|entry| {
            let name = modifier_name(&entry.config);
            let state = if entry.enabled { "✓" } else { " " };
            ListItem::new(format!("{state} {name}"))
        }))
        .collect();

    let selected_index = match app.selected_menu {
        MenuItem::General => 0,
        MenuItem::Modifier(i) => i + 1,
    };

    let mut state = ListState::default();
    state.select(Some(selected_index));

    let block_style = if matches!(app.focus, Focus::Menu) {
        Style::default().fg(Color::Green)
    } else {
        Style::default()
    };

    let list = List::new(items)
        .block(Block::bordered().title("Menu").border_style(block_style))
        .highlight_symbol(" ▶ ")
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    f.render_stateful_widget(list, area, &mut state);
}

fn draw_config(f: &mut Frame, app: &App, area: Rect) {
    let fields: Vec<(&str, String, bool)> = match app.selected_menu {
        MenuItem::General => vec![
            ("Input", app.config.input.display().to_string(), true),
            ("Output", app.config.output.display().to_string(), true),
        ],

        MenuItem::Modifier(idx) => {
            let entry = &app.modifier_ui[idx];

            match &entry.config {
                ModifierConfig::DropEveryNth { n } => {
                    vec![("N", n.to_string(), entry.enabled)]
                }

                ModifierConfig::TcpSegmentation { segment_size } => {
                    vec![("Segment", segment_size.to_string(), entry.enabled)]
                }

                ModifierConfig::TlsClientHelloFragmentation { fragment_size } => {
                    vec![("Fragment", fragment_size.to_string(), entry.enabled)]
                }

                ModifierConfig::TcpOutOfOrder { window } => {
                    vec![("Window", window.to_string(), entry.enabled)]
                }

                ModifierConfig::HttpHeaderFragmentation { fragment_size } => {
                    vec![("Fragment", fragment_size.to_string(), entry.enabled)]
                }

                ModifierConfig::Delay(DelayConfig::Fixed { millis }) => {
                    vec![("Millis", millis.to_string(), entry.enabled)]
                }

                ModifierConfig::Delay(DelayConfig::PacketPacing { millis }) => {
                    vec![("Millis", millis.to_string(), entry.enabled)]
                }

                ModifierConfig::Delay(DelayConfig::FlowRateLimit { bytes_per_second }) => {
                    vec![("Bytes/s", bytes_per_second.to_string(), entry.enabled)]
                }

                ModifierConfig::Delay(DelayConfig::Jitter { min_ms, max_ms }) => vec![
                    ("Min ms", min_ms.to_string(), entry.enabled),
                    ("Max ms", max_ms.to_string(), entry.enabled),
                ],

                ModifierConfig::Delay(DelayConfig::Burst {
                    active_ms,
                    pause_ms,
                }) => vec![
                    ("Active ms", active_ms.to_string(), entry.enabled),
                    ("Pause ms", pause_ms.to_string(), entry.enabled),
                ],
            }
        }
    };

    let items: Vec<ListItem> = fields
        .iter()
        .enumerate()
        .map(|(i, (name, value, active))| {
            let is_selected = app.selected_field == i && matches!(app.focus, Focus::Config);

            let is_editing = app.selected_field == i && matches!(app.focus, Focus::Editing);

            let display_value = if is_editing {
                app.edit_buffer.as_str()
            } else {
                value.as_str()
            };

            let mut style = if *active {
                Style::default()
            } else {
                Style::default().fg(Color::DarkGray)
            };

            if is_selected || is_editing {
                style = style.fg(Color::Yellow).add_modifier(Modifier::BOLD);
            }

            let mut spans = vec![
                Span::styled(format!("{name} : "), Style::default().fg(Color::Gray)),
                Span::styled(display_value, style),
            ];

            if is_editing {
                spans.push(Span::styled("█", Style::default().fg(Color::Yellow)));
            }

            ListItem::new(Line::from(spans))
        })
        .collect();

    let mut state = ListState::default();
    state.select(Some(app.selected_field));

    let block_style = if matches!(app.focus, Focus::Config) {
        Style::default().fg(Color::Green)
    } else {
        Style::default()
    };

    let list = List::new(items)
        .block(Block::bordered().title("Config").border_style(block_style))
        .highlight_style(Style::default());

    f.render_stateful_widget(list, area, &mut state);
}

fn draw_status(f: &mut Frame, app: &App, area: Rect) {
    let mode = match app.focus {
        Focus::Config => "CONFIG",
        Focus::Menu => "MENU",
        Focus::Editing => "EDITING",
    };

    let help = format!(
        "Mode: {} | q: quit | s: save | hjkl/arrows: navigate | enter: edit | tab: toggle",
        mode
    );

    let paragraph = Paragraph::new(help)
        .wrap(Wrap { trim: true })
        .block(Block::bordered().title("Status"))
        .style(Style::default());

    f.render_widget(paragraph, area);
}
