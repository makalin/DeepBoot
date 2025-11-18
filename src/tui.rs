use crate::actions::handle_action;
use crate::batch::{BatchProcessor, BatchResult};
use crate::config::ConfigManager;
use crate::export::Exporter;
use crate::filter::{Filter, SortBy};
use crate::logger::ActionLogger;
use crate::models::{Action, StartupEntry};
use crate::stats::ScanStatistics;
use crate::whitelist::WhitelistManager;
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame, Terminal,
};
use std::io;

#[derive(PartialEq)]
enum ViewMode {
    List,
    Stats,
    Help,
}

pub struct App {
    pub all_entries: Vec<StartupEntry>,
    pub filtered_entries: Vec<StartupEntry>,
    pub selected_indices: Vec<usize>, // For multi-select
    pub selected_index: usize,
    pub list_state: ListState,
    pub view_mode: ViewMode,
    pub show_help: bool,
    pub message: Option<String>,
    pub pending_action: Option<(Action, Vec<usize>)>, // Support batch actions
    pub search_term: String,
    pub filter: Filter,
    pub stats: ScanStatistics,
    pub whitelist_manager: WhitelistManager,
    pub logger: ActionLogger,
    pub config_manager: std::cell::RefCell<ConfigManager>,
    pub sort_by: SortBy,
}

impl App {
    pub fn new(
        entries: Vec<StartupEntry>,
        whitelist_manager: WhitelistManager,
        logger: ActionLogger,
        config_manager: ConfigManager,
    ) -> Self {
        let stats = ScanStatistics::from_entries(&entries);
        let mut filter = Filter::new();
        
        // Apply default sort from config
        let sort_by = match config_manager.borrow().get().default_sort.as_str() {
            "name" => SortBy::Name,
            "source" => SortBy::Source,
            "status" => SortBy::Status,
            "command" => SortBy::Command,
            _ => SortBy::Name,
        };

        let mut filtered_entries = filter.apply(&entries);
        crate::filter::sort_entries(&mut filtered_entries, sort_by);

        let mut list_state = ListState::default();
        if !filtered_entries.is_empty() {
            list_state.select(Some(0));
        }

        Self {
            all_entries: entries,
            filtered_entries,
            selected_indices: vec![],
            selected_index: 0,
            list_state,
            view_mode: ViewMode::List,
            show_help: false,
            message: None,
            pending_action: None,
            search_term: String::new(),
            filter,
            stats,
            whitelist_manager,
            logger,
            config_manager: std::cell::RefCell::new(config_manager),
            sort_by,
        }
    }

    pub fn next(&mut self) {
        if !self.filtered_entries.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.filtered_entries.len();
            self.list_state.select(Some(self.selected_index));
        }
    }

    pub fn previous(&mut self) {
        if !self.filtered_entries.is_empty() {
            self.selected_index = if self.selected_index == 0 {
                self.filtered_entries.len() - 1
            } else {
                self.selected_index - 1
            };
            self.list_state.select(Some(self.selected_index));
        }
    }

    pub fn get_selected_entry(&self) -> Option<&StartupEntry> {
        self.filtered_entries.get(self.selected_index)
    }

    pub fn apply_filter(&mut self) {
        self.filtered_entries = if !self.search_term.is_empty() {
            self.filter.with_search(self.search_term.clone()).apply(&self.all_entries)
        } else {
            self.filter.apply(&self.all_entries)
        };
        crate::filter::sort_entries(&mut self.filtered_entries, self.sort_by);
        self.stats = ScanStatistics::from_entries(&self.filtered_entries);
        
        // Adjust selected index
        if self.selected_index >= self.filtered_entries.len() && !self.filtered_entries.is_empty() {
            self.selected_index = self.filtered_entries.len() - 1;
        }
        if !self.filtered_entries.is_empty() {
            self.list_state.select(Some(self.selected_index));
        }
    }

    pub fn set_message(&mut self, msg: String) {
        self.message = Some(msg);
    }

    pub fn clear_message(&mut self) {
        self.message = None;
    }

    pub fn toggle_selection(&mut self) {
        let idx = self.get_original_index(self.selected_index);
        if let Some(pos) = self.selected_indices.iter().position(|&i| i == idx) {
            self.selected_indices.remove(pos);
        } else {
            self.selected_indices.push(idx);
        }
    }

    fn get_original_index(&self, filtered_idx: usize) -> usize {
        if let Some(entry) = self.filtered_entries.get(filtered_idx) {
            self.all_entries.iter().position(|e| {
                e.name == entry.name && e.source == entry.source && e.command == entry.command
            }).unwrap_or(0)
        } else {
            0
        }
    }
}

