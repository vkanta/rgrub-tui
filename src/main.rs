mod app;
mod model;

use crate::app::{App, View};
use crate::model::parser::GrubParser;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::*, widgets::*};
use std::io;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let settings = GrubParser::parse_settings("/etc/default/grub")?;
    let menu_items = GrubParser::parse_config("/boot/grub/grub.cfg")?;
    let mut app = App::new(settings, menu_items);

    // Terminal setup
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Main event loop
    while !app.should_quit {
        terminal.draw(|f| ui(f, &app))?;

        if event::poll(std::time::Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => app.should_quit = true,
                    KeyCode::Char('s') => app.toggle_view(),
                    KeyCode::Down => {
                        if app.current_view == View::MenuView {
                            app.next_menu_item();
                        } else if app.current_view == View::SettingsView {
                            app.next_settings_item();
                        }
                    }
                    KeyCode::Up => {
                        if app.current_view == View::MenuView {
                            app.prev_menu_item();
                        } else if app.current_view == View::SettingsView {
                            app.prev_settings_item();
                        }
                    }
                    KeyCode::Char('e') => {
                        if app.current_view == View::SettingsView {
                            app.enter_edit_mode();
                        }
                    }
                    KeyCode::Char('S') => {
                        if app.current_view == View::SettingsView {
                            app.request_save();
                        }
                    }
                    KeyCode::Char('a') => {
                        if app.current_view == View::EditingSettings {
                            app.apply_edit();
                        }
                    }
                    KeyCode::Char('c') => {
                        if app.current_view == View::EditingSettings {
                            app.cancel_edit();
                        }
                    }
                    KeyCode::Enter => {
                        if app.current_view == View::EditingSettings {
                             app.apply_edit();
                        } else if app.current_view == View::ConfirmSave {
                             let _ = app.settings.save("/etc/default/grub"); 
                             app.current_view = View::SettingsView;
                        } else {
                             app.select_menu_item();
                        }
                    }
                    KeyCode::Char('n') => {
                         if app.current_view == View::ConfirmSave {
                             app.current_view = View::SettingsView;
                         }
                    }
                    KeyCode::Esc => app.go_back(),
                    KeyCode::Char(c) if app.current_view == View::EditingSettings => {
                        app.edit_buffer.push(c);
                    }
                    KeyCode::Backspace => {
                        if app.current_view == View::EditingSettings {
                            app.edit_buffer.pop();
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}

fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)].as_ref())
        .split(f.size());

    match app.current_view {
        View::MenuView => draw_menu_view(f, chunks[0], app),
        View::SettingsView => draw_settings_view(f, chunks[0], app),
        View::EntryDetailView => {
            let center = Paragraph::new("Entry Detail View (Placeholder)")
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(center, chunks[0]);
        }
        View::EditingSettings => draw_editing_view(f, chunks[0], app),
        View::ConfirmSave => draw_confirm_view(f, chunks[0], app),
    }

    let status_bar = match app.current_view {
        View::MenuView => Paragraph::new(" [s]: Settings | [↑/↓]: Move | [Enter]: Select | [q]: Quit ]"),
        View::SettingsView => Paragraph::new(" [s]: Menu | [↑/↓]: Move | [e]: Edit | [Shift+S]: Save | [q]: Quit ]"),
        View::EntryDetailView => Paragraph::new(" [Esc]: Back | [q]: Quit ]"),
        View::EditingSettings => Paragraph::new(" [Enter]: Apply | [c]: Cancel | Type to edit ]"),
        View::ConfirmSave => Paragraph::new(" [y]: Yes (Save) | [n]: No | [Esc]: Back ]"),
    }.block(Block::default().borders(Borders::TOP));
    f.render_widget(status_bar, chunks[1]);
}

fn draw_menu_view(f: &mut Frame, area: Rect, app: &App) {
    let items: Vec<ListItem> = app.menu_items.iter().map(|item| {
        let label = match item {
            crate::model::entry::GrubMenu::Entry(e) => format!("  {}", e.name),
            crate::model::entry::GrubMenu::Submenu { name, .. } => format!("> {}", name),
        };
        ListItem::new(label)
    }).collect();

    let list = List::new(items)
        .block(Block::default().title(" GRUB Menu ").borders(Borders::ALL))
        .highlight_style(Style::default().bg(Color::Blue).fg(Color::White))
        .highlight_symbol(">> ");

    let mut state = ListState::default();
    state.select(Some(app.menu_selection));
    f.render_stateful_widget(list, area, &mut state);
}

fn draw_settings_view(f: &mut Frame, area: Rect, app: &App) {
    let items: Vec<ListItem> = app.settings.entries.iter().map(|(k, v)| {
        ListItem::new(format!("  {} = {}", k, v))
    }).collect();

    let list = List::new(items)
        .block(Block::default().title(" GRUB Settings (s: Menu | e: Edit | Shift+S: Save) ").borders(Borders::ALL))
        .highlight_style(Style::default().bg(Color::Blue).fg(Color::White))
        .highlight_symbol(">> ");

    let mut state = ListState::default();
    state.select(Some(app.settings_selection));
    f.render_stateful_widget(list, area, &mut state);
}

fn draw_editing_view(f: &mut Frame, area: Rect, app: &App) {
    let key = app.editing_key.as_deref().unwrap_or("Unknown");
    let input = Paragraph::new(app.edit_buffer.as_str())
        .block(Block::default().title(format!(" Editing {} (Enter: Apply | c: Cancel) ", key)).borders(Borders::ALL));
    f.render_widget(input, area);
}

fn draw_confirm_view(f: &mut Frame, area: Rect, app: &App) {
    let text = "Are you sure you want to save changes? (y/n)";
    let block = Paragraph::new(text)
        .block(Block::default().title(" Confirm Save ").borders(Borders::ALL));
    f.render_widget(block, area);
}
