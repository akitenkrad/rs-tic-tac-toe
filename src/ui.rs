//! ターミナル画面の描画．

use ratatui::{
    Frame,
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Clear, Paragraph},
};

use crate::app::{App, Hint, Screen};
use crate::game::{Evaluation, Mark, Outcome, Side, cell_name, solve};
use crate::i18n::{Lang, Msg};

type Rgb = (u8, u8, u8);

const BG: Rgb = (16, 16, 26);
const CELL_BG: Rgb = (35, 35, 57);
const CELL_EDGE: Rgb = (65, 65, 92);
const DIM: Rgb = (110, 110, 140);
const TEXT: Rgb = (222, 222, 236);
const ACCENT: Rgb = (120, 200, 255);
const X_COLOR: Rgb = (110, 220, 160);
const O_COLOR: Rgb = (245, 176, 65);
const DANGER: Rgb = (255, 88, 110);
const PURPLE: Rgb = (165, 150, 255);

pub const MIN_WIDTH: u16 = 80;
pub const MIN_HEIGHT: u16 = 24;

fn c(rgb: Rgb) -> Color {
    Color::Rgb(rgb.0, rgb.1, rgb.2)
}

fn fg(rgb: Rgb) -> Style {
    Style::new().fg(c(rgb))
}

fn shade(rgb: Rgb, delta: i16) -> Rgb {
    let shift = |value: u8| (i16::from(value) + delta).clamp(0, 255) as u8;
    (shift(rgb.0), shift(rgb.1), shift(rgb.2))
}

fn mark_color(mark: Mark) -> Rgb {
    match mark {
        Mark::X => X_COLOR,
        Mark::O => O_COLOR,
    }
}

fn centered(area: Rect, width: u16, height: u16) -> Rect {
    let width = width.min(area.width);
    let height = height.min(area.height);
    Rect::new(
        area.x + area.width.saturating_sub(width) / 2,
        area.y + area.height.saturating_sub(height) / 2,
        width,
        height,
    )
}

fn panel(title: &str) -> Block<'_> {
    Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(fg(DIM))
        .title(Span::styled(title, fg(TEXT).add_modifier(Modifier::BOLD)))
}

fn key(label: &str) -> Span<'static> {
    Span::styled(format!(" {label} "), Style::new().fg(c(BG)).bg(c(ACCENT)).add_modifier(Modifier::BOLD))
}

pub fn draw(frame: &mut Frame, app: &App) {
    let area = frame.area();
    frame.render_widget(Block::new().style(Style::new().bg(c(BG)).fg(c(TEXT))), area);
    if area.width < MIN_WIDTH || area.height < MIN_HEIGHT {
        let message = app.lang.resize(MIN_WIDTH, MIN_HEIGHT, area.width, area.height);
        frame.render_widget(Paragraph::new(message).alignment(Alignment::Center), centered(area, area.width, 1));
        return;
    }
    match app.screen {
        Screen::Title => draw_title(frame, app, area),
        Screen::Playing | Screen::GameOver => draw_game(frame, app, area),
    }
    draw_particles(frame.buffer_mut(), app, area);
    if app.screen == Screen::GameOver {
        let [_, middle, _] = Layout::vertical([Constraint::Length(3), Constraint::Fill(1), Constraint::Length(4)]).areas(area);
        let [_, side] = Layout::horizontal([Constraint::Fill(1), Constraint::Length(32)]).areas(middle);
        let [result, _] = Layout::vertical([Constraint::Length(9), Constraint::Fill(1)]).areas(side);
        draw_result(frame, app, result);
    }
}

