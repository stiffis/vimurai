//! Pure, responsive Ratatui renderer for the Vimurai terminal dojo.

pub mod mascot;
pub mod theme;

use std::time::Duration;

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Gauge, Paragraph, Wrap},
};

use crate::{
    app::{
        AcademyPanel, App, CompletionCard, HOME_ITEMS, Overlay, PracticeKind, Route,
        SessionSummary, Toast, ToastKind,
    },
    curriculum::{Belt, Exercise},
    editor::{Mode, Position},
    progress::ActivityDay,
    terminal_appearance::ThemeSource,
};

use self::{
    mascot::{coach_line, get_compact_mascot_lines, get_mascot_lines},
    theme::Theme,
};

const MIN_WIDTH: u16 = 60;
const MIN_HEIGHT: u16 = 18;
const WIDE_WIDTH: u16 = 100;
const FULL_CAT_WIDTH: u16 = 126;
const FULL_CAT_HEIGHT: u16 = 28;

/// Render one immutable snapshot. Resizes never become editor actions.
pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();
    if area.width == 0 || area.height == 0 {
        return;
    }
    let theme = Theme::gruvbox(app.terminal_theme, app.settings.high_contrast, app.no_color);
    frame.render_widget(
        Block::default().style(Style::default().fg(theme.text)),
        area,
    );

    if app.route == Route::Boot {
        render_boot(frame, area, app, theme);
    } else if area.width < MIN_WIDTH || area.height < MIN_HEIGHT {
        render_tiny(frame, area, app, theme);
    } else {
        let footer = if area.height >= 23 { 3 } else { 2 };
        let shell = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(1),
                Constraint::Length(footer),
            ])
            .split(area);
        render_header(frame, shell[0], app, theme);
        match app.route {
            Route::Boot => {}
            Route::Home => render_home(frame, shell[1], app, theme),
            Route::Academy => render_academy(frame, shell[1], app, theme),
            Route::Practice => render_practice(frame, shell[1], app, theme, area),
            Route::Progress => render_progress(frame, shell[1], app, theme),
            Route::Settings => render_settings(frame, shell[1], app, theme),
            Route::Help => render_help(frame, shell[1], app, theme),
        }
        render_footer(frame, shell[2], app, theme);
    }

    if let Some(overlay) = app.overlay.as_ref() {
        render_overlay(frame, area, app, overlay, theme);
    }
    if let Some(toast) = app.toast.as_ref() {
        render_toast(frame, area, toast, theme);
    }
}

fn render_boot(frame: &mut Frame, area: Rect, app: &App, theme: Theme) {
    let pulse = if !app.settings.animations || app.tick.is_multiple_of(2) {
        theme.primary
    } else {
        theme.cyan
    };
    let box_area = centered(
        area,
        area.width.saturating_sub(4).clamp(1, 112),
        area.height.saturating_sub(2).clamp(1, 23),
    );
    if area.width >= FULL_CAT_WIDTH && area.height >= FULL_CAT_HEIGHT {
        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(63), Constraint::Percentage(37)])
            .split(box_area);
        render_boot_console(frame, cols[0], app, theme, pulse);
        render_mascot(frame, cols[1], app, theme, true, " KAGE // ONLINE ");
    } else {
        render_boot_console(frame, box_area, app, theme, pulse);
    }
}

fn render_boot_console(frame: &mut Frame, area: Rect, app: &App, theme: Theme, pulse: Color) {
    let mut lines = if area.width >= 66 && area.height >= 15 {
        vec![
            Line::from(Span::styled(
                "██╗   ██╗██╗███╗   ███╗██╗   ██╗██████╗  █████╗ ██╗",
                Style::default().fg(pulse).add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
                "██║   ██║██║████╗ ████║██║   ██║██╔══██╗██╔══██╗██║",
                Style::default().fg(pulse),
            )),
            Line::from(Span::styled(
                "██║   ██║██║██╔████╔██║██║   ██║██████╔╝███████║██║",
                Style::default().fg(pulse),
            )),
            Line::from(Span::styled(
                "╚██╗ ██╔╝██║██║╚██╔╝██║██║   ██║██╔══██╗██╔══██║██║",
                Style::default().fg(pulse),
            )),
            Line::from(Span::styled(
                " ╚████╔╝ ██║██║ ╚═╝ ██║╚██████╔╝██║  ██║██║  ██║██║",
                Style::default().fg(pulse),
            )),
            Line::from(Span::styled(
                "  ╚═══╝  ╚═╝╚═╝     ╚═╝ ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝╚═╝",
                Style::default().fg(pulse),
            )),
        ]
    } else {
        vec![Line::from(Span::styled(
            "VIMURAI",
            Style::default().fg(pulse).add_modifier(Modifier::BOLD),
        ))]
    };
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "// DOJO DE MEMORIA MUSCULAR",
        theme.focused(),
    )));
    lines.push(Line::from(Span::styled(boot_stage(app.tick), theme.dim())));
    if area.height >= 18 {
        lines.push(Line::from(Span::styled(
            format!("{} · fondo heredado · progreso local", theme.label()),
            theme.dim(),
        )));
    }
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(3)])
        .split(area);
    frame.render_widget(
        Paragraph::new(lines)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true })
            .block(panel(" BOOT SEQUENCE ", theme, true)),
        rows[0],
    );
    frame.render_widget(
        Gauge::default()
            .block(panel(" NEURAL LINK ", theme, false))
            .gauge_style(Style::default().fg(pulse))
            .ratio(app.tick.min(7) as f64 / 7.0)
            .label(if app.tick >= 7 {
                "ENLACE LISTO · cualquier tecla"
            } else {
                "sincronizando motions…"
            }),
        rows[1],
    );
}

fn boot_stage(tick: u64) -> &'static str {
    match tick.min(7) {
        0 => "> comprobando terminal…",
        1 => "> cargando gramática modal…",
        2 => "> indexando movimientos…",
        3 => "> montando buffers Unicode…",
        4 => "> despertando a Kage…",
        5 => "> conectando dojo local…",
        6 => "> calibrando memoria muscular…",
        _ => "> acceso concedido_",
    }
}

fn render_header(frame: &mut Frame, area: Rect, app: &App, theme: Theme) {
    let right = format!(
        "LVL {:02}  XP {:04}  RACHA {}d",
        app.profile.level, app.profile.xp, app.profile.streak_days
    );
    let left_width = usize::from(area.width).saturating_sub(right.chars().count() + 4);
    let left = clip(
        &format!(" ◈ VIMURAI // {}", route_name(app.route)),
        left_width,
    );
    let spaces =
        usize::from(area.width).saturating_sub(left.chars().count() + right.chars().count() + 1);
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(left, theme.title()),
            Span::raw(" ".repeat(spaces)),
            Span::styled(right, Style::default().fg(theme.amber)),
        ]))
        .style(Style::default().fg(theme.text))
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(theme.border)),
        ),
        area,
    );
}

