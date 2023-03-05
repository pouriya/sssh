use crate::error::AppError;
use crate::settings::{ConfigServer, Settings, DEFAULT_PORT_NUMBER};
use crate::subcommands::edit::run as edit;
use crate::utils::run_command;
use anyhow::Result;
use clap::crate_name;
use crossterm::event::Event::Key;
use crossterm::{
    event,
    event::KeyCode,
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::collections::HashMap;
use std::error::Error;
use std::path::PathBuf;
use tracing::debug;
use tui::layout::Rect;
use tui::text::Text;
use tui::widgets::{List, ListItem};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{
        Block, BorderType, Borders, Cell, ListState, Paragraph, Row, Table, TableState, Wrap,
    },
    Frame, Terminal,
};

#[derive(Debug)]
enum ControlFlow {
    Stop,
    Edit,
    Selected,
    Reload,
}

#[derive(Debug, Clone, Default)]
struct State {
    server_offset: usize,
    username_offset: usize,
    choosing_server: bool,
    choosing_username: bool,
    selected_server: bool,
    server_list: Vec<ConfigServer>,
    server_table_state: TableState,
    username_list_state: ListState,
    working_keys: WorkingKeys,
}

impl PartialEq for State {
    fn eq(&self, other: &Self) -> bool {
        [
            self.choosing_server,
            self.choosing_username,
            self.selected_server,
        ] == [
            other.choosing_server,
            other.choosing_username,
            other.selected_server,
        ] && [self.server_offset, self.username_offset]
            == [other.server_offset, other.username_offset]
    }
}

#[derive(Debug, Clone, PartialEq, Copy)]
struct Theme {
    main_border_bg: Color,
    main_border_fg: Color,
    main_border_title_bg: Color,
    main_border_title_fg: Color,

    help_text_bg: Color,
    help_text_fg: Color,
    help_hint_text_bg: Color,
    help_hint_text_fg: Color,
    help_error_text_bg: Color,
    help_error_text_fg: Color,
    help_paragraph_bg: Color,
    help_paragraph_fg: Color,

    table_border_bg: Color,
    table_border_fg: Color,
    table_border_active_fg: Color,
    table_border_title_bg: Color,
    table_border_title_fg: Color,
    table_title_bg: Color,
    table_title_fg: Color,
    table_row_name_bg: Color,
    table_row_name_fg: Color,
    table_row_hostname_bg: Color,
    table_row_hostname_fg: Color,
    table_row_description_bg: Color,
    table_row_description_fg: Color,
    table_highlight_bg: Color,
    table_highlight_fg: Color,

    list_border_bg: Color,
    list_border_fg: Color,
    list_border_active_fg: Color,
    list_border_title_bg: Color,
    list_border_title_fg: Color,
    username_bg: Color,
    username_fg: Color,
    list_highlight_bg: Color,
    list_highlight_fg: Color,

    help_key_working_bg: Color,
    help_key_working_fg: Color,
    help_key_not_working_bg: Color,
    help_key_not_working_fg: Color,
    help_key_guide_working_bg: Color,
    help_key_guide_working_fg: Color,
    help_key_guide_not_working_bg: Color,
    help_key_guide_not_working_fg: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            main_border_bg: Color::Reset,
            main_border_fg: Color::White,
            main_border_title_bg: Color::Reset,
            main_border_title_fg: Color::White,

            help_text_bg: Color::Reset,
            help_text_fg: Color::White,
            help_hint_text_bg: Color::Reset,
            help_hint_text_fg: Color::White,
            help_error_text_bg: Color::Reset,
            help_error_text_fg: Color::Red,
            help_paragraph_bg: Color::Reset,
            help_paragraph_fg: Color::Reset,