fn draw_title(frame: &mut Frame, app: &App, area: Rect) {
    let [top, logo, subtitle, menu, footer, bottom] = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(5),
        Constraint::Length(2),
        Constraint::Length(10),
        Constraint::Length(2),
        Constraint::Fill(1),
    ])
    .areas(area);
    let _ = (top, bottom);
    let logo_lines = [
        "",
        "╔╦╗ ╦ ╔═╗   ╔╦╗ ╔═╗ ╔═╗   ╔╦╗ ╔═╗ ╔═╗",
        " ║  ║ ║      ║  ╠═╣ ║      ║  ║ ║ ║╣ ",
        " ╩  ╩ ╚═╝    ╩  ╩ ╩ ╚═╝    ╩  ╚═╝ ╚═╝",
        "",
    ];
    frame.render_widget(
        Paragraph::new(
            logo_lines
                .map(|line| Line::styled(line, fg(ACCENT).add_modifier(Modifier::BOLD)))
                .to_vec(),
        )
        .alignment(Alignment::Center),
        logo,
    );
    frame.render_widget(
        Paragraph::new(vec![
            Line::styled("rs-tic-tac-toe", fg(TEXT).add_modifier(Modifier::BOLD)),
            Line::styled(Msg::TitleSubtitle.text(app.lang), fg(DIM)),
        ])
        .alignment(Alignment::Center),
        subtitle,
    );
    let row = |button: &str, label: &str, value: &str| {
        Line::from(vec![
            Span::raw("  "),
            key(button),
            Span::styled(format!("  {label:<12}  "), fg(DIM)),
            Span::styled(value.to_string(), fg(TEXT).add_modifier(Modifier::BOLD)),
        ])
    };
    let lines = vec![
        row("Enter", Msg::StartGame.text(app.lang), ""),
        row("d", Msg::Difficulty.text(app.lang), app.difficulty.label(app.lang)),
        row("f", Msg::First.text(app.lang), app.first.label(app.lang)),
        row("l", Msg::Language.text(app.lang), app.lang.label()),
        row("q", Msg::Quit.text(app.lang), ""),
    ];
    frame.render_widget(
        Paragraph::new(lines).block(panel(Msg::Settings.text(app.lang)).padding(ratatui::widgets::Padding::vertical(1))),
        centered(menu, 54, 9),
    );
    if (app.frame / 18).is_multiple_of(2) {
        frame.render_widget(
            Paragraph::new(Span::styled(Msg::StartPrompt.text(app.lang), fg(ACCENT).add_modifier(Modifier::BOLD))).alignment(Alignment::Center),
            footer,
        );
    }
}

fn draw_game(frame: &mut Frame, app: &App, area: Rect) {
    let [status, middle, help] = Layout::vertical([Constraint::Length(3), Constraint::Fill(1), Constraint::Length(4)]).areas(area);
    draw_status(frame, app, status);
    let [board, side] = Layout::horizontal([Constraint::Fill(1), Constraint::Length(32)]).areas(middle);
    draw_board(frame, app, board);
    if app.screen == Screen::GameOver {
        let [_, log] = Layout::vertical([Constraint::Length(9), Constraint::Fill(1)]).areas(side);
        draw_move_log(frame, app, log);
    } else {
        draw_side(frame, app, side);
    }
    draw_help(frame, app, help);
}

fn draw_status(frame: &mut Frame, app: &App, area: Rect) {
    let mut spans = vec![Span::styled(
        " TIC-TAC-TOE ",
        Style::new().fg(c(BG)).bg(c(ACCENT)).add_modifier(Modifier::BOLD),
    )];
    if let Some(replay) = app.replay() {
        let state = if replay.position == replay.record.moves.len() {
            Msg::Finished
        } else if replay.paused {
            Msg::Paused
        } else {
            Msg::Playing
        };
        spans.extend([
            Span::styled(format!("  {}", state.text(app.lang)), fg(ACCENT).add_modifier(Modifier::BOLD)),
            Span::styled(
                format!("  {} ", app.lang.replay_progress(replay.position, replay.record.moves.len())),
                fg(TEXT),
            ),
            Span::styled(format!(" {} ", Msg::Speed.text(app.lang)), fg(DIM)),
            Span::styled(format!("{}ms", replay.interval.as_millis()), fg(TEXT)),
        ]);
    } else {
        let mark = app.game.mark_for(app.game.to_move);
        spans.extend([
            Span::styled(
                format!("  {}", app.lang.turn(app.side_name(app.game.to_move), mark_label(mark))),
                fg(mark_color(mark)).add_modifier(Modifier::BOLD),
            ),
            Span::styled(format!("  {} ", Msg::Difficulty.text(app.lang)), fg(DIM)),
            Span::styled(app.difficulty.label(app.lang), fg(TEXT)),
            Span::styled(format!("  {}", app.lang.moves_count(app.game.history.len())), fg(TEXT)),
        ]);
    }
    frame.render_widget(Paragraph::new(Line::from(spans)).block(panel("")), area);
}