pub fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
) -> Result<()> {
    loop {
        terminal.draw(|f| ui(f, &app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        if app.pending_action.is_none() && app.search_term.is_empty() {
                            if app.view_mode == ViewMode::Help || app.view_mode == ViewMode::Stats {
                                app.view_mode = ViewMode::List;
                            } else {
                                return Ok(());
                            }
                        } else {
                            app.pending_action = None;
                            app.search_term.clear();
                            app.clear_message();
                        }
                    }
                    KeyCode::Char('h') => {
                        if app.pending_action.is_none() {
                            app.view_mode = if app.view_mode == ViewMode::Help {
                                ViewMode::List
                            } else {
                                ViewMode::Help
                            };
                        }
                    }
                    KeyCode::Char('s') => {
                        if app.pending_action.is_none() {
                            app.view_mode = if app.view_mode == ViewMode::Stats {
                                ViewMode::List
                            } else {
                                ViewMode::Stats
                            };
                        }
                    }
                    KeyCode::Char('/') => {
                        if app.pending_action.is_none() {
                            app.search_term.clear();
                            app.set_message("Enter search term (press Enter to search, Esc to cancel)".to_string());
                        }
                    }
                    KeyCode::Enter => {
                        if !app.search_term.is_empty() {
                            app.apply_filter();
                            app.clear_message();
                        }
                    }
                    KeyCode::Char(c) if !app.search_term.is_empty() && c != '/' => {
                        app.search_term.push(c);
                    }
                    KeyCode::Backspace => {
                        if !app.search_term.is_empty() {
                            app.search_term.pop();
                            app.apply_filter();
                        }
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        if app.pending_action.is_none() {
                            app.next();
                        }
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        if app.pending_action.is_none() {
                            app.previous();
                        }
                    }
                    KeyCode::Char('d') => {
                        if app.pending_action.is_none() {
                            if !app.selected_indices.is_empty() {
                                // Batch disable
                                app.pending_action = Some((Action::Disable, app.selected_indices.clone()));
                                app.set_message(format!(
                                    "Press 'y' to disable {} selected entries or 'n' to cancel",
                                    app.selected_indices.len()
                                ));
                            } else if let Some(entry) = app.get_selected_entry() {
                                app.pending_action = Some((Action::Disable, vec![app.get_original_index(app.selected_index)]));
                                app.set_message(format!(
                                    "Press 'y' to disable '{}' or 'n' to cancel",
                                    entry.name
                                ));
                            }
                        }
                    }
                    KeyCode::Char('r') => {
                        if app.pending_action.is_none() {
                            if !app.selected_indices.is_empty() {
                                // Batch remove
                                app.pending_action = Some((Action::Remove, app.selected_indices.clone()));
                                app.set_message(format!(
                                    "Press 'y' to remove {} selected entries or 'n' to cancel",
                                    app.selected_indices.len()
                                ));
                            } else if let Some(entry) = app.get_selected_entry() {
                                app.pending_action = Some((Action::Remove, vec![app.get_original_index(app.selected_index)]));
                                app.set_message(format!(
                                    "Press 'y' to remove '{}' or 'n' to cancel",
                                    entry.name
                                ));
                            }
                        }
                    }
                    KeyCode::Char('e') => {
                        if app.pending_action.is_none() {
                            // Export
                            match Exporter::export_json(&app.filtered_entries, None) {
                                Ok(path) => {
                                    app.set_message(format!("Exported to: {:?}", path));
                                }
                                Err(e) => {
                                    app.set_message(format!("Export failed: {}", e));
                                }
                            }
                        }
                    }
                    KeyCode::Char('w') => {
                        if app.pending_action.is_none() {
                            if let Some(entry) = app.get_selected_entry() {
                                match app.whitelist_manager.add_to_whitelist(entry) {
                                    Ok(_) => {
                                        app.set_message(format!("Added '{}' to whitelist", entry.name));
                                    }
                                    Err(e) => {
                                        app.set_message(format!("Failed to whitelist: {}", e));
                                    }
                                }
                            }
                        }
                    }
                    KeyCode::Char(' ') => {
                        if app.pending_action.is_none() {
                            app.toggle_selection();
                        }
                    }
                    KeyCode::Char('1') => {
                        app.sort_by = SortBy::Name;
                        app.apply_filter();
                    }
                    KeyCode::Char('2') => {
                        app.sort_by = SortBy::Source;
                        app.apply_filter();
                    }
                    KeyCode::Char('3') => {
                        app.sort_by = SortBy::Status;
                        app.apply_filter();
                    }
                    KeyCode::Char('4') => {
                        app.sort_by = SortBy::Command;
                        app.apply_filter();
                    }
                    KeyCode::Char('y') => {
                        if let Some((action, indices)) = app.pending_action.take() {
                            let entries_to_process: Vec<&StartupEntry> = indices
                                .iter()
                                .filter_map(|&idx| app.all_entries.get(idx))
                                .collect();

                            if entries_to_process.len() > 1 {
                                // Batch operation
                                let batch_processor = BatchProcessor::new(Some(app.logger.clone()));
                                let result = batch_processor.process_batch(
                                    &entries_to_process.iter().map(|e| (*e).clone()).collect::<Vec<_>>(),
                                    action,
                                );
                                app.set_message(result.summary());
                                
                                // Refresh entries
                                app.apply_filter();
                            } else if let Some(entry) = entries_to_process.first() {
                                // Single operation
                                match handle_action(entry, action) {
                                    Ok(_) => {
                                        let _ = app.logger.log_action(
                                            &action.to_string(),
                                            &entry.name,
                                            true,
                                            None,
                                        );
                                        app.set_message(format!(
                                            "Successfully {}d '{}'",
                                            action,
                                            entry.name
                                        ));
                                        if let Action::Disable = action {
                                            if let Some(e) = app.all_entries.iter_mut().find(|e| e.name == entry.name) {
                                                e.enabled = false;
                                            }
                                        } else if let Action::Remove = action {
                                            app.all_entries.retain(|e| e.name != entry.name);
                                        }
                                        app.apply_filter();
                                    }
                                    Err(e) => {
                                        let _ = app.logger.log_action(
                                            &action.to_string(),
                                            &entry.name,
                                            false,
                                            Some(&e.to_string()),
                                        );
                                        app.set_message(format!(
                                            "Error: Failed to {} '{}': {}",
                                            action,
                                            entry.name,
                                            e
                                        ));
                                    }
                                }
                            }
                        }
                    }
                    KeyCode::Char('n') => {
                        app.pending_action = None;
                        app.clear_message();
                    }
                    _ => {}
                }
            }
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &App) {
    match app.view_mode {
        ViewMode::Stats => {
            render_stats_view(f, app);
        }
        ViewMode::Help => {
            render_help_view(f, app);
        }
        ViewMode::List => {
            render_list_view(f, app);
        }
    }
}