            table_border_bg: Color::Reset,
            table_border_fg: Color::Yellow,
            table_border_active_fg: Color::LightYellow,
            table_border_title_bg: Color::Reset,
            table_border_title_fg: Color::Yellow,
            table_title_bg: Color::Reset,
            table_title_fg: Color::LightYellow,
            table_row_name_bg: Color::Reset,
            table_row_name_fg: Color::White,
            table_row_hostname_bg: Color::Reset,
            table_row_hostname_fg: Color::Green,
            table_row_description_bg: Color::Reset,
            table_row_description_fg: Color::LightBlue,
            table_highlight_bg: Color::Reset,
            table_highlight_fg: Color::Reset,

            list_border_bg: Color::Reset,
            list_border_fg: Color::Yellow,
            list_border_active_fg: Color::LightYellow,
            list_border_title_bg: Color::Reset,
            list_border_title_fg: Color::Yellow,
            username_bg: Color::Reset,
            username_fg: Color::LightBlue,
            list_highlight_bg: Color::Reset,
            list_highlight_fg: Color::Reset,

            help_key_working_bg: Color::Reset,
            help_key_working_fg: Color::LightYellow,
            help_key_not_working_bg: Color::Reset,
            help_key_not_working_fg: Color::DarkGray,
            help_key_guide_working_bg: Color::Reset,
            help_key_guide_working_fg: Color::White,
            help_key_guide_not_working_bg: Color::Reset,
            help_key_guide_not_working_fg: Color::DarkGray,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct WorkingKeys {
    pub q: bool,
    pub e: bool,
    pub r: bool,
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    pub enter: bool,
}

impl WorkingKeys {
    pub fn to_info_list(&self) -> Vec<(&'static str, &'static str, bool)> {
        [
            ("Up", "Previous server/username", self.up),
            ("Down", "Next server/username", self.down),
            ("Left", "Choose from servers", self.left),
            ("Right", "Choose from usernames", self.right),
            ("q", "Quit", self.q),
            ("e", "Edit config file", self.e),
            ("r", "Reload config file", self.r),
            ("Enter", "Choose", self.enter),
        ]
        .to_vec()
    }
}

impl TryFrom<HashMap<String, ConfigServer>> for State {
    type Error = AppError;

    fn try_from(value: HashMap<String, ConfigServer>) -> Result<Self, Self::Error> {
        let mut state = State::default();
        value
            .into_iter()
            .for_each(|(_, config)| state.server_list.push(config));
        state.server_list.sort_by_key(|server| server.name.clone());
        state.working_keys.q = true;
        state.working_keys.e = true;
        state.working_keys.r = true;
        if !state.server_list.is_empty() {
            state.working_keys = WorkingKeys {
                q: true,
                e: true,
                r: true,
                up: true,
                down: true,
                left: true,
                right: true,
                enter: true,
            };
            state.choosing_server = true;
            state.next_server();
        }
        Ok(state)
    }
}

impl State {
    pub fn next_server(&mut self) {
        let offset = match self.server_table_state.selected() {
            Some(offset) => {
                if offset >= self.server_list.len() - 1 {
                    0
                } else {
                    offset + 1
                }
            }
            None => 0,
        };
        self.server_offset = offset;
        self.server_table_state.select(Some(offset));
    }

    pub fn previous_server(&mut self) {
        let offset = match self.server_table_state.selected() {
            Some(offset) => {
                if offset == 0 {
                    self.server_list.len() - 1
                } else {
                    offset - 1
                }
            }
            None => 0,
        };
        self.server_offset = offset;
        self.server_table_state.select(Some(offset));
    }

    fn next_username(&mut self) {
        let offset = match self.username_list_state.selected() {
            Some(offset) => {
                if offset >= self.server_list[self.server_offset].username_list.len() - 1 {
                    0
                } else {
                    offset + 1
                }
            }
            None => 0,
        };
        self.username_offset = offset;
        self.username_list_state.select(Some(offset));
    }

