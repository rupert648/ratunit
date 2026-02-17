use crate::app::{App, View};
use junit_parser::TestStatus;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};
use ratatui::Frame;

pub fn render(frame: &mut Frame, app: &App) {
    let [main_area, status_area] =
        Layout::vertical([Constraint::Fill(1), Constraint::Length(2)]).areas(frame.area());

    if app.multi_file {
        let [sidebar_area, content_area] =
            Layout::horizontal([Constraint::Percentage(25), Constraint::Percentage(75)])
                .areas(main_area);
        render_file_sidebar(frame, sidebar_area, app);
        render_content(frame, content_area, app);
    } else {
        render_content(frame, main_area, app);
    }

    render_status_bar(frame, status_area, app);
}

fn render_file_sidebar(frame: &mut Frame, area: Rect, app: &App) {
    let items: Vec<ListItem> = app
        .files
        .iter()
        .enumerate()
        .map(|(i, f)| {
            let passed = f.data.total_passed();
            let failed = f.data.total_failures();
            let total = f.data.total_tests();

            let short_name = f
                .filename
                .strip_prefix("wdio-")
                .unwrap_or(&f.filename)
                .strip_suffix("--report.xml")
                .unwrap_or(&f.filename);

            let style = if failed > 0 {
                Style::default().fg(Color::Red)
            } else {
                Style::default().fg(Color::Green)
            };

            let label = format!("{} ({}/{})", short_name, passed, total);
            let item = ListItem::new(label).style(style);

            if i == app.selected_file {
                item.style(style.add_modifier(Modifier::BOLD))
            } else {
                item
            }
        })
        .collect();

    let block = Block::default()
        .title(" Files ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let mut state = ListState::default().with_selected(Some(app.selected_file));
    let list = List::new(items)
        .block(block)
        .highlight_style(Style::default().bg(Color::DarkGray).bold())
        .highlight_symbol("> ");

    frame.render_stateful_widget(list, area, &mut state);
}

fn render_content(frame: &mut Frame, area: Rect, app: &App) {
    match app.view {
        View::SuiteList => render_suite_list(frame, area, app),
        View::TestList => render_test_list(frame, area, app),
        View::TestDetail => render_test_detail(frame, area, app),
    }
}

fn render_suite_list(frame: &mut Frame, area: Rect, app: &App) {
    let file = app.current_file();
    let items: Vec<ListItem> = file
        .data
        .suites
        .iter()
        .map(|suite| {
            let passed = suite
                .tests
                .saturating_sub(suite.failures + suite.errors + suite.skipped.unwrap_or(0));
            let time_str = suite.time.map(|t| format!("{:.1}s", t)).unwrap_or_default();

            let status_color = if suite.failures > 0 || suite.errors > 0 {
                Color::Red
            } else if suite.skipped.unwrap_or(0) > 0 && suite.tests == suite.skipped.unwrap_or(0) {
                Color::Yellow
            } else {
                Color::Green
            };

            let line = Line::from(vec![
                Span::styled(
                    format!("{:<50} ", truncate_str(&suite.name, 50)),
                    Style::default().fg(status_color),
                ),
                Span::styled(
                    format!("{:>3} tests ", suite.tests),
                    Style::default().fg(Color::White),
                ),
                Span::styled(
                    format!("{:>3} pass ", passed),
                    Style::default().fg(Color::Green),
                ),
                Span::styled(
                    format!("{:>3} fail ", suite.failures),
                    if suite.failures > 0 {
                        Style::default().fg(Color::Red)
                    } else {
                        Style::default().fg(Color::DarkGray)
                    },
                ),
                Span::styled(
                    format!("{:>3} skip ", suite.skipped.unwrap_or(0)),
                    if suite.skipped.unwrap_or(0) > 0 {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default().fg(Color::DarkGray)
                    },
                ),
                Span::styled(
                    format!("{:>8}", time_str),
                    Style::default().fg(Color::DarkGray),
                ),
            ]);

            ListItem::new(line)
        })
        .collect();

    let title = format!(" Test Suites — {} ", file.filename);
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let mut state = ListState::default().with_selected(Some(app.selected_suite));
    let list = List::new(items)
        .block(block)
        .highlight_style(Style::default().bg(Color::DarkGray).bold())
        .highlight_symbol("> ");

    frame.render_stateful_widget(list, area, &mut state);
}

fn render_test_list(frame: &mut Frame, area: Rect, app: &App) {
    let file = app.current_file();
    let suite = &file.data.suites[app.selected_suite];

    let items: Vec<ListItem> = suite
        .test_cases
        .iter()
        .map(|tc| {
            let (badge, badge_color) = match tc.status() {
                TestStatus::Passed => ("PASS", Color::Green),
                TestStatus::Failed => ("FAIL", Color::Red),
                TestStatus::Skipped => ("SKIP", Color::Yellow),
                TestStatus::Errored => ("ERR ", Color::Magenta),
            };

            let time_str = tc.time.map(|t| format!("{:.2}s", t)).unwrap_or_default();

            let line = Line::from(vec![
                Span::styled(
                    format!(" [{}] ", badge),
                    Style::default().fg(badge_color).bold(),
                ),
                Span::styled(
                    format!("{:<70} ", truncate_str(&tc.name, 70)),
                    Style::default().fg(Color::White),
                ),
                Span::styled(
                    format!("{:>8}", time_str),
                    Style::default().fg(Color::DarkGray),
                ),
            ]);

            ListItem::new(line)
        })
        .collect();

    let title = format!(" Tests — {} ", truncate_str(&suite.name, 60));
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let mut state = ListState::default().with_selected(Some(app.selected_test));
    let list = List::new(items)
        .block(block)
        .highlight_style(Style::default().bg(Color::DarkGray).bold())
        .highlight_symbol("> ");

    frame.render_stateful_widget(list, area, &mut state);
}

fn render_test_detail(frame: &mut Frame, area: Rect, app: &App) {
    let file = app.current_file();
    let suite = &file.data.suites[app.selected_suite];
    let tc = &suite.test_cases[app.selected_test];

    let (status_text, status_color) = match tc.status() {
        TestStatus::Passed => ("PASSED", Color::Green),
        TestStatus::Failed => ("FAILED", Color::Red),
        TestStatus::Skipped => ("SKIPPED", Color::Yellow),
        TestStatus::Errored => ("ERROR", Color::Magenta),
    };

    let mut lines: Vec<Line> = Vec::new();

    lines.push(Line::from(vec![
        Span::styled("  Name: ", Style::default().bold().fg(Color::Cyan)),
        Span::raw(&tc.name),
    ]));

    if let Some(ref classname) = tc.classname {
        lines.push(Line::from(vec![
            Span::styled(" Class: ", Style::default().bold().fg(Color::Cyan)),
            Span::raw(classname),
        ]));
    }

    if let Some(ref file_path) = tc.file {
        lines.push(Line::from(vec![
            Span::styled("  File: ", Style::default().bold().fg(Color::Cyan)),
            Span::raw(file_path),
        ]));
    }

    lines.push(Line::from(vec![
        Span::styled("  Time: ", Style::default().bold().fg(Color::Cyan)),
        Span::raw(tc.time.map(|t| format!("{:.3}s", t)).unwrap_or_default()),
    ]));

    lines.push(Line::from(vec![
        Span::styled("Status: ", Style::default().bold().fg(Color::Cyan)),
        Span::styled(status_text, Style::default().fg(status_color).bold()),
    ]));

    lines.push(Line::raw(""));

    if let Some(ref failure) = tc.failure {
        lines.push(Line::styled(
            "── Failure ──────────────────────────────────────────",
            Style::default().fg(Color::Red).bold(),
        ));
        if let Some(ref msg) = failure.message {
            for l in msg.lines() {
                lines.push(Line::styled(l.to_string(), Style::default().fg(Color::Red)));
            }
        }
        if let Some(ref body) = failure.body {
            lines.push(Line::raw(""));
            for l in body.lines() {
                lines.push(Line::raw(format!("  {}", l)));
            }
        }
        lines.push(Line::raw(""));
    }

    if let Some(ref error) = tc.error {
        lines.push(Line::styled(
            "── Error ────────────────────────────────────────────",
            Style::default().fg(Color::Magenta).bold(),
        ));
        if let Some(ref msg) = error.message {
            for l in msg.lines() {
                lines.push(Line::styled(
                    l.to_string(),
                    Style::default().fg(Color::Magenta),
                ));
            }
        }
        if let Some(ref body) = error.body {
            lines.push(Line::raw(""));
            for l in body.lines() {
                lines.push(Line::raw(format!("  {}", l)));
            }
        }
        lines.push(Line::raw(""));
    }

    if let Some(ref stdout) = tc.system_out {
        let trimmed = stdout.trim();
        if !trimmed.is_empty() {
            lines.push(Line::styled(
                "── System Out ───────────────────────────────────────",
                Style::default().fg(Color::Blue).bold(),
            ));
            for l in trimmed.lines() {
                lines.push(Line::raw(format!("  {}", l)));
            }
            lines.push(Line::raw(""));
        }
    }

    if let Some(ref stderr) = tc.system_err {
        let trimmed = stderr.trim();
        if !trimmed.is_empty() {
            lines.push(Line::styled(
                "── System Err ───────────────────────────────────────",
                Style::default().fg(Color::Yellow).bold(),
            ));
            for l in trimmed.lines() {
                lines.push(Line::styled(
                    format!("  {}", l),
                    Style::default().fg(Color::Yellow),
                ));
            }
            lines.push(Line::raw(""));
        }
    }

    let title = format!(" Detail — {} ", truncate_str(&tc.name, 50));
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false })
        .scroll((app.scroll_offset, 0));

    frame.render_widget(paragraph, area);
}