fn draw_board(frame: &mut Frame, app: &App, area: Rect) {
    let title = if app.show_evaluation {
        Msg::BoardEvaluation.text(app.lang)
    } else {
        Msg::Board.text(app.lang)
    };
    frame.render_widget(
        Paragraph::new(Span::styled(title, fg(TEXT).add_modifier(Modifier::BOLD))).alignment(Alignment::Center),
        Rect::new(area.x, area.y, area.width, 1),
    );
    let board = centered(Rect::new(area.x, area.y + 1, area.width, area.height.saturating_sub(1)), 35, 15);
    let evaluations = app.show_evaluation.then(|| solve(&app.game));
    let winning = match app.game.outcome() {
        Some(Outcome::Win { line, .. }) => Some(line),
        _ => None,
    };
    for cell in 0..9 {
        let col = cell % 3;
        let row = cell / 3;
        let rect = Rect::new(board.x + col as u16 * 12, board.y + row as u16 * 5, 11, 5);
        draw_cell(frame.buffer_mut(), app, cell, rect, evaluations.as_ref(), winning);
    }
}

fn draw_cell(buffer: &mut Buffer, app: &App, cell: usize, area: Rect, evaluations: Option<&[Option<Evaluation>; 9]>, winning: Option<[usize; 3]>) {
    let selected = app.cursor == cell && (app.human_can_act() || app.cpu.as_ref().is_some_and(|turn| turn.showing));
    let hinted = matches!(&app.hint, Hint::Moves { cells, .. } if cells.contains(&cell));
    let win_flash = winning.is_some_and(|line| line.contains(&cell)) && (app.frame / 8).is_multiple_of(2);
    let edge = if win_flash {
        DANGER
    } else if hinted {
        X_COLOR
    } else if selected {
        ACCENT
    } else {
        CELL_EDGE
    };
    for dy in 0..area.height {
        for dx in 0..area.width {
            let border = dy == 0 || dy + 1 == area.height || dx == 0 || dx + 1 == area.width;
            let color = if border {
                if dx == 0 || dy == 0 { shade(edge, 24) } else { shade(edge, -32) }
            } else {
                CELL_BG
            };
            buffer[(area.x + dx, area.y + dy)].set_char(' ').set_bg(c(color));
        }
    }

    if let Some(mark) = app.game.board[cell] {
        let progress = app
            .placement
            .as_ref()
            .filter(|placement| placement.cell == cell)
            .map_or(1.0, crate::app::Placement::progress);
        draw_mark(buffer, area, mark, progress);
    } else if let Some(evaluation) = evaluations.and_then(|items| items[cell]) {
        let (label, color) = evaluation_label(evaluation, app.lang);
        let x = area.x + area.width.saturating_sub(label.chars().count() as u16) / 2;
        buffer.set_string(x, area.y + 2, label, fg(color).add_modifier(Modifier::BOLD));
    } else {
        let label = format!("{}", cell + 1);
        buffer.set_string(area.x + 5, area.y + 2, label, fg(DIM));
        buffer.set_string(area.x + 1, area.y + 4, cell_name(cell), fg(DIM));
    }
}

fn draw_mark(buffer: &mut Buffer, area: Rect, mark: Mark, progress: f32) {
    let color = mark_color(mark);
    for dy in 0..3u16 {
        if f32::from(dy + 1) / 3.0 > progress {
            continue;
        }
        for dx in 0..9u16 {
            let group = dx / 3;
            let filled = match mark {
                Mark::X => group == dy || group + dy == 2,
                Mark::O => dy == 0 || dy == 2 || group == 0 || group == 2,
            };
            if filled {
                let tint = if dx % 3 == 0 {
                    shade(color, 38)
                } else if dx % 3 == 2 {
                    shade(color, -38)
                } else {
                    color
                };
                buffer[(area.x + 1 + dx, area.y + 1 + dy)].set_char(' ').set_bg(c(tint));
            }
        }
    }
}

fn evaluation_label(evaluation: Evaluation, lang: Lang) -> (String, Rgb) {
    match evaluation {
        Evaluation::Win(turns) => (lang.evaluation(Msg::EvalWin, turns), X_COLOR),
        Evaluation::Draw => (Msg::EvalDraw.text(lang).into(), PURPLE),
        Evaluation::Loss(turns) => (lang.evaluation(Msg::EvalLoss, turns), DANGER),
    }
}