    fn previous_username(&mut self) {
        let offset = match self.username_list_state.selected() {
            Some(offset) => {
                if offset == 0 {
                    self.server_list[self.server_offset].username_list.len() - 1
                } else {
                    offset - 1
                }
            }
            None => 0,
        };
        self.username_offset = offset;
        self.username_list_state.select(Some(offset));
    }
}

pub fn run(settings: &mut Settings) -> Result<(), AppError> {
    let mut maybe_error = match settings.try_load_and_set_configuration() {
        Ok(_) => None,
        Err(ref error @ AppError::ConfigSyntax { ref source, .. }) => {
            Some(format!("{}\n{}", error, source))
        }
        Err(error) => return Err(error),
    };
    settings.check_editor_command()?;
    let theme = Theme::default();
    loop {
        settings.ensure_script_file()?;
        let mut state = State::try_from(settings.configuration.servers.clone())?;
        enable_raw_mode().map_err(|source| AppError::UI { source })?;
        let mut stdout = std::io::stdout();
        execute!(stdout, EnterAlternateScreen).map_err(|source| AppError::UI { source })?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend).map_err(|source| AppError::UI { source })?;
        let result = run_tui(
            settings,
            &mut state,
            &mut terminal,
            maybe_error.clone(),
            theme,
        );
        disable_raw_mode().map_err(|source| AppError::UI { source })?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)
            .map_err(|source| AppError::UI { source })?;
        terminal
            .show_cursor()
            .map_err(|source| AppError::UI { source })?;
        match result {
            Ok(ControlFlow::Edit) => {
                maybe_error = edit_and_convert_errors(settings)?;
            }
            Ok(ControlFlow::Stop) => return Ok(()),
            Ok(ControlFlow::Selected) => {
                let server = state.server_list[state.server_offset].clone();
                let username = server.username_list[state.username_offset].clone();
                let run_scrip_result = if settings.skip_select {
                    println!(
                        "You have selected `{}` ({}@{}). Skip running script file.",
                        server.name, username, server.hostname
                    );
                    Ok(())
                } else {
                    run_script(settings, server, username)
                };
                return run_scrip_result;
            }
            Ok(ControlFlow::Reload) => {
                maybe_error = match settings.try_load_and_set_configuration() {
                    Ok(_) => None,
                    Err(ref error @ AppError::ConfigSyntax { ref source, .. }) => {
                        Some(format!("{}\n{}", error, source))
                    }
                    Err(error) => return Err(error),
                };
            }
            Err(error) => return Err(error),
        }
    }
}

fn edit_and_convert_errors(settings: &mut Settings) -> Result<Option<String>, AppError> {
    match edit(settings) {
        Ok(_) => Ok(None),
        Err(ref error @ AppError::ConfigSyntax { .. }) => {
            Ok(Some(error_to_string(error, String::new())))
        }
        Err(ref error @ AppError::EditorFastStop) => Ok(Some(format!(
            "{}\nAfter editing configuration file press `r` to reload it.\n\nfile: {:?}",
            error_to_string(error, String::new()),
            settings.configuration_file
        ))),
        Err(ref error @ AppError::ProcessStart { .. }) => Ok(Some(format!(
            "{}\nEdit the file manually and press `r` to reload it.",
            error_to_string(error, String::new())
        ))),
        Err(ref error @ AppError::ProcessWait { .. }) => Ok(Some(format!(
            "{}\nEdit the file manually and press `r` to reload it.",
            error_to_string(error, String::new())
        ))),
        Err(ref error @ AppError::ProcessFailed { .. }) => Ok(Some(format!(
            "{}\nEdit the file manually and press `r` to reload it.",
            error_to_string(error, String::new())
        ))),
        Err(error) => Err(error),
    }
}

fn error_to_string<E: Error>(error: E, prefix: String) -> String {
    let text = format!("{}\n{}", prefix, error);
    if let Some(source) = error.source() {
        error_to_string(source, text)
    } else {
        text
    }
}