fn render_footer(frame: &mut Frame, area: Rect, app: &App, theme: Theme) {
    let keys = route_keys(app.route, app.practice.as_ref().map(|session| session.kind));
    let lines = if area.height >= 3 {
        vec![
            Line::from(vec![
                Span::styled(
                    " CONTROLES ",
                    Style::default()
                        .fg(theme.on_accent)
                        .bg(theme.primary)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(format!(" {keys}"), theme.dim()),
            ]),
            Line::from(vec![
                Span::styled(" // ", theme.focused()),
                Span::styled(
                    app.startup_warning
                        .as_deref()
                        .unwrap_or_else(|| route_status(app)),
                    Style::default().fg(theme.text),
                ),
            ]),
        ]
    } else {
        vec![Line::from(Span::styled(
            clip(keys, usize::from(area.width).saturating_sub(2)),
            theme.dim(),
        ))]
    };
    frame.render_widget(
        Paragraph::new(lines)
            .style(Style::default().fg(theme.text))
            .block(
                Block::default()
                    .borders(Borders::TOP)
                    .border_style(Style::default().fg(theme.border)),
            ),
        area,
    );
}

fn render_tiny(frame: &mut Frame, area: Rect, app: &App, theme: Theme) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(area);
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(" VIMURAI ", theme.selected()),
            Span::styled(format!(" // {}", route_name(app.route)), theme.focused()),
        ]))
        .style(Style::default().fg(theme.text)),
        rows[0],
    );
    let mut lines = vec![Line::from(Span::styled(
        format!(
            "vista segura {}×{} · recomendado 60×18",
            area.width, area.height
        ),
        theme.dim(),
    ))];
    match app.route {
        Route::Home => {
            if let Some(item) =
                HOME_ITEMS.get(app.home_index.min(HOME_ITEMS.len().saturating_sub(1)))
            {
                lines.push(Line::from(Span::styled(
                    format!("> [{}] {}", item.key, item.label),
                    theme.selected(),
                )));
                lines.push(Line::from(item.description));
            }
            lines.push(Line::from(Span::styled(
                format!(
                    "Kage (=^..^=) · nivel {} · {} XP",
                    app.profile.level, app.profile.xp
                ),
                theme.focused(),
            )));
        }
        Route::Academy => {
            let belt = app.selected_belt();
            let (done, total) = app.belt_progress(belt);
            lines.push(Line::from(Span::styled(
                format!("> {} · {done}/{total}", belt.metadata().name),
                theme.focused(),
            )));
            if let Some(index) = app.selected_exercise_index()
                && let Some(exercise) = app.campaign.get(index)
            {
                lines.push(Line::from(exercise.title));
                lines.push(Line::from(Span::styled(exercise.objective, theme.dim())));
            }
        }
        Route::Practice => {
            if let Some(session) = app.practice.as_ref() {
                if let Some(exercise) = app.current_exercise() {
                    lines.push(Line::from(Span::styled(
                        exercise.objective,
                        theme.focused(),
                    )));
                }
                let cursor = session.editor.cursor();
                if let Some(line) = session.editor.lines().get(cursor.row) {
                    lines.push(Line::from(format!(
                        "{:>3} {}",
                        cursor.row.saturating_add(1),
                        line.iter().collect::<String>()
                    )));
                }
                lines.push(Line::from(vec![
                    Span::styled(
                        format!(" {} ", mode_name(session.editor.mode())),
                        mode_badge(theme, session.editor.mode()),
                    ),
                    Span::styled(format!(" {}", session.status), theme.dim()),
                ]));
            }
        }
        Route::Progress => {
            lines.push(Line::from(format!(
                "Nivel {} · {} XP · racha {}d",
                app.profile.level, app.profile.xp, app.profile.streak_days
            )));
            lines.push(Line::from(format!(
                "Precisión {}% · {} sesiones",
                app.profile.accuracy_percent(),
                app.profile.total_sessions
            )));
            lines.push(activity_compact_line(app, theme));
        }
        Route::Settings => {
            let settings = setting_rows(app);
            if let Some((name, value, _)) =
                settings.get(app.settings_index.min(settings.len().saturating_sub(1)))
            {
                lines.push(Line::from(Span::styled(
                    format!("> {name}"),
                    theme.focused(),
                )));
                lines.push(Line::from(value.clone()));
            }
        }
        Route::Help => {
            lines.push(Line::from(format!("Buscar: {}_", app.help_query)));
            for command in app
                .filtered_commands()
                .into_iter()
                .skip(app.help_scroll)
                .take(3)
            {
                lines.push(Line::from(vec![
                    Span::styled(format!("{:>8} ", command.keys), theme.focused()),
                    Span::raw(command.name),
                ]));
            }
        }
        Route::Boot => {}
    }
    frame.render_widget(
        Paragraph::new(lines)
            .block(panel(" TERMINAL COMPACTA ", theme, true))
            .wrap(Wrap { trim: true }),
        rows[1],
    );
    frame.render_widget(
        Paragraph::new(clip(
            route_keys(app.route, app.practice.as_ref().map(|session| session.kind)),
            usize::from(area.width),
        ))
        .style(Style::default().fg(theme.muted)),
        rows[2],
    );
}

fn render_home(frame: &mut Frame, area: Rect, app: &App, theme: Theme) {
    let wide = area.width >= WIDE_WIDTH;
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(if wide {
            [Constraint::Percentage(55), Constraint::Percentage(45)]
        } else {
            [Constraint::Percentage(62), Constraint::Percentage(38)]
        })
        .split(area);
    render_home_menu(frame, cols[0], app, theme);
    if wide {
        let profile_height = if area.height >= 25 { 8 } else { 6 };
        let right = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(profile_height), Constraint::Min(1)])
            .split(cols[1]);
        render_profile(frame, right[0], app, theme);
        let full = frame.area().width >= FULL_CAT_WIDTH && frame.area().height >= FULL_CAT_HEIGHT;
        render_mascot(frame, right[1], app, theme, full, " KAGE // SENSEI ");
    } else {
        render_mascot(frame, cols[1], app, theme, false, " KAGE // ONLINE ");
    }
}

fn render_home_menu(frame: &mut Frame, area: Rect, app: &App, theme: Theme) {
    let two_lines = area.height >= 16;
    let per_item = if two_lines { 2 } else { 1 };
    let visible = (usize::from(area.height.saturating_sub(2)) / per_item)
        .max(1)
        .min(HOME_ITEMS.len());
    let selected = app.home_index.min(HOME_ITEMS.len().saturating_sub(1));
    let start = scroll_start(selected, HOME_ITEMS.len(), visible);
    let mut lines = Vec::new();
    for (index, item) in HOME_ITEMS.iter().enumerate().skip(start).take(visible) {
        let active = index == selected;
        lines.push(Line::from(vec![
            Span::styled(
                if active { " >> " } else { "    " },
                if active { theme.title() } else { theme.dim() },
            ),
            Span::styled(
                format!(" {} ", item.key),
                if active {
                    theme.selected()
                } else {
                    theme.focused()
                },
            ),
            Span::styled(
                format!("  {}", item.label),
                if active {
                    theme.title()
                } else {
                    Style::default().fg(theme.text)
                },
            ),
        ]));
        if two_lines {
            lines.push(Line::from(Span::styled(
                format!("        {}", item.description),
                theme.dim(),
            )));
        }
    }
    frame.render_widget(
        Paragraph::new(lines)
            .block(panel(" NODOS DE ACCESO ", theme, true))
            .wrap(Wrap { trim: true }),
        area,
    );
}

fn render_profile(frame: &mut Frame, area: Rect, app: &App, theme: Theme) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(2), Constraint::Length(3)])
        .split(area);
    frame.render_widget(
        Paragraph::new(vec![
            Line::from(vec![
                Span::styled(
                    format!(" NIVEL {:02} ", app.profile.level),
                    theme.selected(),
                ),
                Span::styled(
                    format!("  racha {}d", app.profile.streak_days),
                    Style::default().fg(theme.amber),
                ),
            ]),
            Line::from(format!(
                " {} sesiones · {} min · precisión {}%",
                app.profile.total_sessions,
                app.profile.total_practice_seconds / 60,
                app.profile.accuracy_percent()
            )),
        ])
        .block(panel(" PERFIL LOCAL ", theme, false)),
        rows[0],
    );
    frame.render_widget(
        Gauge::default()
            .block(panel(" SIGUIENTE NIVEL ", theme, false))
            .gauge_style(Style::default().fg(theme.primary))
            .ratio((app.profile.xp_into_level() as f64 / 250.0).clamp(0.0, 1.0))
            .label(format!("{}/250 XP", app.profile.xp_into_level())),
        rows[1],
    );
}

fn render_mascot(frame: &mut Frame, area: Rect, app: &App, theme: Theme, full: bool, title: &str) {
    // Both explicit ASCII mode and NO_COLOR avoid truecolor pixel art. The
    // colored variants are remapped to the active Gruvbox palette.
    let mut lines = if app.force_ascii || app.no_color {
        vec![
            Line::from(Span::styled("   /|__|", theme.title())),
            Line::from(Span::styled("  ( o.o )", theme.focused())),
            Line::from(Span::styled("   > ^ <", Style::default().fg(theme.amber))),
        ]
    } else if full {
        get_mascot_lines(&app.mascot_state, theme)
    } else {
        get_compact_mascot_lines(app.mascot_state, app.tick, theme)
    };
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        clip(
            coach_line(app.mascot_state),
            usize::from(area.width.saturating_sub(2)),
        ),
        theme.dim(),
    )));
    frame.render_widget(
        Paragraph::new(lines)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true })
            .block(panel(title, theme, false)),
        area,
    );
}

fn render_academy(frame: &mut Frame, area: Rect, app: &App, theme: Theme) {
    if area.width >= WIDE_WIDTH {
        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(29),
                Constraint::Length(39),
                Constraint::Min(20),
            ])
            .split(area);
        render_belts(frame, cols[0], app, theme);
        render_exercises(frame, cols[1], app, theme);
        render_exercise_detail(frame, cols[2], app, theme);
    } else {
        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(38), Constraint::Percentage(62)])
            .split(area);
        render_belts(frame, cols[0], app, theme);
        let right = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
            .split(cols[1]);
        render_exercises(frame, right[0], app, theme);
        render_exercise_detail(frame, right[1], app, theme);
    }
}