fn render_status_bar(frame: &mut Frame, area: Rect, app: &App) {
    let [stats_area, keys_area] =
        Layout::vertical([Constraint::Length(1), Constraint::Length(1)]).areas(area);

    let stats_line = Line::from(vec![
        Span::styled(" Total: ", Style::default().bold()),
        Span::styled(
            format!("{} ", app.aggregate_tests()),
            Style::default().fg(Color::White).bold(),
        ),
        Span::raw("│ "),
        Span::styled("Passed: ", Style::default().fg(Color::Green)),
        Span::styled(
            format!("{} ", app.aggregate_passed()),
            Style::default().fg(Color::Green).bold(),
        ),
        Span::raw("│ "),
        Span::styled("Failed: ", Style::default().fg(Color::Red)),
        Span::styled(
            format!("{} ", app.aggregate_failures()),
            Style::default().fg(Color::Red).bold(),
        ),
        Span::raw("│ "),
        Span::styled("Errors: ", Style::default().fg(Color::Magenta)),
        Span::styled(
            format!("{} ", app.aggregate_errors()),
            Style::default().fg(Color::Magenta).bold(),
        ),
        Span::raw("│ "),
        Span::styled("Skipped: ", Style::default().fg(Color::Yellow)),
        Span::styled(
            format!("{}", app.aggregate_skipped()),
            Style::default().fg(Color::Yellow).bold(),
        ),
    ]);

    let keys_line = match app.view {
        View::SuiteList => Line::from(vec![
            Span::styled(" j/k", Style::default().bold().fg(Color::Cyan)),
            Span::raw(" navigate  "),
            Span::styled("Enter", Style::default().bold().fg(Color::Cyan)),
            Span::raw(" open  "),
            if app.multi_file {
                Span::styled("Tab", Style::default().bold().fg(Color::Cyan))
            } else {
                Span::raw("")
            },
            if app.multi_file {
                Span::raw(" switch file  ")
            } else {
                Span::raw("")
            },
            Span::styled("q", Style::default().bold().fg(Color::Cyan)),
            Span::raw(" quit"),
        ]),
        View::TestList => Line::from(vec![
            Span::styled(" j/k", Style::default().bold().fg(Color::Cyan)),
            Span::raw(" navigate  "),
            Span::styled("Enter", Style::default().bold().fg(Color::Cyan)),
            Span::raw(" detail  "),
            Span::styled("Esc", Style::default().bold().fg(Color::Cyan)),
            Span::raw(" back  "),
            Span::styled("q", Style::default().bold().fg(Color::Cyan)),
            Span::raw(" quit"),
        ]),
        View::TestDetail => Line::from(vec![
            Span::styled(" j/k", Style::default().bold().fg(Color::Cyan)),
            Span::raw(" scroll  "),
            Span::styled("Esc", Style::default().bold().fg(Color::Cyan)),
            Span::raw(" back  "),
            Span::styled("q", Style::default().bold().fg(Color::Cyan)),
            Span::raw(" quit"),
        ]),
    };

    let stats_widget =
        Paragraph::new(stats_line).style(Style::default().bg(Color::DarkGray).fg(Color::White));
    let keys_widget = Paragraph::new(keys_line).style(Style::default().fg(Color::DarkGray));

    frame.render_widget(stats_widget, stats_area);
    frame.render_widget(keys_widget, keys_area);
}

fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}