fn run_tui<B: Backend>(
    settings: &mut Settings,
    state: &mut State,
    terminal: &mut Terminal<B>,
    maybe_error: Option<String>,
    theme: Theme,
) -> Result<ControlFlow, AppError> {
    loop {
        terminal
            .draw(|frame| draw_ui(settings, state, frame, maybe_error.clone(), theme))
            .map_err(|source| AppError::UI { source })?;
        let event = event::read().map_err(|source| AppError::UI { source })?;
        debug!(event = ?event, "Got new terminal event");
        let mut state_before_handling_event = state.clone();
        let maybe_control_flow = match event {
            Key(key) => match key.code {
                KeyCode::Char('q') => Some(ControlFlow::Stop),
                KeyCode::Char('e') => Some(ControlFlow::Edit),
                KeyCode::Char('r') => Some(ControlFlow::Reload),
                KeyCode::Enter if state.choosing_username => Some(ControlFlow::Selected),
                _ if maybe_error.is_none() => {
                    match key.code {
                        up_down if up_down == KeyCode::Up || up_down == KeyCode::Down => {
                            if state.choosing_username {
                                if up_down == KeyCode::Up {
                                    state.previous_username()
                                } else if up_down == KeyCode::Down {
                                    state.next_username()
                                } else {
                                    unreachable!()
                                }
                            } else if up_down == KeyCode::Up {
                                state.previous_server()
                            } else if up_down == KeyCode::Down {
                                state.next_server()
                            } else {
                                unreachable!()
                            }
                        }
                        left_right
                            if [
                                KeyCode::Left,
                                KeyCode::Right,
                                KeyCode::Backspace,
                                KeyCode::Enter,
                            ]
                            .contains(&left_right) =>
                        {
                            if [KeyCode::Left, KeyCode::Backspace].contains(&left_right) {
                                state.choosing_username = false;
                                state.username_list_state.select(None);
                                state.choosing_server = true;
                            } else if [KeyCode::Right, KeyCode::Enter].contains(&left_right) {
                                state.choosing_username = true;
                                state.next_username();
                                state.choosing_server = false;
                            } else {
                                unreachable!()
                            }
                        }
                        key_code => debug!(key_code = ?key_code, "Unhandled terminal key event"),
                    };
                    None
                }
                key_code => {
                    debug!(key_code = ?key_code, "Unhandled terminal key event");
                    None
                }
            },
            event => {
                debug!(event = ?event, "Unhandled terminal event");
                None
            }
        };
        if state != &mut state_before_handling_event {
            debug!(previous = ?state_before_handling_event, now = ?state, "State changed during handling terminal event")
        }
        if let Some(control_flow) = maybe_control_flow {
            debug!(control_flow = ?control_flow, "Control flow changed during handling terminal event");
            return Ok(control_flow);
        }
    }
}

fn draw_ui<B: Backend>(
    settings: &mut Settings,
    state: &mut State,
    frame: &mut Frame<B>,
    maybe_error: Option<String>,
    theme: Theme,
) {
    let size = frame.size();
    let block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(
            crate_name!(),
            Style::default()
                .bg(theme.main_border_title_bg)
                .fg(theme.main_border_title_fg)
                .add_modifier(Modifier::BOLD),
        ))
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .border_style(
            Style::default()
                .bg(theme.main_border_bg)
                .fg(theme.main_border_fg),
        );
    frame.render_widget(block, size);
    let chunk_list = Layout::default()
        .direction(Direction::Vertical)
        .vertical_margin(2)
        .horizontal_margin(3)
        .constraints(
            [
                Constraint::Percentage(10),
                Constraint::Percentage(75),
                Constraint::Percentage(15),
            ]
            .as_ref(),
        )
        .split(frame.size());
    main_help_ui(
        settings,
        state,
        frame,
        chunk_list[0],
        maybe_error.clone(),
        theme,
    );
    if let Some(error) = maybe_error {
        error_ui(settings, state, frame, chunk_list[1], error, theme);
    } else {
        let table_chunk_list = Layout::default()
            .direction(Direction::Horizontal)
            .vertical_margin(1)
            .horizontal_margin(0)
            .constraints([Constraint::Percentage(80), Constraint::Percentage(20)].as_ref())
            .split(chunk_list[1]);
        server_table_ui(settings, state, frame, table_chunk_list[0], theme);
        username_list_ui(settings, state, frame, table_chunk_list[1], theme);
    }
    help_key_ui(settings, state, frame, chunk_list[2], theme)
}