fn draw_side(frame: &mut Frame, app: &App, area: Rect) {
    if app.replay().is_some() {
        draw_move_log(frame, app, area);
        return;
    }

    let [info, stats, log] = Layout::vertical([Constraint::Length(6), Constraint::Length(5), Constraint::Fill(1)]).areas(area);
    let human_mark = app.human_mark();
    let hint = match &app.hint {
        Hint::None => format!("h: {}", Msg::Hint.text(app.lang)),
        Hint::Moves { evaluation, .. } => match evaluation {
            Evaluation::Win(_) => Msg::HighlightWinning.text(app.lang).into(),
            Evaluation::Draw => Msg::OnlyDraw.text(app.lang).into(),
            Evaluation::Loss(_) => Msg::CannotAvoidLoss.text(app.lang).into(),
        },
    };
    let info_lines = vec![
        Line::from(vec![
            Span::styled(format!(" {} ", app.side_name(Side::Human)), fg(DIM)),
            Span::styled(mark_label(human_mark), fg(mark_color(human_mark))),
        ]),
        Line::from(vec![
            Span::styled(format!(" {} ", app.side_name(Side::Cpu)), fg(DIM)),
            Span::styled(mark_label(app.game.mark_for(Side::Cpu)), fg(mark_color(app.game.mark_for(Side::Cpu)))),
        ]),
        Line::from(vec![
            Span::styled(format!(" {} ", Msg::Difficulty.text(app.lang)), fg(DIM)),
            Span::styled(app.difficulty.label(app.lang), fg(TEXT)),
        ]),
        Line::styled(format!(" {hint}"), fg(ACCENT)),
    ];
    frame.render_widget(Paragraph::new(info_lines).block(panel(Msg::Status.text(app.lang))), info);

    let stats_line = app.lang.stats(app.stats.wins, app.stats.losses, app.stats.draws);
    frame.render_widget(
        Paragraph::new(vec![Line::raw(""), Line::styled(stats_line, fg(TEXT).add_modifier(Modifier::BOLD))])
            .block(panel(Msg::SessionScore.text(app.lang))),
        stats,
    );

    draw_move_log(frame, app, log);
}

fn draw_move_log(frame: &mut Frame, app: &App, area: Rect) {
    let rows = area.height.saturating_sub(2) as usize;
    let start = app.log.len().saturating_sub(rows);
    let lines = app.log[start..]
        .iter()
        .enumerate()
        .map(|(offset, &(side, mark, cell))| {
            Line::from(vec![
                Span::styled(format!(" {:>2}. ", start + offset + 1), fg(DIM)),
                Span::styled(format!("{} ", app.side_name(side)), fg(mark_color(mark))),
                Span::styled(format!("{} {}", mark_label(mark), cell_name(cell)), fg(TEXT)),
            ])
        })
        .collect::<Vec<_>>();
    frame.render_widget(Paragraph::new(lines).block(panel(Msg::MoveLog.text(app.lang))), area);
}

fn draw_help(frame: &mut Frame, app: &App, area: Rect) {
    let (items, split): (&[(&str, &str)], usize) = if app.replay().is_some() {
        (
            &[
                ("Space", Msg::PlayPause.text(app.lang)),
                ("←→", Msg::PreviousNext.text(app.lang)),
                ("Home/End", Msg::BeginningEnd.text(app.lang)),
                ("+/-", Msg::Speed.text(app.lang)),
                ("e", Msg::Evaluation.text(app.lang)),
                ("q", Msg::Quit.text(app.lang)),
            ],
            3,
        )
    } else if app.screen == Screen::GameOver {
        (
            &[
                ("Enter", Msg::NewGame.text(app.lang)),
                ("s", Msg::Save.text(app.lang)),
                ("t", Msg::Title.text(app.lang)),
                ("q", Msg::Quit.text(app.lang)),
            ],
            4,
        )
    } else {
        (
            &[
                (Msg::ArrowKeys.text(app.lang), Msg::Select.text(app.lang)),
                ("Enter", Msg::Place.text(app.lang)),
                ("h", Msg::Hint.text(app.lang)),
                ("e", Msg::Evaluation.text(app.lang)),
                ("s", Msg::Save.text(app.lang)),
                ("n", Msg::New.text(app.lang)),
                ("t", Msg::Title.text(app.lang)),
                ("q", Msg::Quit.text(app.lang)),
            ],
            4,
        )
    };
    let help_line = |row: &[(&str, &str)]| {
        let mut spans = Vec::new();
        for (button, description) in row {
            spans.push(key(button));
            spans.push(Span::styled(format!(" {description} "), fg(DIM)));
        }
        Line::from(spans)
    };
    let mut lines = vec![help_line(&items[..split])];
    if split < items.len() {
        lines.push(help_line(&items[split..]));
    }
    frame.render_widget(Paragraph::new(lines).alignment(Alignment::Center).block(panel("")), area);
}