fn render_belts(frame: &mut Frame, area: Rect, app: &App, theme: Theme) {
    let focused = app.academy_panel == AcademyPanel::Belts;
    let selected = app.belt_index.min(Belt::all().len().saturating_sub(1));
    let dense = area.height < 16;
    let mut lines = Vec::new();
    for (index, belt) in Belt::all().iter().copied().enumerate() {
        let (done, total) = app.belt_progress(belt);
        let unlocked = app.belt_unlocked(belt);
        let active = index == selected;
        let state = if unlocked {
            if done == total && total > 0 {
                "✓"
            } else {
                "◇"
            }
        } else if app.force_ascii {
            "X"
        } else {
            "🔒"
        };
        let progress = if dense {
            format!(" {done}/{total}")
        } else {
            String::new()
        };
        lines.push(Line::from(vec![
            Span::styled(
                if active { "> " } else { "  " },
                if active { theme.title() } else { theme.dim() },
            ),
            Span::styled(
                format!(
                    "{state} {}",
                    clip(
                        belt.metadata().name,
                        usize::from(area.width).saturating_sub(if dense { 12 } else { 7 })
                    )
                ),
                Style::default()
                    .fg(if unlocked {
                        belt_color(app, theme, belt)
                    } else {
                        theme.muted
                    })
                    .add_modifier(if active {
                        Modifier::BOLD
                    } else {
                        Modifier::empty()
                    }),
            ),
            Span::styled(progress, theme.dim()),
        ]));
        if !dense {
            lines.push(Line::from(Span::styled(
                format!(
                    "    {} {done}/{total}",
                    mini_bar(done, total, 7, app.force_ascii)
                ),
                theme.dim(),
            )));
        }
    }
    frame.render_widget(
        Paragraph::new(lines).block(panel(" CINTURONES ", theme, focused)),
        area,
    );
}

fn render_exercises(frame: &mut Frame, area: Rect, app: &App, theme: Theme) {
    let focused = app.academy_panel == AcademyPanel::Exercises;
    let indices = app.selected_belt_exercise_indices();
    let selected = app
        .exercise_index_in_belt
        .min(indices.len().saturating_sub(1));
    let visible = usize::from(area.height.saturating_sub(2)).max(1);
    let start = scroll_start(selected, indices.len(), visible);
    let unlocked = app.belt_unlocked(app.selected_belt());
    let mut lines = Vec::new();
    for (local, campaign) in indices
        .iter()
        .copied()
        .enumerate()
        .skip(start)
        .take(visible)
    {
        let Some(exercise) = app.campaign.get(campaign) else {
            continue;
        };
        let record = app.records.get(exercise.id);
        let done = record.is_some_and(|record| record.completions > 0);
        let stars = record.map_or(0, |record| record.stars);
        let active = local == selected;
        let state = if !unlocked {
            if app.force_ascii { "[X]" } else { "🔒" }
        } else if done {
            if app.force_ascii { "[+]" } else { "◆" }
        } else if app.force_ascii {
            "[ ]"
        } else {
            "◇"
        };
        lines.push(Line::from(vec![
            Span::styled(
                if active { "> " } else { "  " },
                if active { theme.title() } else { theme.dim() },
            ),
            Span::styled(
                format!("{state} "),
                if unlocked {
                    theme.focused()
                } else {
                    theme.dim()
                },
            ),
            Span::styled(
                pad(exercise.title, usize::from(area.width).saturating_sub(13)),
                if active {
                    theme.title()
                } else {
                    Style::default().fg(theme.text)
                },
            ),
            Span::styled(
                stars_text(stars, app.force_ascii),
                Style::default().fg(theme.amber),
            ),
        ]));
    }
    if lines.is_empty() {
        lines.push(Line::from(Span::styled("Sin ejercicios.", theme.dim())));
    }
    frame.render_widget(
        Paragraph::new(lines).block(panel(" EJERCICIOS ", theme, focused)),
        area,
    );
}

fn render_exercise_detail(frame: &mut Frame, area: Rect, app: &App, theme: Theme) {
    let Some(index) = app.selected_exercise_index() else {
        frame.render_widget(
            Paragraph::new("Selecciona un cinturón.").block(panel(" DOSSIER ", theme, false)),
            area,
        );
        return;
    };
    let Some(exercise) = app.campaign.get(index) else {
        return;
    };
    let unlocked = app.belt_unlocked(exercise.belt);
    if area.height < 12 {
        let width = usize::from(area.width.saturating_sub(2));
        let action = if unlocked {
            "Enter: entrar al dojo"
        } else {
            "Bloqueado: completa el cinturón anterior"
        };
        frame.render_widget(
            Paragraph::new(vec![
                Line::from(Span::styled(clip(exercise.title, width), theme.title())),
                Line::from(clip(exercise.objective, width)),
                Line::from(Span::styled(
                    clip(&goal_summary(exercise), width),
                    theme.dim(),
                )),
                Line::from(Span::styled(
                    clip(action, width),
                    if unlocked {
                        theme.focused()
                    } else {
                        theme.error()
                    },
                )),
            ])
            .block(panel(" DOSSIER ", theme, false))
            .wrap(Wrap { trim: true }),
            area,
        );
        return;
    }
    let mut lines = vec![
        Line::from(Span::styled(exercise.title, theme.title())),
        Line::from(vec![
            Span::styled(format!(" {} ", exercise.id), theme.selected()),
            Span::styled(
                format!(
                    " ~{}s · óptimo {}",
                    exercise.estimated_secs, exercise.optimal_actions
                ),
                theme.dim(),
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled("MISIÓN", theme.focused())),
        Line::from(exercise.objective),
        Line::from(""),
        Line::from(Span::styled("CONTEXTO", theme.focused())),
        Line::from(Span::styled(exercise.context, theme.dim())),
    ];
    if area.height >= 16 {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("META  ", Style::default().fg(theme.amber)),
            Span::raw(goal_summary(exercise)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("SKILLS  ", Style::default().fg(theme.violet)),
            Span::raw(exercise.skills.join(" · ")),
        ]));
    }
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        if unlocked {
            "Enter: entrar al dojo"
        } else {
            "ACCESO DENEGADO // completa el cinturón anterior"
        },
        if unlocked {
            theme.focused()
        } else {
            theme.error()
        },
    )));
    frame.render_widget(
        Paragraph::new(lines)
            .block(panel(" DOSSIER DE MISIÓN ", theme, false))
            .wrap(Wrap { trim: true }),
        area,
    );
}

fn render_practice(frame: &mut Frame, area: Rect, app: &App, theme: Theme, screen: Rect) {
    let Some(session) = app.practice.as_ref() else {
        frame.render_widget(
            Paragraph::new("Buffer desconectado. Pulsa F2 para volver.")
                .alignment(Alignment::Center)
                .block(panel(" PRACTICE ", theme, true)),
            area,
        );
        return;
    };
    let show_side = area.width >= 78 && area.height >= 15;
    let (main, side) = if show_side {
        let full_cat = screen.width >= FULL_CAT_WIDTH && screen.height >= FULL_CAT_HEIGHT;
        let side_width = if full_cat {
            42
        } else if area.width >= WIDE_WIDTH {
            36
        } else {
            25
        };
        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(30), Constraint::Length(side_width)])
            .split(area);
        (cols[0], Some(cols[1]))
    } else {
        (area, None)
    };
    let objective_height = if main.height >= 20 { 7 } else { 4 };
    let status_height = if main.height >= 11 { 4 } else { 3 };
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(objective_height),
            Constraint::Min(3),
            Constraint::Length(status_height),
        ])
        .split(main);
    render_objective(frame, rows[0], app, theme);
    render_editor(frame, rows[1], app, theme);
    render_practice_status(frame, rows[2], app, theme, side.is_none());
    if let Some(side) = side {
        let full = screen.width >= FULL_CAT_WIDTH && screen.height >= FULL_CAT_HEIGHT;
        let cat_height = if full {
            16.min(side.height.saturating_sub(5).max(1))
        } else {
            12.min(side.height.saturating_sub(5).max(1))
        };
        let right = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(cat_height), Constraint::Min(3)])
            .split(side);
        render_mascot(frame, right[0], app, theme, full, " KAGE // COACH ");
        render_telemetry(frame, right[1], app, theme);
    }
    let _ = session;
}

fn render_objective(frame: &mut Frame, area: Rect, app: &App, theme: Theme) {
    let Some(session) = app.practice.as_ref() else {
        return;
    };
    let kind = match session.kind {
        PracticeKind::Daily => "DAILY DRILL",
        PracticeKind::Guided => "ACADEMIA",
        PracticeKind::Free => "SANDBOX // NO SCORE",
    };
    let counter = if session.kind == PracticeKind::Daily {
        format!(
            " nodo {}/{}",
            session
                .queue_position
                .saturating_add(1)
                .min(session.queue.len().max(1)),
            session.queue.len().max(1)
        )
    } else {
        String::new()
    };
    let mut lines = vec![Line::from(vec![
        Span::styled(format!(" {kind} "), theme.selected()),
        Span::styled(counter, Style::default().fg(theme.amber)),
    ])];
    if let Some(exercise) = app.current_exercise() {
        if area.height <= 4 {
            lines.push(Line::from(exercise.objective));
        } else {
            lines.push(Line::from(Span::styled(exercise.title, theme.title())));
            lines.push(Line::from(exercise.objective));
        }
        if area.height >= 6 {
            lines.push(Line::from(Span::styled(
                if session.show_hint {
                    format!("PISTA // {}", exercise.hint)
                } else {
                    "F1 revela una pista · piensa en intención".to_owned()
                },
                if session.show_hint {
                    Style::default().fg(theme.amber)
                } else {
                    theme.dim()
                },
            )));
        }
    } else {
        lines.push(Line::from(
            "Explora motions sobre código real sin alterar tu puntuación.",
        ));
        lines.push(Line::from(Span::styled(
            "F6 rota snippets · F5 reinicia",
            theme.dim(),
        )));
    }
    frame.render_widget(
        Paragraph::new(lines)
            .block(panel(" OBJETIVO ", theme, true))
            .wrap(Wrap { trim: true }),
        area,
    );
}

