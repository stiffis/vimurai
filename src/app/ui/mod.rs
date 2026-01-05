use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

use super::screens::*;
use crate::engine::mode::VimMode;

pub struct UI;

impl UI {
    pub fn new() -> Self {
        Self
    }

    pub fn render(&self, frame: &mut Frame, app: &mut super::app::App) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(10),
                Constraint::Length(2),
                Constraint::Length(1),
            ])
            .split(frame.size());

        draw_title_bar(frame, chunks[0]);

        match app.current_screen {
            Screen::MainMenu => self.render_main_menu(frame, app, chunks[1]),
            Screen::DailyDrill | Screen::FreePractice => {
                self.render_practice(frame, app, chunks[1])
            }
            Screen::GuidedLearning => self.render_guided_learning(frame, app, chunks[1]),
            Screen::Progress => self.render_progress(frame, app, chunks[1]),
            Screen::Settings => self.render_settings(frame, app, chunks[1]),
            Screen::Help => self.render_help(frame, chunks[1]),
        }

        let status_message = match app.current_screen {
            Screen::MainMenu => "Main Menu".to_string(),
            Screen::DailyDrill => format!(
                "Daily Drill | Level: {:?}",
                app.practice_state.current_exercise.as_ref().map(|e| &e.title).unwrap_or(&"None".to_string())
            ),
            Screen::FreePractice => "Free Practice Mode".to_string(),
            Screen::GuidedLearning => "Guided Learning".to_string(),
            Screen::Progress => "Your Progress".to_string(),
            Screen::Settings => "Settings".to_string(),
            Screen::Help => "Help & Shortcuts".to_string(),
        };
        draw_status_bar(frame, app, &status_message);

        let help_hints = match app.current_screen {
            Screen::MainMenu => vec![("↑↓/jk", "navigate"), ("Enter", "select"), ("q", "quit")],
            Screen::DailyDrill | Screen::FreePractice | Screen::GuidedLearning => {
                vec![("Esc", "normal"), (":", "command"), ("i/a", "insert"), ("v", "visual")]
            }
            Screen::Progress => vec![("Esc/q", "back")],
            Screen::Settings => vec![("↑↓/jk", "navigate"), ("Space", "toggle"), ("Esc/q", "back")],
            Screen::Help => vec![("Esc/q", "back")],
        };
        draw_help_bar(frame, &help_hints);

        // Draw quit confirmation modal if showing
        if app.show_quit_confirm {
            draw_quit_confirm(frame);
        }
        
        // Draw success overlay if correct
        if app.practice_state.is_correct == Some(true) {
            draw_success_overlay(frame);
        }
    }

    fn render_main_menu(&self, frame: &mut Frame, app: &super::app::App, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(30),
                Constraint::Percentage(50),
                Constraint::Percentage(20),
            ])
            .split(area);

        let welcome_text = Text::from(vec![
            Line::from(vec![Span::styled(
                "VIMURAI",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD | Modifier::ITALIC),
            )]),
            Line::from(""),
            Line::from("Master Vim through"),
            Line::from("muscle memory."),
            Line::from(""),
            Line::from(vec![
                Span::styled("Daily Practice: ", Style::default().fg(Color::Yellow)),
                Span::styled("3-5 min", Style::default().fg(Color::Green)),
            ]),
            Line::from(vec![
                Span::styled("Commands: ", Style::default().fg(Color::Yellow)),
                Span::styled("25+", Style::default().fg(Color::Green)),
            ]),
            Line::from(""),
            Line::from("Your Stats:"),
            Line::from(format!("  Level: {}", app.progress_state.level)),
            Line::from(format!("  Streak: {} days", app.progress_state.streak_days)),
        ]);

        let welcome_block = Block::default()
            .title("Welcome")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Gray));
        frame.render_widget(welcome_block, chunks[0]);
        frame.render_widget(welcome_text, self.inner_area(chunks[0]));

        let menu_items: Vec<ListItem> = app
            .main_menu_state
            .items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let prefix = if i == app.main_menu_state.selected_index {
                    "►"
                } else {
                    " "
                };
                let title = format!("{} [{}] {}", prefix, item.shortcut, item.title);

                let style = if i == app.main_menu_state.selected_index {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };

                ListItem::new(Line::from(vec![Span::styled(title, style)])).style(
                    if i == app.main_menu_state.selected_index {
                        Style::default().bg(Color::DarkGray)
                    } else {
                        Style::default()
                    },
                )
            })
            .collect();

        let menu = List::new(menu_items)
            .block(Block::default().title("Menu").borders(Borders::ALL))
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

        frame.render_widget(menu, chunks[1]);

        let tips_text = Text::from(vec![
            Line::from(vec![Span::styled(
                "Quick Tips",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from("• Practice daily"),
            Line::from("• Focus on basics"),
            Line::from("• Build muscle"),
            Line::from("  memory"),
            Line::from("• Use 'A' to add"),
            Line::from("  at line end"),
        ]);

        let tips_block = Block::default()
            .title("Tips")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Gray));
        frame.render_widget(tips_block, chunks[2]);
        frame.render_widget(tips_text, self.inner_area(chunks[2]));
    }

    fn render_practice(&self, frame: &mut Frame, app: &super::app::App, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(4),
                Constraint::Min(10),
                Constraint::Length(4),
            ])
            .split(area);

        let instruction = Paragraph::new(app.practice_state.current_instruction.clone())
            .style(Style::default().fg(Color::Yellow))
            .block(
                Block::default()
                    .title("Instructions")
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::DarkGray)),
            );
        frame.render_widget(instruction, chunks[0]);

        if app.settings_state.hints_enabled && !app.practice_state.hint.is_empty() {
            let hint = Paragraph::new(app.practice_state.hint.clone())
                .style(Style::default().fg(Color::Gray).italic())
                .alignment(Alignment::Center);
            frame.render_widget(hint, Rect::new(chunks[0].x + 1, chunks[0].y + 2, chunks[0].width - 2, 1));
        }

        // Calculate gutter width
        let line_count = app.practice_state.vim_buffer.lines.len();
        let gutter_width = line_count.to_string().len();

        let buffer_content: Vec<Line> = app
            .practice_state
            .vim_buffer
            .lines
            .iter()
            .enumerate()
            .map(|(i, line)| {
                // Line Number Style
                let is_current = i == app.practice_state.vim_buffer.cursor_row;
                let num_style = if is_current {
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::DarkGray)
                };
                
                let line_num = format!("{:>width$} │ ", i + 1, width = gutter_width);
                let num_span = Span::styled(line_num, num_style);

                // Line Content
                let content_spans = if is_current {
                    let col = app.practice_state.vim_buffer.cursor_col;
                    if col < line.len() {
                        let at = line.chars().nth(col).unwrap_or(' ');
                        let before = &line[..col];
                        let after = &line[col + 1..];
                        vec![
                            num_span,
                            Span::raw(before),
                            Span::styled(at.to_string(), Style::default().bg(Color::Yellow).fg(Color::Black)),
                            Span::raw(after),
                        ]
                    } else {
                        vec![
                            num_span,
                            Span::raw(line),
                            Span::styled(" ", Style::default().bg(Color::Yellow)),
                        ]
                    }
                } else {
                    vec![num_span, Span::raw(line)]
                };
                
                Line::from(content_spans)
            })
            .collect();

        let buffer = Paragraph::new(Text::from(buffer_content))
            .block(
                Block::default()
                    .title(format!(" Vim Buffer | Mode: {:?} ", app.practice_state.vim_mode))
                    .borders(Borders::ALL)
                    .style(match app.practice_state.vim_mode {
                        VimMode::Normal => Style::default().fg(Color::Green),
                        VimMode::Insert => Style::default().fg(Color::Blue),
                        VimMode::Visual => Style::default().fg(Color::Yellow),
                        VimMode::Command => Style::default().fg(Color::Magenta),
                        VimMode::OperatorPending(_) => Style::default().fg(Color::LightRed),
                    }),
            );
        frame.render_widget(buffer, chunks[1]);

        let stats_text = format!(
            " Time: {:.1}s | Row: {} | Col: {} | Exercise: {}/{} ",
            app.practice_state.elapsed_time().as_secs_f64(),
            app.practice_state.vim_buffer.cursor_row + 1,
            app.practice_state.vim_buffer.cursor_col + 1,
            app.practice_state.exercise_number,
            app.practice_state.total_exercises
        );

        let stats = Paragraph::new(stats_text)
            .style(Style::default().fg(Color::DarkGray))
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(stats, chunks[2]);
    }

    fn render_progress(&self, frame: &mut Frame, app: &super::app::App, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        let stats_content = vec![
            Line::from(vec![Span::styled(
                "PROGRESS",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD | Modifier::ITALIC),
            )]),
            Line::from(""),
            Line::from(format!("Level: {}", app.progress_state.level)),
            Line::from(format!(
                "XP: {}/{}",
                app.progress_state.xp, app.progress_state.xp_for_next_level
            )),
            Line::from(format!(
                "Progress: {:.0}%",
                app.progress_state.level_progress() * 100.0
            )),
            Line::from(""),
            Line::from("Commands:"),
            Line::from(format!(
                "  Mastered: {}",
                app.progress_state.commands_mastered
            )),
            Line::from(format!(
                "  Learning: {}",
                app.progress_state.commands_learning
            )),
            Line::from(""),
            Line::from("Activity:"),
            Line::from(format!("  Streak: {} days", app.progress_state.streak_days)),
            Line::from(format!(
                "  Sessions: {}",
                app.progress_state.total_sessions
            )),
        ];

        let stats_block = Block::default()
            .title("Stats")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Gray));
        frame.render_widget(stats_block, chunks[0]);
        frame.render_widget(
            Paragraph::new(Text::from(stats_content)),
            self.inner_area(chunks[0]),
        );

        let _right_content = vec![
            Line::from(vec![Span::styled(
                "ACHIEVEMENTS",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD | Modifier::ITALIC),
            )]),
            Line::from(""),
            Line::from(format!(
                "Unlocked: {}/10",
                app.progress_state.achievements_unlocked
            )),
            Line::from(""),
            Line::from("Recent Activity:"),
        ];

        let achievements_block = Block::default()
            .title("Achievements")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Gray));
        frame.render_widget(achievements_block, chunks[1]);

        if app.progress_state.recent_activity.is_empty() {
            let empty_text = Paragraph::new("  No activity yet");
            frame.render_widget(empty_text, self.inner_area(chunks[1]));
        } else {
            for activity in app.progress_state.recent_activity.iter().take(5) {
                let activity_line = Paragraph::new(format!(
                    "  {} +{}",
                    activity.description, activity.xp_gained
                ));
                frame.render_widget(activity_line, self.inner_area(chunks[1]));
            }
        }
    }

    fn render_settings(&self, frame: &mut Frame, app: &super::app::App, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(4),
                Constraint::Length(4),
                Constraint::Length(4),
                Constraint::Length(4),
                Constraint::Min(5),
            ])
            .split(area);

        let hints_text = if app.settings_state.hints_enabled { "On" } else { "Off" };
        let drill_text = format!("{:?} min", app.settings_state.drill_duration);
        let difficulty_text = format!("{:?}", app.settings_state.difficulty);
        let sound_text = if app.settings_state.sound_enabled { "On" } else { "Off" };

        let settings_items = vec![
            ("Hints", hints_text),
            ("Drill Duration", &drill_text),
            ("Difficulty", &difficulty_text),
            ("Sound", sound_text),
        ];

        for (i, (name, value)) in settings_items.iter().enumerate() {
            let item = format!(
                "{} {} {}",
                if i == app.settings_state.selected_index { "►" } else { " " },
                name,
                if i == app.settings_state.selected_index {
                    format!("[ {} ]", value)
                } else {
                    format!("  {}  ", value)
                }
            );

            let style = if i == app.settings_state.selected_index {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let paragraph = Paragraph::new(item)
                .style(style)
                .block(Block::default().borders(Borders::ALL));

            frame.render_widget(paragraph, chunks[i]);
        }

        let instructions = Paragraph::new("Use ↑↓ or j/k to navigate, Space/Enter to toggle, Esc to return to menu")
            .style(Style::default().fg(Color::DarkGray).italic())
            .alignment(Alignment::Center);
        frame.render_widget(instructions, chunks[4]);
    }

    fn render_help(&self, frame: &mut Frame, area: Rect) {
        let help_content = vec![
            Line::from(vec![Span::styled(
                "KEYBOARD SHORTCUTS",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD | Modifier::ITALIC),
            )]),
            Line::from(""),
            Line::from("Main Menu:"),
            Line::from("  ↑↓ / jk / Tab  Navigate menu"),
            Line::from("  Enter           Select option"),
            Line::from("  d               Daily Drill"),
            Line::from("  g               Guided Learning"),
            Line::from("  f               Free Practice"),
            Line::from("  p               Progress"),
            Line::from("  s               Settings"),
            Line::from("  ?               Help"),
            Line::from("  q / Esc         Quit"),
            Line::from(""),
            Line::from("Practice Mode (Vim-style):"),
            Line::from("  h j k l         Move cursor"),
            Line::from("  w b             Word forward/backward"),
            Line::from("  0 $             Line start/end"),
            Line::from("  gg G            File start/end"),
            Line::from("  i               Insert mode"),
            Line::from("  a               Append (insert after)"),
            Line::from("  A               Append line end"),
            Line::from("  o O             New line below/above"),
            Line::from("  x               Delete character"),
            Line::from("  dd              Delete line"),
            Line::from("  yy              Yank line"),
            Line::from("  p P             Paste below/above"),
            Line::from("  u               Undo"),
            Line::from("  v               Visual mode"),
            Line::from("  Esc             Return to Normal mode"),
        ];

        let help = Paragraph::new(Text::from(help_content))
            .block(Block::default().title("Help").borders(Borders::ALL))
            .wrap(Wrap { trim: true });
        frame.render_widget(help, area);
    }

    fn render_guided_learning(&self, frame: &mut Frame, app: &super::app::App, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
            .split(area);

        // --- Left Panel: Levels ---
        let levels: Vec<ListItem> = app.guided_learning_state.levels
            .iter()
            .enumerate()
            .map(|(i, (level, _))| {
                let is_selected = i == app.guided_learning_state.selected_level_index;
                let style = if is_selected {
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };
                let prefix = if is_selected { "► " } else { "  " };
                ListItem::new(format!("{}{:?}", prefix, level)).style(style)
            })
            .collect();
        
        let levels_block = Block::default()
            .title("Levels")
            .borders(Borders::ALL)
            .style(if app.guided_learning_state.active_panel == GuidedLearningPanel::Levels {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::Gray)
            });
            
        let levels_list = List::new(levels)
            .block(levels_block)
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED));
            
        frame.render_widget(levels_list, chunks[0]);

        // --- Right Panel: Exercises ---
        if let Some((_, exercises)) = app.guided_learning_state.levels.get(app.guided_learning_state.selected_level_index) {
             let ex_items: Vec<ListItem> = exercises
                .iter()
                .enumerate()
                .map(|(i, ex)| {
                    let is_selected = i == app.guided_learning_state.selected_exercise_index;
                    let style = if is_selected {
                        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::White)
                    };
                    let prefix = if is_selected { "► " } else { "  " };
                    ListItem::new(format!("{}{}: {}", prefix, ex.id, ex.title)).style(style)
                })
                .collect();

            let ex_block = Block::default()
                .title("Exercises")
                .borders(Borders::ALL)
                .style(if app.guided_learning_state.active_panel == GuidedLearningPanel::Exercises {
                    Style::default().fg(Color::Cyan)
                } else {
                    Style::default().fg(Color::Gray)
                });

            let ex_list = List::new(ex_items).block(ex_block);
            frame.render_widget(ex_list, chunks[1]);
        }
    }

    fn inner_area(&self, area: Rect) -> Rect {
        Rect::new(area.x + 1, area.y + 1, area.width - 2, area.height - 2)
    }
}