fn draw_particles(buffer: &mut Buffer, app: &App, area: Rect) {
    for particle in &app.particles {
        if !(0.0..1.0).contains(&particle.x) || !(0.0..1.0).contains(&particle.y) {
            continue;
        }
        let x = area.x + (particle.x * f32::from(area.width)) as u16;
        let y = area.y + (particle.y * f32::from(area.height)) as u16;
        if x < area.right() && y < area.bottom() && buffer[(x, y)].symbol() == " " {
            buffer[(x, y)].set_char(particle.glyph).set_fg(c(particle.color));
        }
    }
}

fn draw_result(frame: &mut Frame, app: &App, rect: Rect) {
    let (headline, color) = match app.game.outcome() {
        Some(Outcome::Win { side, .. }) => (app.message.clone(), mark_color(app.game.mark_for(side))),
        Some(Outcome::Draw) => (app.message.clone(), PURPLE),
        None => (app.message.clone(), ACCENT),
    };
    let pulse = if (app.frame / 8).is_multiple_of(2) { color } else { shade(color, -70) };
    let mut lines = vec![
        Line::raw(""),
        Line::styled(headline, fg(color).add_modifier(Modifier::BOLD)),
        Line::raw(""),
        Line::styled(app.lang.moves_count(app.game.history.len()), fg(TEXT)),
        Line::raw(""),
    ];
    if app.replay().is_some() {
        lines.push(Line::from(vec![
            key("←"),
            Span::styled(format!(" {}   ", Msg::Back.text(app.lang)), fg(DIM)),
            key("q"),
            Span::styled(format!(" {}", Msg::Quit.text(app.lang)), fg(DIM)),
        ]));
    } else {
        lines.push(Line::from(vec![
            key("Enter"),
            Span::styled(format!(" {}   ", Msg::NewGame.text(app.lang)), fg(DIM)),
            key("t"),
            Span::styled(format!(" {}", Msg::Title.text(app.lang)), fg(DIM)),
        ]));
    }
    frame.render_widget(Clear, rect);
    frame.render_widget(
        Paragraph::new(lines).alignment(Alignment::Center).block(
            Block::bordered()
                .border_type(BorderType::Double)
                .border_style(fg(pulse))
                .style(Style::new().bg(c(BG))),
        ),
        rect,
    );
}