fn render_editor(frame: &mut Frame, area: Rect, app: &App, theme: Theme) {
    let Some(session) = app.practice.as_ref() else {
        return;
    };
    let editor = &session.editor;
    let pending = editor.pending_display();
    let title = if pending.is_empty() {
        format!(" BUFFER // {} ", mode_name(editor.mode()))
    } else {
        format!(" BUFFER // {} // {}_ ", mode_name(editor.mode()), pending)
    };
    let block = panel(&title, theme, true);
    let inner = block.inner(area);
    frame.render_widget(block, area);
    if inner.width == 0 || inner.height == 0 {
        return;
    }

    let buffer = editor.lines();
    let cursor = editor.cursor();
    let mode = editor.mode();
    let selection = editor.selection();
    let target = app
        .current_exercise()
        .and_then(|exercise| exercise.goal.expected_position());
    let visible = usize::from(inner.height);
    let top = session
        .viewport_top
        .min(buffer.len().saturating_sub(visible));
    let number_width = digits(buffer.len().max(1)).max(2);
    let gutter = number_width.saturating_add(3);
    let text_width = usize::from(inner.width).saturating_sub(gutter).max(1);
    let left = cursor.col.saturating_sub(text_width.saturating_sub(4));
    let mut lines = Vec::with_capacity(visible);
    for row in top..top.saturating_add(visible) {
        let Some(line) = buffer.get(row) else {
            lines.push(Line::from(Span::styled("~", theme.dim())));
            continue;
        };
        let marker = if row == cursor.row {
            ">"
        } else if target.is_some_and(|position| position.row == row) {
            if app.force_ascii { "|" } else { "│" }
        } else {
            " "
        };
        let mut spans = vec![Span::styled(
            format!("{marker}{:>number_width$} ", row.saturating_add(1)),
            if row == cursor.row {
                Style::default().fg(theme.cyan).add_modifier(Modifier::BOLD)
            } else {
                theme.dim()
            },
        )];
        if left > 0 {
            spans.push(Span::styled(
                if app.force_ascii { "<" } else { "‹" },
                theme.dim(),
            ));
        }
        let prefix = usize::from(left > 0);
        let take = text_width.saturating_sub(prefix);
        let source = line.iter().collect::<String>();
        let comment = source.trim_start().starts_with("//") || source.trim_start().starts_with('#');
        for (col, character) in line.iter().copied().enumerate().skip(left).take(take) {
            let position = Position { row, col };
            let mut style = if comment {
                theme.dim()
            } else if character.is_ascii_digit() {
                Style::default().fg(theme.violet)
            } else if character.is_ascii_punctuation() {
                Style::default().fg(theme.cyan)
            } else {
                Style::default().fg(theme.text)
            };
            if selection.is_some_and(|range| selected_cell(position, range, mode)) {
                style = style.fg(theme.on_accent).bg(theme.violet);
            }
            if target == Some(position) {
                style = theme.target();
            }
            if cursor == position {
                style = cursor_style(theme, mode);
            }
            spans.push(Span::styled(character.to_string(), style));
        }
        if row == cursor.row
            && cursor.col >= line.len()
            && cursor.col >= left
            && cursor.col < left.saturating_add(take)
        {
            spans.push(Span::styled(" ", cursor_style(theme, mode)));
        }
        if line.len().saturating_sub(left) > take {
            spans.push(Span::styled(
                if app.force_ascii { ">" } else { "…" },
                theme.dim(),
            ));
        }
        lines.push(Line::from(spans));
    }
    frame.render_widget(Paragraph::new(lines), inner);
}

fn selected_cell(position: Position, (a, b): (Position, Position), mode: Mode) -> bool {
    let (first, last) = if (a.row, a.col) <= (b.row, b.col) {
        (a, b)
    } else {
        (b, a)
    };
    if mode == Mode::VisualLine {
        position.row >= first.row && position.row <= last.row
    } else {
        (position.row, position.col) >= (first.row, first.col)
            && (position.row, position.col) <= (last.row, last.col)
    }
}

fn cursor_style(theme: Theme, mode: Mode) -> Style {
    let color = match mode {
        Mode::Normal => theme.primary,
        Mode::Insert => theme.cyan,
        Mode::VisualChar | Mode::VisualLine => theme.violet,
        Mode::Command | Mode::Search => theme.amber,
    };
    Style::default()
        .fg(theme.on_accent)
        .bg(color)
        .add_modifier(Modifier::BOLD)
}

fn render_practice_status(
    frame: &mut Frame,
    area: Rect,
    app: &App,
    theme: Theme,
    include_kage: bool,
) {
    let Some(session) = app.practice.as_ref() else {
        return;
    };
    let trace = session
        .trace
        .iter()
        .rev()
        .take(9)
        .rev()
        .cloned()
        .collect::<Vec<_>>()
        .join(" ");
    let pending = session.editor.pending_display();
    let inner_width = usize::from(area.width.saturating_sub(2));
    let mode_width = mode_name(session.editor.mode())
        .chars()
        .count()
        .saturating_add(4);
    let status = clip(
        &session.status,
        inner_width.saturating_sub(mode_width).max(1),
    );
    let mut lines = vec![
        Line::from(vec![
            Span::styled(
                format!(" {} ", mode_name(session.editor.mode())),
                mode_badge(theme, session.editor.mode()),
            ),
            Span::styled(format!("  {status}"), Style::default().fg(theme.text)),
        ]),
        Line::from(Span::styled(
            clip(
                &if pending.is_empty() {
                    format!(
                        "keys: {}",
                        if trace.is_empty() {
                            "esperando señal…"
                        } else {
                            &trace
                        }
                    )
                } else {
                    format!("keys: {trace}  // pending {pending}_")
                },
                inner_width,
            ),
            theme.dim(),
        )),
    ];
    if include_kage && area.height >= 4 {
        lines.push(Line::from(Span::styled(
            "KAGE (=^..^=)  piensa en verbo + movimiento",
            theme.focused(),
        )));
    }
    frame.render_widget(
        Paragraph::new(lines)
            .block(panel(" SIGNAL / KEYCAST ", theme, false))
            .wrap(Wrap { trim: true }),
        area,
    );
}

fn render_telemetry(frame: &mut Frame, area: Rect, app: &App, theme: Theme) {
    let Some(session) = app.practice.as_ref() else {
        return;
    };
    let register = session.editor.register().replace(char::from(10), "↵");
    let pending = session.editor.pending_display();
    let mut lines = vec![
        Line::from(vec![
            Span::styled("ACC ", theme.dim()),
            Span::styled(session.semantic_actions.to_string(), theme.focused()),
            Span::styled("  KEY ", theme.dim()),
            Span::styled(session.keystrokes.to_string(), theme.focused()),
            Span::styled("  ERR ", theme.dim()),
            Span::styled(
                session.mistakes.to_string(),
                if session.mistakes == 0 {
                    theme.focused()
                } else {
                    theme.error()
                },
            ),
        ]),
        Line::from(vec![
            Span::styled("HINT ", theme.dim()),
            Span::styled(session.hints.to_string(), Style::default().fg(theme.amber)),
            Span::styled("  TIME ", theme.dim()),
            Span::raw(format_duration(session.exercise_started_at.elapsed())),
        ]),
        Line::from(vec![
            Span::styled("REG ", theme.dim()),
            Span::styled(
                if register.is_empty() {
                    "∅".to_owned()
                } else {
                    clip(&register, 18)
                },
                Style::default().fg(theme.violet),
            ),
            Span::styled(
                if session.editor.register_is_linewise() {
                    " [line]"
                } else {
                    ""
                },
                theme.dim(),
            ),
        ]),
    ];
    if !pending.is_empty() {
        lines.push(Line::from(Span::styled(
            format!("PENDING {pending}_"),
            Style::default()
                .fg(theme.amber)
                .add_modifier(Modifier::BOLD),
        )));
    }
    if let Some(exercise) = app.current_exercise()
        && session.show_hint
    {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            exercise.hint,
            Style::default().fg(theme.amber),
        )));
    }
    frame.render_widget(
        Paragraph::new(lines)
            .block(panel(" TELEMETRÍA ", theme, false))
            .wrap(Wrap { trim: true }),
        area,
    );
}