fn main_help_ui<B: Backend>(
    settings: &mut Settings,
    _state: &mut State,
    frame: &mut Frame<B>,
    rect: Rect,
    maybe_error: Option<String>,
    theme: Theme,
) {
    let help_text = if maybe_error.is_some() {
        Text::from(Span::styled(
            "There is no server to choose...",
            Style::default()
                .bg(theme.help_error_text_bg)
                .fg(theme.help_error_text_fg)
                .add_modifier(Modifier::BOLD),
        ))
    } else {
        let mut text_list = [Spans::from(Span::styled(
            "Select Your SSH server to connect.",
            Style::default()
                .bg(theme.help_text_bg)
                .fg(theme.help_text_fg)
                .add_modifier(Modifier::BOLD),
        ))]
        .to_vec();
        if settings.is_default_servers() {
            text_list.push(Spans::from(
                [
                    Span::styled(
                        "Hint: Press ",
                        Style::default()
                            .bg(theme.help_hint_text_bg)
                            .fg(theme.help_hint_text_fg)
                            .add_modifier(Modifier::DIM),
                    ),
                    Span::styled(
                        "e",
                        Style::default()
                            .bg(theme.help_hint_text_bg)
                            .fg(theme.help_hint_text_fg)
                            .add_modifier(Modifier::BOLD)
                            .add_modifier(Modifier::DIM)
                            .add_modifier(Modifier::RAPID_BLINK),
                    ),
                    Span::styled(
                        " key to edit configuration file",
                        Style::default()
                            .bg(theme.help_hint_text_bg)
                            .fg(theme.help_hint_text_fg)
                            .add_modifier(Modifier::DIM),
                    ),
                ]
                .to_vec(),
            ))
        }
        Text::from(text_list)
    };
    let help_paragraph = Paragraph::new(help_text)
        .style(
            Style::default()
                .bg(theme.help_paragraph_bg)
                .fg(theme.help_paragraph_fg),
        )
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    frame.render_widget(help_paragraph, rect);
}

fn error_ui<B: Backend>(
    _settings: &mut Settings,
    _state: &mut State,
    frame: &mut Frame<B>,
    rect: Rect,
    error: String,
    theme: Theme,
) {
    let mut max_line_length = 0;
    let mut line_count = 0;
    let error_text: Vec<_> = error
        .lines()
        .map(|line| {
            if line.len() > max_line_length {
                max_line_length = line.len()
            };
            line_count += 1;
            Spans::from(Span::styled(line, Style::default().fg(Color::LightRed)))
        })
        .collect();
    let help_paragraph = Paragraph::new(error_text)
        .style(
            Style::default()
                .bg(theme.help_paragraph_bg)
                .fg(theme.help_paragraph_fg)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: false });
    let vertical_layout_list = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(if rect.height > line_count {
                    (rect.height - line_count) / 2
                } else {
                    0
                }),
                Constraint::Length(line_count),
                Constraint::Length(if rect.height > line_count {
                    (rect.height - line_count) / 2
                } else {
                    0
                }),
            ]
            .as_ref(),
        )
        .split(rect);
    let horizontal_layout_list = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Length(if rect.width > max_line_length as u16 {
                    (rect.width - max_line_length as u16) / 2
                } else {
                    0
                }),
                Constraint::Length(max_line_length as u16),
                Constraint::Length(if rect.width > max_line_length as u16 {
                    (rect.width - max_line_length as u16) / 2
                } else {
                    0
                }),
            ]
            .as_ref(),
        )
        .split(vertical_layout_list[1]);
    frame.render_widget(help_paragraph, horizontal_layout_list[1]);
}