fn mark_label(mark: Mark) -> &'static str {
    match mark {
        Mark::X => "X",
        Mark::O => "O",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::{App, Screen};
    use crate::game::{Game, Side};
    use crate::i18n::{Lang, Msg};
    use crate::record::Record;
    use ratatui::crossterm::event::{KeyCode, KeyEvent};
    use ratatui::{Terminal, backend::TestBackend};
    use std::time::Duration;

    fn render(app: &App, width: u16, height: u16) -> String {
        let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
        terminal.draw(|frame| draw(frame, app)).unwrap();
        let buffer = terminal.backend().buffer();
        let mut output = String::new();
        for y in 0..height {
            let mut skip = 0;
            for x in 0..width {
                if skip > 0 {
                    skip -= 1;
                    continue;
                }
                let symbol = buffer[(x, y)].symbol();
                output.push_str(symbol);
                skip = symbol
                    .chars()
                    .map(|character| {
                        if !character.is_ascii() && character as u32 >= 0x1100 && character as u32 > 0x2fff {
                            2
                        } else {
                            1
                        }
                    })
                    .sum::<usize>()
                    .saturating_sub(1);
            }
            output.push('\n');
        }
        output
    }

    fn render_buffer(app: &App, width: u16, height: u16) -> Buffer {
        let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
        terminal.draw(|frame| draw(frame, app)).unwrap();
        terminal.backend().buffer().clone()
    }

    fn assert_within_width(screen: &str, width: u16) {
        for line in screen.lines() {
            assert!(Line::raw(line).width() <= usize::from(width), "line exceeds {width} columns: {line:?}");
        }
    }

    fn completed_app(first: Side, moves: &[usize], message: &str, lang: Lang) -> App {
        let mut app = App::with_lang(lang);
        app.first = first;
        app.game = Game::new(first);
        for &cell in moves {
            app.game.apply(cell).unwrap();
        }
        app.screen = Screen::GameOver;
        app.message = message.into();
        app
    }

    #[test]
    fn minimum_size_renders_title_playing_and_evaluation_panel() {
        for (lang, start, turn, score, title_label, quit, evaluation_label, draw_label) in [
            (Lang::En, "Start game", "Your turn", "Session score", "Title", "Quit", "Position", "Draw"),
            (
                Lang::Ja,
                "ゲーム開始",
                "あなたの番",
                "セッション戦績",
                "タイトル",
                "終了",
                "局面評価",
                "引分",
            ),
        ] {
            let mut app = App::with_lang(lang);
            let screen = render(&app, MIN_WIDTH, MIN_HEIGHT);
            assert!(screen.contains("rs-tic-tac-toe"), "{screen}");
            assert!(screen.contains(start), "{screen}");
            assert!(screen.contains(Msg::Language.text(lang)), "{screen}");
            if lang == Lang::En {
                assert!(screen.contains("First player  You"), "{screen}");
            }
            assert_within_width(&screen, MIN_WIDTH);

            app.start();
            let screen = render(&app, MIN_WIDTH, MIN_HEIGHT);
            for expected in [turn, score, title_label, quit] {
                assert!(screen.contains(expected), "missing {expected:?}: {screen}");
            }
            if lang == Lang::En {
                assert!(!screen.contains("Turn Your turn"), "{screen}");
            }
            assert_within_width(&screen, MIN_WIDTH);

            app.show_evaluation = true;
            let screen = render(&app, MIN_WIDTH, MIN_HEIGHT);
            assert!(screen.contains(evaluation_label), "{screen}");
            assert!(screen.contains(draw_label), "{screen}");
            assert_within_width(&screen, MIN_WIDTH);
        }
    }

    #[test]
    fn hint_explains_draw_only_and_forced_loss_positions() {
        let record = Record {
            first: Side::Cpu,
            names: [None, None],
            moves: vec![0, 4, 8, 2, 6],
        };
        for (lang, draw_text, loss_text) in [
            (Lang::En, "Only a draw is possible", "A loss cannot be avoided"),
            (Lang::Ja, "引き分けしかありません", "負けを避けられません"),
        ] {
            let mut draw = App::with_lang(lang);
            draw.start();
            draw.on_key(KeyEvent::from(KeyCode::Char('h')));
            let screen = render(&draw, MIN_WIDTH, MIN_HEIGHT);
            assert!(screen.contains(draw_text), "{screen}");

            let mut loss = App::with_lang(lang);
            loss.first = Side::Cpu;
            loss.start();
            loss.game = record.game_at(record.moves.len()).unwrap();
            loss.on_key(KeyEvent::from(KeyCode::Char('h')));
            let screen = render(&loss, MIN_WIDTH, MIN_HEIGHT);
            assert!(screen.contains(loss_text), "{screen}");
        }
    }

    #[test]
    fn undersized_terminal_shows_resize_message() {
        for (lang, expected) in [(Lang::En, "Resize the terminal"), (Lang::Ja, "以上に広げてください")] {
            let screen = render(&App::with_lang(lang), MIN_WIDTH - 1, MIN_HEIGHT - 1);
            assert!(screen.contains(expected), "{screen}");
            assert_within_width(&screen, MIN_WIDTH - 1);
        }
    }

    #[test]
    fn minimum_size_renders_win_loss_and_draw_results() {
        for lang in [Lang::En, Lang::Ja] {
            let win = completed_app(Side::Human, &[0, 3, 1, 4, 2], Msg::HumanWins.text(lang), lang);
            let screen = render(&win, MIN_WIDTH, MIN_HEIGHT);
            assert!(screen.contains(Msg::HumanWins.text(lang)), "{screen}");
            assert_within_width(&screen, MIN_WIDTH);

            let loss = completed_app(Side::Cpu, &[0, 3, 1, 4, 2], Msg::CpuWins.text(lang), lang);
            let screen = render(&loss, MIN_WIDTH, MIN_HEIGHT);
            assert!(screen.contains(Msg::CpuWins.text(lang)), "{screen}");
            assert_within_width(&screen, MIN_WIDTH);

            let draw = completed_app(Side::Human, &[0, 1, 2, 4, 3, 5, 7, 6, 8], Msg::Draw.text(lang), lang);
            let screen = render(&draw, MIN_WIDTH, MIN_HEIGHT);
            assert!(screen.contains(Msg::Draw.text(lang)), "{screen}");
            assert_within_width(&screen, MIN_WIDTH);
        }
    }

    fn replay_record(complete: bool) -> Record {
        Record {
            first: Side::Human,
            names: [Some("Alice".into()), Some("Bob".into())],
            moves: if complete { vec![0, 3, 1, 4, 2] } else { vec![0, 4, 8] },
        }
    }

    #[test]
    fn minimum_size_renders_replay_midway_complete_and_unfinished() {
        for (lang, paused, winner, end) in [
            (Lang::En, "Paused", "Alice wins", "End of record"),
            (Lang::Ja, "一時停止", "Alice の勝ち", "棋譜の終わり"),
        ] {
            let mut midway = App::new_replay(replay_record(true), Duration::from_millis(1_500), true, false, lang);
            midway.on_key(KeyEvent::from(KeyCode::Right));
            let screen = render(&midway, MIN_WIDTH, MIN_HEIGHT);
            assert!(screen.contains("Alice"), "{screen}");
            assert!(screen.contains(paused), "{screen}");
            assert!(screen.contains(if lang == Lang::En { "Move 1/5" } else { "手数 1/5" }), "{screen}");
            assert_within_width(&screen, MIN_WIDTH);

            let mut complete = App::new_replay(replay_record(true), Duration::from_millis(1_500), true, false, lang);
            complete.on_key(KeyEvent::from(KeyCode::End));
            assert!(render(&complete, MIN_WIDTH, MIN_HEIGHT).contains(winner));

            let mut unfinished = App::new_replay(replay_record(false), Duration::from_millis(1_500), true, false, lang);
            unfinished.on_key(KeyEvent::from(KeyCode::End));
            assert!(render(&unfinished, MIN_WIDTH, MIN_HEIGHT).contains(end));
        }
    }

    #[test]
    fn game_over_result_stays_in_side_column_and_leaves_winning_cells_visible() {
        for lang in [Lang::En, Lang::Ja] {
            let normal = completed_app(Side::Human, &[0, 3, 1, 4, 2], Msg::HumanWins.text(lang), lang);
            let mut replay = App::new_replay(replay_record(true), Duration::from_millis(1_500), true, false, lang);
            replay.on_key(KeyEvent::from(KeyCode::End));

            for app in [&normal, &replay] {
                let buffer = render_buffer(app, MIN_WIDTH, MIN_HEIGHT);
                let result_left = buffer
                    .content
                    .iter()
                    .enumerate()
                    .filter(|(_, cell)| cell.symbol() == "╔")
                    .map(|(index, _)| index as u16 % MIN_WIDTH)
                    .min()
                    .expect("result panel has a double border");
                let side_left = MIN_WIDTH - 32;
                assert!(result_left >= side_left, "result starts at {result_left}, before side column {side_left}");

                // The winning X marks fill five centre-tint cells in each top-row board cell.
                // A popup over the board clears some of those cells.
                let board_origin = (6, 4);
                for cell in 0..3 {
                    let cell_x = board_origin.0 + cell * 12;
                    let mark_pixels = (board_origin.1..board_origin.1 + 5)
                        .flat_map(|y| (cell_x..cell_x + 11).map(move |x| (x, y)))
                        .filter(|&(x, y)| buffer[(x, y)].bg == c(X_COLOR))
                        .count();
                    assert_eq!(mark_pixels, 5, "winning cell {cell} was obscured");
                }
            }
        }
    }

    #[test]
    fn minimum_size_game_over_result_keeps_all_text_in_both_modes_and_languages() {
        for lang in [Lang::En, Lang::Ja] {
            let normal = completed_app(Side::Human, &[0, 3, 1, 4, 2], Msg::HumanWins.text(lang), lang);
            let normal_screen = render(&normal, MIN_WIDTH, MIN_HEIGHT);
            for expected in [
                Msg::HumanWins.text(lang),
                Msg::NewGame.text(lang),
                Msg::Title.text(lang),
                if lang == Lang::En { "Moves 5" } else { "手数 5" },
            ] {
                assert!(normal_screen.contains(expected), "missing {expected:?}: {normal_screen}");
            }

            let mut replay = App::new_replay(replay_record(true), Duration::from_millis(1_500), true, false, lang);
            replay.on_key(KeyEvent::from(KeyCode::End));
            let replay_screen = render(&replay, MIN_WIDTH, MIN_HEIGHT);
            for expected in [
                if lang == Lang::En { "Alice wins" } else { "Alice の勝ち" },
                Msg::Back.text(lang),
                Msg::Quit.text(lang),
                if lang == Lang::En { "Moves 5" } else { "手数 5" },
            ] {
                assert!(replay_screen.contains(expected), "missing {expected:?}: {replay_screen}");
            }
        }
    }

    #[test]
    fn replay_omits_play_only_side_information_while_normal_play_keeps_it() {
        for lang in [Lang::En, Lang::Ja] {
            let mut normal = App::with_lang(lang);
            normal.start();
            let normal_screen = render(&normal, MIN_WIDTH, MIN_HEIGHT);
            for expected in [
                Msg::Difficulty.text(lang),
                Msg::SessionScore.text(lang),
                if lang == Lang::En { "h: Hint" } else { "h: ヒント" },
            ] {
                assert!(normal_screen.contains(expected), "missing {expected:?}: {normal_screen}");
            }

            let replay = App::new_replay(replay_record(true), Duration::from_millis(1_500), true, false, lang);
            let replay_screen = render(&replay, MIN_WIDTH, MIN_HEIGHT);
            for absent in [
                Msg::Difficulty.text(lang),
                Msg::SessionScore.text(lang),
                if lang == Lang::En { "h: Hint" } else { "h: ヒント" },
            ] {
                assert!(!replay_screen.contains(absent), "unexpected {absent:?}: {replay_screen}");
            }
        }
    }

    #[test]
    fn replay_header_is_finished_at_end_and_playing_after_step_back() {
        for (lang, finished, playing) in [(Lang::En, "Finished", "Playing"), (Lang::Ja, "再生終了", "再生中")] {
            for paused in [false, true] {
                let mut app = App::new_replay(replay_record(true), Duration::from_millis(1_500), paused, false, lang);
                app.on_key(KeyEvent::from(KeyCode::End));
                let screen = render(&app, MIN_WIDTH, MIN_HEIGHT);
                assert!(screen.contains(finished), "missing {finished:?}: {screen}");

                app.on_key(KeyEvent::from(KeyCode::Left));
                let expected = if paused { Msg::Paused.text(lang) } else { playing };
                let screen = render(&app, MIN_WIDTH, MIN_HEIGHT);
                assert!(screen.contains(expected), "missing {expected:?}: {screen}");
                assert!(!screen.contains(finished), "unexpected {finished:?}: {screen}");
            }
        }
    }

    #[test]
    #[ignore]
    fn dump_screens() {
        for lang in [Lang::En, Lang::Ja] {
            let mut app = App::with_lang(lang);
            println!("{}", render(&app, MIN_WIDTH, MIN_HEIGHT));
            app.start();
            app.show_evaluation = true;
            println!("{}", render(&app, MIN_WIDTH, MIN_HEIGHT));

            let mut replay = App::new_replay(replay_record(true), Duration::from_millis(1_500), false, false, lang);
            replay.on_key(KeyEvent::from(KeyCode::End));
            println!("{}", render(&replay, MIN_WIDTH, MIN_HEIGHT));
        }
    }

    #[test]
    #[ignore]
    fn plays_one_game_against_cpu() {
        for lang in [Lang::En, Lang::Ja] {
            let mut app = App::with_lang(lang);
            app.first = Side::Cpu;
            app.start();
            let started = std::time::Instant::now();
            while app.screen == Screen::Playing && started.elapsed() < Duration::from_secs(30) {
                if app.human_can_act()
                    && let Some(cell) = app.game.legal_moves().first().copied()
                {
                    app.cursor = cell;
                    app.on_key(KeyEvent::from(KeyCode::Enter));
                }
                app.on_tick();
                let screen = render(&app, MIN_WIDTH, MIN_HEIGHT);
                assert_within_width(&screen, MIN_WIDTH);
                std::thread::sleep(Duration::from_millis(33));
            }
            assert_eq!(app.screen, Screen::GameOver);
            println!("result = {}, moves = {:?}", app.message, app.log);
        }
    }
}