fn render_progress(frame: &mut Frame, area: Rect, app: &App, theme: Theme) {
    let wide = area.width >= WIDE_WIDTH && area.height >= 22;
    if wide {
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(4),
                Constraint::Length(10),
                Constraint::Min(3),
            ])
            .split(area);
        render_xp(frame, rows[0], app, theme);
        render_stats(frame, rows[1], app, theme);
        render_heatmap(frame, rows[2], app, theme, false);
        render_achievements(frame, rows[3], app, theme);
    } else {
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(4),
                Constraint::Min(4),
            ])
            .split(area);
        render_xp(frame, rows[0], app, theme);
        render_stats_compact(frame, rows[1], app, theme);
        let bottom = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
            .split(rows[2]);
        render_heatmap(frame, bottom[0], app, theme, true);
        render_achievements(frame, bottom[1], app, theme);
    }
}

fn render_xp(frame: &mut Frame, area: Rect, app: &App, theme: Theme) {
    let current = app.profile.xp_into_level();
    frame.render_widget(
        Gauge::default()
            .block(panel(
                &format!(" NIVEL {:02} // ASCENSO ", app.profile.level),
                theme,
                true,
            ))
            .gauge_style(
                Style::default()
                    .fg(theme.primary)
                    .add_modifier(Modifier::BOLD),
            )
            .ratio((current as f64 / 250.0).clamp(0.0, 1.0))
            .label(format!(
                "{} XP TOTAL · {current}/250 PARA NIVEL {}",
                app.profile.xp,
                app.profile.level.saturating_add(1)
            )),
        area,
    );
}

fn render_stats(frame: &mut Frame, area: Rect, app: &App, theme: Theme) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(area);
    let cards = [
        (
            "RACHA",
            format!("{} días", app.profile.streak_days),
            format!("récord {}d", app.profile.best_streak),
        ),
        (
            "PRECISIÓN",
            format!("{}%", app.profile.accuracy_percent()),
            format!("{} acciones", app.profile.total_actions),
        ),
        (
            "DOMINIO",
            app.profile.commands_mastered.to_string(),
            format!("{} aprendiendo", app.profile.commands_learning),
        ),
        (
            "PRÁCTICA",
            format!("{} min", app.profile.total_practice_seconds / 60),
            format!("{} sesiones", app.profile.total_sessions),
        ),
    ];
    for (rect, (title, value, detail)) in cols.iter().copied().zip(cards) {
        frame.render_widget(
            Paragraph::new(vec![
                Line::from(Span::styled(value, theme.title())),
                Line::from(Span::styled(detail, theme.dim())),
            ])
            .alignment(Alignment::Center)
            .block(panel(&format!(" {title} "), theme, false)),
            rect,
        );
    }
}

fn render_stats_compact(frame: &mut Frame, area: Rect, app: &App, theme: Theme) {
    frame.render_widget(
        Paragraph::new(vec![
            Line::from(vec![
                Span::styled(
                    format!(" {}d racha ", app.profile.streak_days),
                    theme.selected(),
                ),
                Span::styled(
                    format!(" {}% precisión", app.profile.accuracy_percent()),
                    theme.focused(),
                ),
            ]),
            Line::from(format!(
                " {} sesiones · {} min · {} dominados / {} aprendiendo",
                app.profile.total_sessions,
                app.profile.total_practice_seconds / 60,
                app.profile.commands_mastered,
                app.profile.commands_learning
            )),
        ])
        .block(panel(" TELEMETRÍA GLOBAL ", theme, false)),
        area,
    );
}

fn render_heatmap(frame: &mut Frame, area: Rect, app: &App, theme: Theme, compact: bool) {
    let activity = trailing_activity(app);
    let mut lines = Vec::new();
    if compact {
        let mut spans = vec![Span::styled("35d ", theme.dim())];
        for day in &activity {
            spans.push(Span::styled(
                if app.force_ascii {
                    activity_char(day)
                } else {
                    "■"
                },
                activity_style(day, theme),
            ));
        }
        lines.push(Line::from(spans));
        lines.push(Line::from(Span::styled(
            "menos ··· más  → hoy",
            theme.dim(),
        )));
    } else {
        lines.push(Line::from(Span::styled(
            "       −35d       −28d       −21d       −14d        −7d",
            theme.dim(),
        )));
        let labels = ["LU", "MA", "MI", "JU", "VI", "SÁ", "DO"];
        for (weekday, label) in labels.iter().enumerate() {
            let mut spans = vec![Span::styled(format!(" {label:>2}  "), theme.dim())];
            for week in 0usize..5 {
                let index = week.saturating_mul(7).saturating_add(weekday);
                if let Some(day) = activity.get(index) {
                    spans.push(Span::styled(
                        if app.force_ascii {
                            format!(" {} ", activity_char(day))
                        } else {
                            " ██ ".to_owned()
                        },
                        activity_style(day, theme),
                    ));
                }
            }
            lines.push(Line::from(spans));
        }
    }
    frame.render_widget(
        Paragraph::new(lines).block(panel(" ACTIVIDAD // 35 DÍAS ", theme, false)),
        area,
    );
}

fn render_achievements(frame: &mut Frame, area: Rect, app: &App, theme: Theme) {
    const ALL: [(&str, &str); 4] = [
        ("first_steps", "Primer corte"),
        ("perfect_form", "Forma perfecta"),
        ("first_session", "Ritual completo"),
        ("week_warrior", "Guerrero semanal"),
    ];
    let inner_height = usize::from(area.height.saturating_sub(2)).max(1);
    let achievement = |id: &str, name: &'static str| {
        let unlocked = app.achievements.iter().any(|(value, _)| value == id);
        vec![
            Span::styled(
                if unlocked { "◆ " } else { "◇ " },
                if unlocked {
                    Style::default().fg(theme.amber)
                } else {
                    theme.dim()
                },
            ),
            Span::styled(
                name,
                if unlocked {
                    Style::default().fg(theme.text)
                } else {
                    theme.dim()
                },
            ),
        ]
    };
    let lines = if area.width >= 60 && inner_height < ALL.len() {
        ALL.chunks(2)
            .take(inner_height)
            .map(|pair| {
                let mut spans = Vec::new();
                for (index, (id, name)) in pair.iter().copied().enumerate() {
                    if index > 0 {
                        spans.push(Span::raw("    "));
                    }
                    spans.extend(achievement(id, name));
                }
                Line::from(spans)
            })
            .collect::<Vec<_>>()
    } else {
        ALL.into_iter()
            .take(inner_height)
            .map(|(id, name)| Line::from(achievement(id, name)))
            .collect::<Vec<_>>()
    };
    frame.render_widget(
        Paragraph::new(lines).block(panel(" LOGROS ", theme, false)),
        area,
    );
}

fn render_settings(frame: &mut Frame, area: Rect, app: &App, theme: Theme) {
    let columns = (area.width >= WIDE_WIDTH).then(|| {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(58), Constraint::Percentage(42)])
            .split(area)
    });
    let list_area = columns.as_ref().map_or(area, |columns| columns[0]);
    let settings = setting_rows(app);
    let selected = app.settings_index.min(settings.len().saturating_sub(1));
    let visible = usize::from(list_area.height.saturating_sub(2)).max(1);
    let start = scroll_start(selected, settings.len(), visible);
    let mut lines = Vec::new();
    for (index, (name, value, _)) in settings.iter().enumerate().skip(start).take(visible) {
        let active = index == selected;
        let name_width = usize::from(list_area.width)
            .saturating_sub(value.chars().count() + 9)
            .max(1);
        lines.push(Line::from(vec![
            Span::styled(
                if active { " > " } else { "   " },
                if active { theme.title() } else { theme.dim() },
            ),
            Span::styled(
                pad(name, name_width),
                if active {
                    theme.focused()
                } else {
                    Style::default().fg(theme.text)
                },
            ),
            Span::styled(
                format!(" {value} "),
                if active {
                    theme.selected()
                } else {
                    Style::default().fg(theme.amber)
                },
            ),
        ]));
    }
    frame.render_widget(
        Paragraph::new(lines).block(panel(
            &format!(
                " CONFIGURACIÓN // {} {} ",
                theme.label(),
                theme_source_label(app.theme_source)
            ),
            theme,
            true,
        )),
        list_area,
    );
    if let Some(columns) = columns {
        let (name, value, description) = &settings[selected];
        frame.render_widget(
            Paragraph::new(vec![
                Line::from(Span::styled(name.clone(), theme.title())),
                Line::from(Span::styled(
                    value.clone(),
                    Style::default().fg(theme.amber),
                )),
                Line::from(""),
                Line::from(description.clone()),
                Line::from(""),
                Line::from(Span::styled("h/l cambia · Enter confirma", theme.dim())),
                Line::from(Span::styled("Persistencia SQLite local.", theme.dim())),
            ])
            .block(panel(" INSPECTOR ", theme, false))
            .wrap(Wrap { trim: true }),
            columns[1],
        );
    }
}