fn draw_title_bar(frame: &mut Frame, area: Rect) {
    let title = Line::from(vec![
        Span::styled(
            " VIMURAI ",
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(
            "Master Vim through muscle memory",
            Style::default().fg(Color::DarkGray),
        ),
    ]);

    let title_bar = Block::default()
        .title_alignment(Alignment::Left)
        .borders(Borders::TOP | Borders::BOTTOM)
        .style(Style::default().bg(Color::DarkGray).fg(Color::DarkGray));

    frame.render_widget(title_bar, area);
    frame.render_widget(title, area);
}

fn draw_status_bar(frame: &mut Frame, app: &super::app::App, message: &str) {
    let mode = app.practice_state.vim_mode;
    let mode_color = match mode {
        VimMode::Normal => Color::Green,
        VimMode::Insert => Color::Blue,
        VimMode::Visual => Color::Yellow,
        VimMode::Command => Color::Magenta,
        VimMode::OperatorPending(_) => Color::LightRed,
    };

    let mode_text = format!(" {:?} ", mode).to_uppercase();
    
    // Command buffer or status message
    let center_text = if mode == VimMode::Command {
        format!(":{}", app.practice_state.key_buffer)
    } else if !app.practice_state.key_buffer.is_empty() {
        format!("{} (pending: {})", message, app.practice_state.key_buffer)
    } else {
        message.to_string()
    };

    let cursor_text = format!(
        " Ln {}, Col {} ",
        app.practice_state.vim_buffer.cursor_row + 1,
        app.practice_state.vim_buffer.cursor_col + 1
    );

    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(mode_text.len() as u16),
            Constraint::Min(10),
            Constraint::Length(cursor_text.len() as u16),
        ])
        .split(Rect::new(0, frame.size().height - 2, frame.size().width, 1));

    // Mode block
    frame.render_widget(
        Paragraph::new(mode_text)
            .style(Style::default().bg(mode_color).fg(Color::Black).add_modifier(Modifier::BOLD)),
        layout[0],
    );

    // Center message
    frame.render_widget(
        Paragraph::new(center_text)
            .style(Style::default().bg(Color::DarkGray).fg(Color::White)),
        layout[1],
    );

    // Cursor position
    frame.render_widget(
        Paragraph::new(cursor_text)
            .style(Style::default().bg(mode_color).fg(Color::Black)),
        layout[2],
    );
}