fn render_list_view<B: Backend>(f: &mut Frame<B>, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Status bar
            Constraint::Min(10),  // Main list
            Constraint::Length(6), // Details
        ])
        .split(f.size());

    // Status bar
    let status_text = vec![
        Line::from(vec![
            Span::styled(
                format!("Entries: {}/{} | ", app.filtered_entries.len(), app.all_entries.len()),
                Style::default().fg(Color::Cyan),
            ),
            Span::styled(
                format!("Selected: {} | ", app.selected_indices.len()),
                Style::default().fg(Color::Yellow),
            ),
            Span::styled(
                format!("Sort: {:?} | ", app.sort_by),
                Style::default().fg(Color::Magenta),
            ),
            if !app.search_term.is_empty() {
                Span::styled(
                    format!("Search: {} | ", app.search_term),
                    Style::default().fg(Color::Green),
                )
            } else {
                Span::raw("")
            },
            Span::styled("Press 'h' for help", Style::default().fg(Color::DarkGray)),
        ]),
    ];

    let status = Paragraph::new(status_text)
        .block(Block::default().borders(Borders::ALL).title("Status"));
    f.render_widget(status, chunks[0]);

    // Main list
    let list_items: Vec<ListItem> = app
        .filtered_entries
        .iter()
        .enumerate()
        .map(|(idx, entry)| {
            let is_selected = app.selected_indices.contains(&app.get_original_index(idx));
            let is_current = idx == app.selected_index;

            let selection_indicator = if is_selected {
                Span::styled("✓ ", Style::default().fg(Color::Green))
            } else {
                Span::raw("  ")
            };

            let enabled_indicator = if entry.enabled {
                Span::styled("● ", Style::default().fg(Color::Green))
            } else {
                Span::styled("○ ", Style::default().fg(Color::Red))
            };

            let source = Span::styled(
                format!("[{}] ", entry.source),
                Style::default().fg(Color::Cyan),
            );

            let name = Span::styled(
                entry.name.clone(),
                if is_current {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                },
            );

            let command = Span::styled(
                format!(" → {}", entry.command),
                Style::default().fg(Color::Gray),
            );

            ListItem::new(vec![selection_indicator, enabled_indicator, source, name, command])
        })
        .collect();

    let list = List::new(list_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("DeepBoot Pro - Startup Entries")
                .title_alignment(Alignment::Center),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    f.render_stateful_widget(list, chunks[1], &mut app.list_state.clone());

    // Details panel
    let details_text = if let Some(entry) = app.get_selected_entry() {
        vec![
            Line::from(Span::styled(
                format!("Name: {}", entry.name),
                Style::default().fg(Color::White),
            )),
            Line::from(Span::styled(
                format!("Source: {}", entry.source),
                Style::default().fg(Color::Cyan),
            )),
            Line::from(Span::styled(
                format!("Command: {}", entry.command),
                Style::default().fg(Color::Gray),
            )),
            Line::from(Span::styled(
                format!("Status: {}", if entry.enabled { "Enabled" } else { "Disabled" }),
                Style::default().fg(if entry.enabled { Color::Green } else { Color::Red }),
            )),
            if let Some(desc) = &entry.description {
                Line::from(Span::styled(
                    format!("Description: {}", desc),
                    Style::default().fg(Color::DarkGray),
                ))
            } else {
                Line::from("")
            },
        ]
    } else {
        vec![Line::from("No entry selected")]
    };

    let details = Paragraph::new(details_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Details")
                .title_alignment(Alignment::Center),
        )
        .wrap(Wrap { trim: true });

    f.render_widget(details, chunks[2]);

    // Show message if any
    if let Some(msg) = &app.message {
        let msg_paragraph = Paragraph::new(msg.as_str())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Message")
                    .title_alignment(Alignment::Center),
            )
            .style(Style::default().fg(Color::Yellow))
            .wrap(Wrap { trim: true });

        let area = centered_rect(60, 5, f.size());
        f.render_widget(msg_paragraph, area);
    }
}