fn setting_rows(app: &App) -> Vec<(String, String, String)> {
    let flag = |value| if value { "ON" } else { "OFF" }.to_owned();
    let difficulty = match app.settings.difficulty {
        0 => "RELAJADO",
        2 => "ESTRICTO",
        _ => "EQUILIBRADO",
    };
    vec![
        (
            "Pistas".into(),
            flag(app.settings.hints),
            "Guía contextual con F1 durante un reto.".into(),
        ),
        (
            "Animaciones".into(),
            flag(app.settings.animations),
            "Pulsos sutiles y secuencia de arranque.".into(),
        ),
        (
            "Alto contraste".into(),
            flag(app.settings.high_contrast),
            "Refuerza bordes y estados.".into(),
        ),
        (
            "Sonido".into(),
            flag(app.settings.sound),
            "Feedback sonoro opcional.".into(),
        ),
        (
            "Daily Drill".into(),
            format!("{} MIN", app.settings.drill_minutes),
            "Cola finita de 3, 5 o 10 minutos.".into(),
        ),
        (
            "Dificultad".into(),
            difficulty.into(),
            "Rigor frente a la ruta óptima.".into(),
        ),
        (
            "Reiniciar progreso".into(),
            "PELIGRO".into(),
            "Borra XP, actividad, revisiones y logros.".into(),
        ),
        (
            "Volver al dojo".into(),
            "ENTER".into(),
            "Regresa al nodo principal.".into(),
        ),
    ]
}

fn render_help(frame: &mut Frame, area: Rect, app: &App, theme: Theme) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(1)])
        .split(area);
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(
                " / ",
                if app.help_searching {
                    theme.selected()
                } else {
                    theme.focused()
                },
            ),
            Span::styled(
                if app.help_query.is_empty() {
                    "buscar por tecla, nombre o intención".to_owned()
                } else {
                    app.help_query.clone()
                },
                Style::default().fg(theme.text),
            ),
            Span::styled(
                if app.help_searching {
                    "_  ESCRIBIENDO"
                } else {
                    "  / para editar"
                },
                theme.dim(),
            ),
        ]))
        .block(panel(" ÍNDICE DE MOTIONS ", theme, app.help_searching)),
        rows[0],
    );
    let commands = app.filtered_commands();
    let visible = usize::from(rows[1].height.saturating_sub(3)).max(1);
    let start = app.help_scroll.min(commands.len().saturating_sub(visible));
    let width = usize::from(rows[1].width.saturating_sub(2));
    let key_width = if width >= 80 { 16 } else { 11 };
    let name_width = if width >= 80 { 24 } else { 17 };
    let description_width = width.saturating_sub(key_width + name_width + 3);
    let mut lines = vec![Line::from(vec![
        Span::styled(pad("TECLA", key_width), theme.title()),
        Span::styled(pad("COMANDO", name_width), theme.title()),
        Span::styled("DESCRIPCIÓN", theme.title()),
    ])];
    for command in commands.iter().copied().skip(start).take(visible) {
        let unlocked = app.belt_unlocked(command.introduced_in);
        lines.push(Line::from(vec![
            Span::styled(
                pad(command.keys, key_width),
                if unlocked {
                    theme.focused()
                } else {
                    theme.dim()
                },
            ),
            Span::styled(
                pad(command.name, name_width),
                if unlocked {
                    Style::default().fg(theme.text)
                } else {
                    theme.dim()
                },
            ),
            Span::styled(clip(command.description, description_width), theme.dim()),
        ]));
    }
    if lines.len() == 1 {
        lines.push(Line::from(Span::styled(
            "Sin coincidencias.",
            theme.error(),
        )));
    }
    frame.render_widget(
        Paragraph::new(lines).block(panel(
            &format!(" REFERENCIA // {} RESULTADOS ", commands.len()),
            theme,
            false,
        )),
        rows[1],
    );
}

fn render_overlay(frame: &mut Frame, area: Rect, app: &App, overlay: &Overlay, theme: Theme) {
    let (width, height, title, lines) = match overlay {
        Overlay::Welcome => (
            72,
            18,
            " BIENVENIDO, APRENDIZ ",
            vec![
                Line::from(Span::styled("VIMURAI NO ES UNA CHULETA.", theme.title())),
                Line::from(Span::styled(
                    "Es un dojo para convertir intención en memoria muscular.",
                    theme.focused(),
                )),
                Line::from(""),
                Line::from("Cada reto te da una meta real sobre código. Tú decides la ruta."),
                Line::from("El mini-Vim observa tu ruta y las teclas objetivo, no la velocidad."),
                Line::from(""),
                Line::from(vec![
                    Span::styled(" NORMAL ", theme.selected()),
                    Span::raw(" navega   "),
                    Span::styled(
                        " INSERT ",
                        Style::default().fg(theme.on_accent).bg(theme.cyan),
                    ),
                    Span::raw(" escribe   "),
                    Span::styled(
                        " VISUAL ",
                        Style::default().fg(theme.on_accent).bg(theme.violet),
                    ),
                    Span::raw(" selecciona"),
                ]),
                Line::from(""),
                Line::from(Span::styled(
                    "Kage reacciona a tu técnica sin robar teclas de Vim.",
                    theme.dim(),
                )),
                Line::from(""),
                Line::from(vec![
                    Span::styled(" g ", theme.selected()),
                    Span::raw(" Academia    "),
                    Span::styled(" Enter / Espacio ", theme.focused()),
                    Span::raw(" entrar"),
                ]),
            ],
        ),
        Overlay::QuitConfirm => (
            54,
            8,
            " CERRAR CONEXIÓN ",
            vec![
                Line::from(Span::styled("¿Salir de Vimurai?", theme.title())),
                Line::from("Tu progreso confirmado ya está guardado localmente."),
                Line::from(""),
                Line::from(vec![
                    Span::styled(" Enter / y ", theme.selected()),
                    Span::raw(" salir    "),
                    Span::styled(" n / Esc ", theme.focused()),
                    Span::raw(" cancelar"),
                ]),
            ],
        ),
        Overlay::ResetProgressConfirm => (
            66,
            10,
            " ZONA DE PELIGRO ",
            vec![
                Line::from(Span::styled("REINICIO IRREVERSIBLE", theme.error())),
                Line::from("Se eliminarán XP, cinturones, racha, actividad y revisiones."),
                Line::from("La configuración se conservará."),
                Line::from(""),
                Line::from(vec![
                    Span::styled(
                        " y ",
                        Style::default()
                            .fg(theme.on_accent)
                            .bg(theme.danger)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" confirmar    "),
                    Span::styled(" n / Esc ", theme.focused()),
                    Span::raw(" cancelar"),
                ]),
            ],
        ),
        Overlay::Completion(card) => (
            72,
            18,
            " MISIÓN COMPLETADA ",
            completion_lines(app, card, theme),
        ),
        Overlay::SessionSummary(summary) => {
            (66, 15, " INFORME DE SESIÓN ", summary_lines(summary, theme))
        }
    };
    let modal = centered(
        area,
        width.min(area.width.saturating_sub(2).max(1)),
        height.min(area.height.saturating_sub(2).max(1)),
    );
    frame.render_widget(Clear, modal);
    frame.render_widget(
        Paragraph::new(lines)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true })
            .style(Style::default().fg(theme.text))
            .block(
                panel(title, theme, true)
                    .border_type(BorderType::Double)
                    .style(Style::default().fg(theme.text)),
            ),
        modal,
    );
}

fn completion_lines(app: &App, card: &CompletionCard, theme: Theme) -> Vec<Line<'static>> {
    let exercise = app.campaign.get(card.exercise_index);
    let title = exercise.map_or("Ejercicio", |exercise| exercise.title);
    let efficiency = exercise.map_or(0, |exercise| {
        if card.actions == 0 {
            0
        } else {
            (u32::from(exercise.optimal_actions).saturating_mul(100) / card.actions).min(100)
        }
    });
    let mut lines = vec![
        Line::from(Span::styled("◆ ACCESS GRANTED ◆", theme.title())),
        Line::from(Span::styled(title, theme.focused())),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                format!(" {} ", stars_text(card.reward.stars, app.force_ascii)),
                Style::default()
                    .fg(theme.amber)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(format!("  +{} XP", card.reward.xp), theme.title()),
        ]),
        Line::from(format!(
            "{} acciones · {} teclas · {} errores · {} pistas · {} · eficiencia {}%",
            card.actions,
            card.keystrokes,
            card.mistakes,
            card.hints,
            format_duration(card.elapsed),
            efficiency
        )),
    ];
    if card.reward.first_completion {
        lines.push(Line::from(Span::styled(
            "BONUS // primera resolución",
            Style::default().fg(theme.amber),
        )));
    }
    if !card.missing_skills.is_empty() {
        lines.push(Line::from(Span::styled(
            format!("Ruta alternativa: {}", card.missing_skills.join(", ")),
            theme.dim(),
        )));
    }
    if !card.violations.is_empty() {
        lines.push(Line::from(Span::styled(
            format!("A mejorar: evita {}", card.violations.join(", ")),
            theme.error(),
        )));
    }
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled(" Enter / n ", theme.selected()),
        Span::raw(" continuar    "),
        Span::styled(" r ", theme.focused()),
        Span::raw(" repetir sin puntuar    "),
        Span::styled(" Esc ", theme.focused()),
        Span::raw(" salir"),
    ]));
    lines
}