fn draw_help_bar(frame: &mut Frame, hints: &[(&str, &str)]) {
    let help_text: Vec<Span> = hints
        .iter()
        .flat_map(|(key, desc)| {
            vec![
                Span::styled(
                    format!("[{}]", key),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(format!(" {} ", desc), Style::default().fg(Color::DarkGray)),
            ]
        })
        .collect();

    let help_line = Paragraph::new(Line::from(help_text))
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);

    let area = Rect::new(0, frame.size().height - 1, frame.size().width, 1);
    frame.render_widget(help_line, area);
}

fn draw_quit_confirm(frame: &mut Frame) {
    // Create centered modal
    let area = frame.size();
    let modal_width = 50.min(area.width - 4);
    let modal_height = 7;

    let modal_x = (area.width.saturating_sub(modal_width)) / 2;
    let modal_y = (area.height.saturating_sub(modal_height)) / 2;

    let modal_area = Rect::new(modal_x, modal_y, modal_width, modal_height);

    // Clear background with semi-transparent effect (using DarkGray)
    let background = Block::default()
        .style(Style::default().bg(Color::DarkGray));
    frame.render_widget(background, modal_area);

    // Draw the confirmation dialog
    let content = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "Are you sure you want to quit?",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("[Y]", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::styled(" Yes  ", Style::default().fg(Color::White)),
            Span::styled("[N]", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled(" No  ", Style::default().fg(Color::White)),
            Span::styled("[Esc]", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(" Cancel", Style::default().fg(Color::White)),
        ]),
    ];

    let dialog = Paragraph::new(Text::from(content))
        .block(
            Block::default()
                .title(" Confirm Quit ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Red))
                .style(Style::default().bg(Color::Black)),
        )
        .alignment(Alignment::Center);

    frame.render_widget(dialog, modal_area);
}

fn draw_success_overlay(frame: &mut Frame) {
    let area = frame.size();
    let modal_width = 40;
    let modal_height = 5;
    
    let x = (area.width.saturating_sub(modal_width)) / 2;
    let y = (area.height.saturating_sub(modal_height)) / 2;
    let rect = Rect::new(x, y, modal_width, modal_height);

    let block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Green).fg(Color::Black));
    
    let text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("SUCCESS!", Style::default().add_modifier(Modifier::BOLD))
        ]),
        Line::from("Press Enter to continue"),
    ];
    
    let p = Paragraph::new(text)
        .block(block)
        .alignment(Alignment::Center);
        
    frame.render_widget(p, rect);
}