fn server_table_ui<B: Backend>(
    _settings: &mut Settings,
    state: &mut State,
    frame: &mut Frame<B>,
    rect: Rect,
    theme: Theme,
) {
    let table_header_cells = ["Name", "Hostname", "Description"]
        .into_iter()
        .map(|header| {
            Cell::from(header).style(
                Style::default()
                    .bg(theme.table_title_bg)
                    .fg(theme.table_title_fg)
                    .add_modifier(Modifier::BOLD),
            )
        });
    let table_header = Row::new(table_header_cells)
        .style(
            Style::default()
                .fg(Color::Reset)
                .add_modifier(Modifier::BOLD),
        )
        .height(1)
        .bottom_margin(1);
    let row_list = state.server_list.iter().map(|server| {
        let height = server
            .description
            .clone()
            .chars()
            .filter(|character| *character == '\n')
            .count()
            + 1;
        let cell_list = [
            Cell::from(Span::styled(
                server.name.clone(),
                Style::default()
                    .bg(theme.table_row_name_bg)
                    .fg(theme.table_row_name_fg),
            )),
            Cell::from(Span::styled(
                format!(
                    "{}{}",
                    server.hostname.clone(),
                    if server.port == DEFAULT_PORT_NUMBER {
                        String::new()
                    } else {
                        String::from(":") + server.port.to_string().as_str()
                    }
                ),
                Style::default()
                    .bg(theme.table_row_hostname_bg)
                    .fg(theme.table_row_hostname_fg),
            )),
            Cell::from(
                server
                    .description
                    .lines()
                    .map(|line| {
                        Spans::from(Span::styled(
                            line,
                            Style::default()
                                .bg(theme.table_row_description_bg)
                                .fg(theme.table_row_description_fg),
                        ))
                    })
                    .collect::<Vec<_>>(),
            ),
        ]
        .to_vec();
        Row::new(cell_list).height(height as u16).bottom_margin(1)
    });
    let table = Table::new(row_list)
        .header(table_header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(Span::styled(
                    "Servers",
                    Style::default()
                        .bg(theme.table_border_title_bg)
                        .fg(theme.table_border_title_fg)
                        .add_modifier(Modifier::BOLD),
                ))
                .title_alignment(Alignment::Center)
                .border_style(Style::default().bg(theme.table_border_bg).fg(
                    if state.choosing_username {
                        theme.table_border_fg
                    } else {
                        theme.table_border_active_fg
                    },
                )),
        )
        .highlight_style(
            Style::default()
                .bg(theme.table_highlight_bg)
                .fg(theme.table_highlight_fg)
                .add_modifier(Modifier::BOLD),
        )
        // .highlight_symbol(">> ")
        .widths(&[
            Constraint::Percentage(15),
            Constraint::Percentage(20),
            Constraint::Percentage(65),
        ]);
    frame.render_stateful_widget(table, rect, &mut state.server_table_state);
}

fn username_list_ui<B: Backend>(
    _settings: &mut Settings,
    state: &mut State,
    frame: &mut Frame<B>,
    rect: Rect,
    theme: Theme,
) {
    let server = state.server_list[state.server_offset].clone();
    let items = server
        .username_list
        .iter()
        .map(|username| {
            ListItem::new(Span::styled(
                username,
                Style::default()
                    .bg(theme.username_bg)
                    .fg(theme.username_fg)
                    .add_modifier(Modifier::BOLD),
            ))
        })
        .collect::<Vec<_>>();
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().bg(theme.list_border_bg).fg(
                    if state.choosing_username {
                        theme.list_border_active_fg
                    } else {
                        theme.list_border_fg
                    },
                ))
                .title(Span::styled(
                    "Usernames",
                    Style::default()
                        .bg(theme.list_border_title_bg)
                        .fg(theme.list_border_title_fg)
                        .add_modifier(Modifier::BOLD),
                ))
                .title_alignment(Alignment::Center),
        )
        // .highlight_symbol(">> ")
        .highlight_style(
            Style::default()
                .bg(theme.list_highlight_bg)
                .fg(theme.list_highlight_fg)
                .add_modifier(Modifier::BOLD),
        );
    frame.render_stateful_widget(list, rect, &mut state.username_list_state);
}