fn summary_lines(summary: &SessionSummary, theme: Theme) -> Vec<Line<'static>> {
    let percent = if summary.attempted == 0 {
        0
    } else {
        summary.completed.saturating_mul(100) / summary.attempted
    };
    vec![
        Line::from(Span::styled("DAILY DRILL // SESIÓN CERRADA", theme.title())),
        Line::from(""),
        Line::from(vec![
            Span::styled(format!(" {} ", summary.completed), theme.selected()),
            Span::raw(format!(
                " de {} completados · {percent}%",
                summary.attempted
            )),
        ]),
        Line::from(vec![
            Span::styled(
                format!(" +{} XP ", summary.xp),
                Style::default().fg(theme.on_accent).bg(theme.amber),
            ),
            Span::raw(format!(
                " {} acciones · {}",
                summary.actions,
                format_duration(summary.elapsed)
            )),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "La repetición espaciada programó tu próximo encuentro.",
            theme.dim(),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled(" Enter ", theme.selected()),
            Span::raw(" volver    "),
            Span::styled(" d ", theme.focused()),
            Span::raw(" otro drill"),
        ]),
    ]
}

fn render_toast(frame: &mut Frame, area: Rect, toast: &Toast, theme: Theme) {
    let width = area.width.saturating_sub(2).clamp(1, 68);
    let height = 3.min(area.height.max(1));
    let rect = Rect::new(
        area.x.saturating_add(area.width.saturating_sub(width + 1)),
        area.y
            .saturating_add(1)
            .min(area.bottom().saturating_sub(height)),
        width,
        height,
    );
    let (label, color) = match toast.kind {
        ToastKind::Info => ("INFO", theme.cyan),
        ToastKind::Success => ("OK", theme.primary),
        ToastKind::Warning => ("WARN", theme.amber),
        ToastKind::Error => ("ERROR", theme.danger),
    };
    frame.render_widget(Clear, rect);
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(
                format!(" {label} "),
                Style::default()
                    .fg(theme.on_accent)
                    .bg(color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(format!(" {}", toast.text), Style::default().fg(theme.text)),
        ]))
        .style(Style::default().fg(theme.text))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(color))
                .style(Style::default().fg(theme.text)),
        ),
        rect,
    );
}

fn panel(title: &str, theme: Theme, focused: bool) -> Block<'static> {
    Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(if focused { theme.primary } else { theme.border }))
        .title(Span::styled(
            title.to_owned(),
            if focused { theme.title() } else { theme.dim() },
        ))
        .style(Style::default().fg(theme.text))
}

fn centered(area: Rect, width: u16, height: u16) -> Rect {
    let width = width.min(area.width);
    let height = height.min(area.height);
    Rect::new(
        area.x.saturating_add(area.width.saturating_sub(width) / 2),
        area.y
            .saturating_add(area.height.saturating_sub(height) / 2),
        width,
        height,
    )
}

fn route_name(route: Route) -> &'static str {
    match route {
        Route::Boot => "BOOT",
        Route::Home => "ROOT",
        Route::Academy => "ACADEMIA",
        Route::Practice => "PRACTICE",
        Route::Progress => "PROGRESO",
        Route::Settings => "AJUSTES",
        Route::Help => "REFERENCIA",
    }
}

fn theme_source_label(source: ThemeSource) -> &'static str {
    match source {
        ThemeSource::Osc11 => "OSC11",
        ThemeSource::ColorFgBg => "COLORFGBG",
        ThemeSource::Environment(_) => "OVERRIDE",
        ThemeSource::Default => "AUTO",
    }
}

fn route_keys(route: Route, kind: Option<PracticeKind>) -> &'static str {
    match route {
        Route::Boot => "cualquier tecla: omitir",
        Route::Home => "j/k mover · Enter abrir · ? ayuda · q salir",
        Route::Academy => "h/l panel · j/k mover · Enter abrir · Esc volver",
        Route::Practice if kind == Some(PracticeKind::Free) => {
            "Vim · F1 ayuda · F2 salir · F5 reset · F6 snippet · ^Q cerrar"
        }
        Route::Practice => "Vim activo · F1 pista · F2 salir · F5 reiniciar · Ctrl-Q cerrar",
        Route::Progress => "r actualizar · Esc volver · F1 ayuda",
        Route::Settings => "j/k mover · h/l cambiar · Enter activar · Esc volver",
        Route::Help => "/ buscar · j/k scroll · g inicio · Esc volver",
    }
}

fn route_status(app: &App) -> &str {
    match app.route {
        Route::Home => coach_line(app.mascot_state),
        Route::Academy => "Cada cinturón desbloquea una gramática más poderosa.",
        Route::Practice => app
            .practice
            .as_ref()
            .map_or("Buffer desconectado", |session| session.status.as_str()),
        Route::Progress => "Precisión primero; velocidad después.",
        Route::Settings => "Los cambios se guardan automáticamente.",
        Route::Help => "Busca por tecla, nombre o intención.",
        Route::Boot => "Inicializando…",
    }
}

fn mode_name(mode: Mode) -> &'static str {
    match mode {
        Mode::Normal => "NORMAL",
        Mode::Insert => "INSERT",
        Mode::VisualChar => "VISUAL",
        Mode::VisualLine => "V-LINE",
        Mode::Command => "COMMAND",
        Mode::Search => "SEARCH",
    }
}

fn mode_badge(theme: Theme, mode: Mode) -> Style {
    let color = match mode {
        Mode::Normal => theme.primary,
        Mode::Insert => theme.cyan,
        Mode::VisualChar | Mode::VisualLine => theme.violet,
        Mode::Command | Mode::Search => theme.amber,
    };
    Style::default()
        .fg(theme.on_accent)
        .bg(color)
        .add_modifier(Modifier::BOLD)
}

fn goal_summary(exercise: &Exercise) -> String {
    let mode = mode_name(exercise.goal.expected_mode());
    match (
        exercise.goal.expected_position(),
        exercise.goal.expected_lines(),
    ) {
        (Some(position), Some(_)) => format!(
            "buffer + cursor {}:{} + {mode}",
            position.row.saturating_add(1),
            position.col.saturating_add(1)
        ),
        (Some(position), None) => format!(
            "cursor {}:{} + {mode}",
            position.row.saturating_add(1),
            position.col.saturating_add(1)
        ),
        (None, Some(_)) => format!("transformar buffer + {mode}"),
        (None, None) => mode.to_owned(),
    }
}

fn belt_color(app: &App, theme: Theme, belt: Belt) -> Color {
    if app.no_color || app.settings.high_contrast {
        theme.primary
    } else {
        match belt {
            Belt::Survivor => theme.text,
            Belt::Sniper => theme.amber,
            Belt::Refactorer => theme.orange,
            Belt::Surgeon => theme.danger,
            Belt::Architect => theme.blue,
            Belt::Wizard => theme.violet,
        }
    }
}

fn stars_text(stars: u8, ascii: bool) -> String {
    let stars = stars.min(3);
    if ascii {
        format!(
            "[{}{}]",
            "*".repeat(usize::from(stars)),
            ".".repeat(usize::from(3 - stars))
        )
    } else {
        format!(
            "{}{}",
            "★".repeat(usize::from(stars)),
            "☆".repeat(usize::from(3 - stars))
        )
    }
}

fn mini_bar(done: usize, total: usize, width: usize, ascii: bool) -> String {
    let filled = if total == 0 {
        0
    } else {
        done.min(total).saturating_mul(width) / total
    };
    let (on, off) = if ascii { ("#", ".") } else { ("━", "─") };
    format!(
        "{}{}",
        on.repeat(filled),
        off.repeat(width.saturating_sub(filled))
    )
}

fn trailing_activity(app: &App) -> Vec<&ActivityDay> {
    app.activity
        .iter()
        .skip(app.activity.len().saturating_sub(35))
        .collect()
}

fn activity_compact_line(app: &App, theme: Theme) -> Line<'static> {
    let mut spans = vec![Span::styled("35d ", theme.dim())];
    for day in trailing_activity(app) {
        spans.push(Span::styled(activity_char(day), activity_style(day, theme)));
    }
    Line::from(spans)
}

fn activity_char(day: &ActivityDay) -> &'static str {
    match activity_level(day) {
        0 => ".",
        1 => "o",
        2 => "O",
        _ => "#",
    }
}

fn activity_style(day: &ActivityDay, theme: Theme) -> Style {
    match activity_level(day) {
        0 => Style::default().fg(theme.border),
        1 => Style::default().fg(theme.muted),
        2 => Style::default().fg(theme.cyan),
        _ => Style::default()
            .fg(theme.primary)
            .add_modifier(Modifier::BOLD),
    }
}