fn render_stats_view<B: Backend>(f: &mut Frame<B>, app: &App) {
    let stats_text = app.stats.get_summary();
    let stats_lines: Vec<Line> = stats_text
        .lines()
        .map(|line| Line::from(Span::raw(line)))
        .collect();

    let stats_paragraph = Paragraph::new(stats_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Statistics")
                .title_alignment(Alignment::Center),
        )
        .wrap(Wrap { trim: true });

    f.render_widget(stats_paragraph, f.size());
}

fn render_help_view<B: Backend>(f: &mut Frame<B>, app: &App) {
    let help_text = vec![
        Line::from(""),
        Line::from(Span::styled("Navigation:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        Line::from("  ↑/k - Move up"),
        Line::from("  ↓/j - Move down"),
        Line::from("  Space - Toggle selection"),
        Line::from(""),
        Line::from(Span::styled("Actions:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        Line::from("  d   - Disable selected entry(ies)"),
        Line::from("  r   - Remove selected entry(ies)"),
        Line::from("  w   - Add to whitelist"),
        Line::from("  e   - Export to JSON"),
        Line::from(""),
        Line::from(Span::styled("Views:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        Line::from("  s   - Show statistics"),
        Line::from("  h   - Toggle help"),
        Line::from(""),
        Line::from(Span::styled("Search & Filter:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        Line::from("  /   - Start search"),
        Line::from("  Esc - Cancel search"),
        Line::from(""),
        Line::from(Span::styled("Sorting:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        Line::from("  1   - Sort by name"),
        Line::from("  2   - Sort by source"),
        Line::from("  3   - Sort by status"),
        Line::from("  4   - Sort by command"),
        Line::from(""),
        Line::from(Span::styled("Other:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        Line::from("  q   - Quit"),
        Line::from(""),
        Line::from(Span::styled("Legend:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        Line::from("  ● - Enabled"),
        Line::from("  ○ - Disabled"),
        Line::from("  ✓ - Selected"),
    ];

    let help_paragraph = Paragraph::new(help_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Help - DeepBoot Pro")
                .title_alignment(Alignment::Center),
        )
        .wrap(Wrap { trim: true });

    f.render_widget(help_paragraph, f.size());
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