fn help_key_ui<B: Backend>(
    _settings: &mut Settings,
    state: &mut State,
    frame: &mut Frame<B>,
    rect: Rect,
    theme: Theme,
) {
    // |----|----|----|----|
    // |----|----|----|----|
    // |----|----|----|----|
    // |----|----|----|----|
    let key_info_list = state.working_keys.to_info_list();
    let key_info_list_length = key_info_list.len();
    let column_size = 4;
    let mut row_size = key_info_list_length / column_size;
    if key_info_list_length % column_size != 0 {
        row_size += 1
    }
    let horizontal_constraint_list: Vec<Constraint> = (1..=row_size)
        .map(|_| Constraint::Percentage((100 / row_size) as u16))
        .collect();
    let horizontal_constraint_list_lenght = horizontal_constraint_list.len();
    let horizontal_area_list = Layout::default()
        .direction(Direction::Vertical)
        // .margin(3)
        .constraints(horizontal_constraint_list)
        .split(rect);
    let mut rect_list = Vec::new();
    for horizontal_constraint_offset in 0..horizontal_constraint_list_lenght {
        let horizontal_area = horizontal_area_list[horizontal_constraint_offset];
        let horizontal_constraint_list: Vec<Constraint> = (1..=column_size)
            .map(|_| Constraint::Percentage((100 / column_size) as u16))
            .collect();
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints(horizontal_constraint_list)
            .split(horizontal_area)
            .into_iter()
            .for_each(|rect| rect_list.push(rect));
    }
    for (offset, (key_name, help, is_working)) in key_info_list.into_iter().enumerate() {
        let help_text = [Spans::from(vec![
            Span::from("["),
            Span::styled(
                key_name,
                Style::default()
                    .bg(if is_working {
                        theme.help_key_working_bg
                    } else {
                        theme.help_key_not_working_bg
                    })
                    .fg(if is_working {
                        theme.help_key_working_fg
                    } else {
                        theme.help_key_not_working_fg
                    })
                    .add_modifier(Modifier::BOLD),
            ),
            Span::from("] "),
            Span::styled(
                help,
                Style::default()
                    .bg(if is_working {
                        theme.help_key_guide_working_bg
                    } else {
                        theme.help_key_guide_not_working_bg
                    })
                    .fg(if is_working {
                        theme.help_key_guide_working_fg
                    } else {
                        theme.help_key_guide_not_working_fg
                    }),
            ),
        ])]
        .to_vec();
        let help_paragraph = Paragraph::new(help_text)
            .style(Style::default())
            // .wrap(Wrap { trim: true })
            .alignment(Alignment::Center);
        frame.render_widget(help_paragraph, rect_list[offset]);
    }
}

fn run_script(
    settings: &mut Settings,
    server: ConfigServer,
    username: String,
) -> Result<(), AppError> {
    let hostname = server.hostname.clone();
    let address = format!("{}@{}", username, server.hostname);
    let port = server.port.to_string();
    let mut argument_list = [
        address.clone(),
        username.clone(),
        hostname.clone(),
        port.clone(),
    ]
    .to_vec();
    let mut env_list = [
        ("SSSH_ADDRESS", address.as_str()),
        ("SSSH_USERNAME", username.as_str()),
        ("SSSH_HOSTNAME", hostname.as_str()),
        ("SSSH_PORT", port.as_str()),
    ]
    .to_vec();
    if settings.verbose {
        argument_list.push("1".to_string());
        env_list.push(("SSSH_DEBUG", "1"));
    } else {
        argument_list.push("0".to_string());
    }
    let argument_list = argument_list
        .into_iter()
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    let command = settings.script_file.clone();
    let _ = run_command("Script", command, argument_list, env_list)?;
    Ok(())
}