fn activity_level(day: &ActivityDay) -> u8 {
    match day.actions.saturating_add(day.sessions.saturating_mul(6)) {
        0 => 0,
        1..=12 => 1,
        13..=40 => 2,
        _ => 3,
    }
}

fn format_duration(duration: Duration) -> String {
    let seconds = duration.as_secs();
    format!("{:02}:{:02}", seconds / 60, seconds % 60)
}

fn digits(mut number: usize) -> usize {
    let mut digits = 1;
    while number >= 10 {
        number /= 10;
        digits += 1;
    }
    digits
}

fn scroll_start(selected: usize, total: usize, visible: usize) -> usize {
    if total <= visible {
        0
    } else {
        selected
            .saturating_sub(visible / 2)
            .min(total.saturating_sub(visible))
    }
}

fn clip(value: &str, width: usize) -> String {
    if width == 0 {
        return String::new();
    }
    if value.chars().count() <= width {
        return value.to_owned();
    }
    if width == 1 {
        return "…".to_owned();
    }
    let mut result = value
        .chars()
        .take(width.saturating_sub(1))
        .collect::<String>();
    result.push('…');
    result
}

fn pad(value: &str, width: usize) -> String {
    let value = clip(value, width);
    let spaces = width.saturating_sub(value.chars().count());
    format!("{value}{}", " ".repeat(spaces))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use ratatui::{Terminal, backend::TestBackend, buffer::Buffer};

    use crate::{app::AppOptions, progress::Reward, terminal_appearance::TerminalTheme};

    fn app() -> App {
        let mut app = App::in_memory(AppOptions {
            skip_boot: true,
            no_animation: true,
            ..AppOptions::default()
        })
        .expect("in-memory app");
        app.overlay = None;
        app
    }

    fn draw(app: &App, width: u16, height: u16) -> String {
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).expect("terminal");
        terminal.draw(|frame| render(frame, app)).expect("render");
        let mut screen = String::new();
        for row in terminal
            .backend()
            .buffer()
            .content()
            .chunks(usize::from(width))
        {
            for cell in row {
                screen.push_str(cell.symbol());
            }
            screen.push('\n');
        }
        screen
    }

    fn draw_buffer(app: &App, width: u16, height: u16) -> Buffer {
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).expect("terminal");
        terminal.draw(|frame| render(frame, app)).expect("render");
        terminal.backend().buffer().clone()
    }

    #[test]
    fn every_route_renders_at_all_breakpoints() {
        let mut app = app();
        app.handle_key(KeyEvent::new(KeyCode::Char('f'), KeyModifiers::NONE));
        assert_eq!(app.route, Route::Practice);
        for (width, height) in [
            (50, 12),
            (59, 17),
            (60, 18),
            (80, 24),
            (100, 28),
            (100, 30),
            (126, 35),
            (140, 40),
        ] {
            for route in [
                Route::Boot,
                Route::Home,
                Route::Academy,
                Route::Practice,
                Route::Progress,
                Route::Settings,
                Route::Help,
            ] {
                app.route = route;
                let _ = draw(&app, width, height);
            }
        }
    }

    #[test]
    fn every_overlay_renders_tiny_and_wide() {
        let mut app = app();
        let overlays = vec![
            Overlay::Welcome,
            Overlay::QuitConfirm,
            Overlay::ResetProgressConfirm,
            Overlay::Completion(CompletionCard {
                exercise_index: 0,
                reward: Reward {
                    xp: 70,
                    stars: 3,
                    quality: 5,
                    first_completion: true,
                },
                actions: 4,
                keystrokes: 12,
                mistakes: 0,
                hints: 0,
                elapsed: Duration::from_secs(19),
                missing_skills: Vec::new(),
                violations: Vec::new(),
            }),
            Overlay::SessionSummary(SessionSummary {
                attempted: 5,
                completed: 4,
                xp: 155,
                actions: 23,
                elapsed: Duration::from_secs(211),
            }),
        ];
        for overlay in overlays {
            app.overlay = Some(overlay);
            for (width, height) in [
                (50, 12),
                (59, 17),
                (60, 18),
                (80, 24),
                (100, 28),
                (126, 35),
                (140, 40),
            ] {
                let _ = draw(&app, width, height);
            }
        }
    }

    #[test]
    fn responsive_screens_keep_their_semantic_landmarks() {
        let mut app = app();
        let tiny = draw(&app, 59, 17);
        if std::env::var_os("VIMURAI_UI_DUMP").is_some() {
            eprintln!("--- 59x17 HOME ---\n{tiny}");
        }
        assert!(tiny.contains("TERMINAL COMPACTA"));
        assert!(tiny.contains("DAILY DRILL"));

        let compact_home = draw(&app, 60, 18);
        assert!(compact_home.contains("NODOS DE ACCESO"));
        assert!(compact_home.contains("KAGE // ONLINE"));

        app.route = Route::Academy;
        let academy = draw(&app, 60, 18);
        if std::env::var_os("VIMURAI_UI_DUMP").is_some() {
            eprintln!("--- 60x18 ACADEMY ---\n{academy}");
        }
        assert!(academy.contains("CINTURONES"));
        assert!(academy.contains("EJERCICIOS"));
        assert!(academy.contains("DOSSIER"));

        app.route = Route::Home;
        app.handle_key(KeyEvent::new(KeyCode::Char('f'), KeyModifiers::NONE));
        let practice = draw(&app, 80, 24);
        if std::env::var_os("VIMURAI_UI_DUMP").is_some() {
            eprintln!("--- 80x24 PRACTICE ---\n{practice}");
        }
        assert!(practice.contains("SANDBOX // NO SCORE"));
        assert!(practice.contains("BUFFER // NORMAL"));
        assert!(practice.contains("SIGNAL / KEYCAST"));
        assert!(practice.contains("keys: esperando señal"));

        app.handle_key(KeyEvent::new(KeyCode::Char('v'), KeyModifiers::NONE));
        let visual = draw(&app, 80, 24);
        assert!(visual.contains("BUFFER // VISUAL"));
        app.handle_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
        app.handle_key(KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE));
        let pending = draw(&app, 80, 24);
        assert!(pending.contains("d_"));

        app.route = Route::Progress;
        let progress = draw(&app, 100, 28);
        if std::env::var_os("VIMURAI_UI_DUMP").is_some() {
            eprintln!("--- 100x28 PROGRESS ---\n{progress}");
        }
        assert!(progress.contains("ACTIVIDAD // 35 DÍAS"));
        assert!(progress.contains("LOGROS"));

        app.route = Route::Help;
        let help = draw(&app, 126, 35);
        if std::env::var_os("VIMURAI_UI_DUMP").is_some() {
            eprintln!("--- 126x35 HELP ---\n{help}");
        }
        assert!(help.contains("ÍNDICE DE MOTIONS"));
        assert!(help.contains("REFERENCIA"));

        app.route = Route::Settings;
        app.settings_index = 0;
        app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        let toast = draw(&app, 100, 28);
        assert!(toast.contains("Ajuste guardado"));
    }

    #[test]
    fn no_color_uses_the_ascii_mascot() {
        let mut app = app();
        app.no_color = true;
        let screen = draw(&app, 126, 35);
        assert!(screen.contains("( o.o )"));
    }

    #[test]
    fn both_themes_keep_surfaces_transparent_and_only_color_semantic_cells() {
        let mut app = app();
        app.no_color = false;
        for variant in [TerminalTheme::Dark, TerminalTheme::Light] {
            app.terminal_theme = variant;
            let theme = Theme::gruvbox(variant, false, false);
            let buffer = draw_buffer(&app, 100, 28);
            let allowed_backgrounds = [
                Color::Reset,
                theme.primary,
                theme.cyan,
                theme.amber,
                theme.danger,
                theme.violet,
            ];
            assert!(
                buffer
                    .content()
                    .iter()
                    .all(|cell| { allowed_backgrounds.contains(&cell.bg) })
            );
            assert!(buffer.content().iter().any(|cell| cell.fg == theme.primary));
            assert!(buffer.content().iter().any(|cell| cell.bg == theme.primary));
        }

        app.no_color = true;
        let monochrome = draw_buffer(&app, 100, 28);
        assert!(
            monochrome
                .content()
                .iter()
                .all(|cell| cell.bg == Color::Reset)
        );
    }

    #[test]
    fn full_cat_appears_only_at_the_large_breakpoint() {
        let mut app = app();
        app.no_color = false;
        app.force_ascii = false;
        let compact = draw(&app, 100, 28);
        assert!(!compact.contains("zZz..."));
        let full = draw(&app, 126, 35);
        assert!(full.contains("zZz..."));
    }

    #[test]
    fn helpers_saturate() {
        assert_eq!(clip("abc", 0), "");
        assert_eq!(clip("abc", 1), "…");
        assert_eq!(scroll_start(usize::MAX, 0, 0), 0);
        assert_eq!(mini_bar(99, 0, 7, true), ".......");
    }
}
