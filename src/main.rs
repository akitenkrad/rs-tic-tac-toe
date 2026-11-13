use std::fmt::Write as _;
use std::io::{self, Read as _, Write as _};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::{Duration, Instant};

use clap::{Parser, Subcommand};
use ratatui::crossterm::event::{self, Event, KeyEventKind};
use rs_tic_tac_toe::app::App;
use rs_tic_tac_toe::game::{Game, Mark, Outcome};
use rs_tic_tac_toe::i18n::{Lang, Msg};
use rs_tic_tac_toe::record::Record;
use rs_tic_tac_toe::{record, ui};

const TICK: Duration = Duration::from_millis(33);
const DEFAULT_REPLAY_INTERVAL_MS: u64 = 1_500;

#[derive(Debug, Parser)]
#[command(name = "rs-tic-tac-toe")]
struct Cli {
    /// Directory used when saving records from a normal game.
    #[arg(long, default_value = ".", global = true)]
    record_dir: PathBuf,

    /// Display language.
    #[arg(long, default_value = "en", global = true, value_parser = parse_lang)]
    lang: Lang,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Replay a .tttrec game record.
    Replay {
        /// Record file, or - to read from standard input.
        file: PathBuf,

        /// Automatic playback interval in milliseconds.
        #[arg(long, default_value_t = DEFAULT_REPLAY_INTERVAL_MS)]
        interval: u64,

        /// Start playback paused.
        #[arg(long)]
        paused: bool,

        /// Exit three seconds after reaching a finished record's end.
        #[arg(long)]
        exit_on_end: bool,

        /// Print the replay as text instead of starting the TUI.
        #[arg(long)]
        print: bool,
    },
}

fn main() -> ExitCode {
    match execute(Cli::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("rs-tic-tac-toe: {error}");
            ExitCode::from(1)
        }
    }
}

fn execute(cli: Cli) -> Result<(), String> {
    let lang = cli.lang;
    match cli.command {
        None => {
            let mut app = App::with_lang(lang);
            app.set_record_dir(cli.record_dir);
            run_tui(app).map_err(|error| error.to_string())
        }
        Some(Command::Replay {
            file,
            interval,
            paused,
            exit_on_end,
            print,
        }) => {
            let text = read_record_text(&file)?;
            let record = record::parse(&text).map_err(|error| format!("{}: {error}", file.display()))?;
            if print {
                io::stdout()
                    .write_all(format_replay(&record, lang).as_bytes())
                    .map_err(|error| format!("failed to write replay: {error}"))?;
                return Ok(());
            }
            let app = App::new_replay(record, Duration::from_millis(interval), paused, exit_on_end, lang);
            run_tui(app).map_err(|error| error.to_string())
        }
    }
}

fn read_record_text(path: &Path) -> Result<String, String> {
    if path.as_os_str() == "-" {
        let mut text = String::new();
        io::stdin()
            .read_to_string(&mut text)
            .map_err(|error| format!("failed to read stdin: {error}"))?;
        Ok(text)
    } else {
        std::fs::read_to_string(path).map_err(|error| format!("failed to read {}: {error}", path.display()))
    }
}

fn parse_lang(value: &str) -> Result<Lang, String> {
    Lang::from_str(value).ok_or_else(|| "language must be 'en' or 'ja'".into())
}

fn format_replay(record: &Record, lang: Lang) -> String {
    let mut output = String::new();
    let mut game = Game::new(record.first);
    for (index, &cell) in record.moves.iter().enumerate() {
        let side = game.to_move;
        let mark = game.mark_for(side);
        game.apply(cell).expect("parsed record moves are legal");
        let _ = writeln!(
            output,
            "{}",
            lang.move_line(
                index + 1,
                record.name(side, lang),
                mark_label(mark),
                &rs_tic_tac_toe::game::cell_name(cell)
            )
        );
        output.push_str(&format_board(&game));
    }
    match game.outcome() {
        Some(Outcome::Win { side, .. }) => {
            let _ = writeln!(output, "{}", lang.winner_line(record.name(side, lang)));
        }
        Some(Outcome::Draw) => {
            output.push_str(Msg::ResultDraw.text(lang));
            output.push('\n');
        }
        None => {
            output.push_str(Msg::ResultUnfinished.text(lang));
            output.push('\n');
        }
    }
    output
}

fn format_board(game: &Game) -> String {
    let mut output = String::new();
    for row in 0..3 {
        for col in 0..3 {
            if col > 0 {
                output.push(' ');
            }
            output.push(match game.board[row * 3 + col] {
                Some(Mark::X) => 'X',
                Some(Mark::O) => 'O',
                None => '.',
            });
        }
        output.push('\n');
    }
    output
}

fn mark_label(mark: Mark) -> &'static str {
    match mark {
        Mark::X => "X",
        Mark::O => "O",
    }
}

struct RestoreTerminal;

impl Drop for RestoreTerminal {
    fn drop(&mut self) {
        ratatui::restore();
    }
}

fn run_tui(app: App) -> io::Result<()> {
    let mut terminal = ratatui::init();
    let _restore = RestoreTerminal;
    run(&mut terminal, app)
}

fn run(terminal: &mut ratatui::DefaultTerminal, mut app: App) -> io::Result<()> {
    let mut last_tick = Instant::now();
    while !app.should_quit {
        terminal.draw(|frame| ui::draw(frame, &app))?;
        let timeout = TICK.saturating_sub(last_tick.elapsed());
        if event::poll(timeout)?
            && let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            app.on_key(key);
        }
        if last_tick.elapsed() >= TICK {
            app.on_tick();
            last_tick = Instant::now();
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;
    use rs_tic_tac_toe::game::Side;
    use rs_tic_tac_toe::i18n::Lang;
    use rs_tic_tac_toe::record::{Record, parse};
    use std::path::PathBuf;

    #[test]
    fn cli_defaults_to_normal_play_in_current_directory() {
        let cli = Cli::try_parse_from(["rs-tic-tac-toe"]).unwrap();
        assert_eq!(cli.record_dir, PathBuf::from("."));
        assert_eq!(cli.lang, Lang::En);
        assert!(cli.command.is_none());
    }

    #[test]
    fn replay_cli_defaults_and_all_options() {
        let cli = Cli::try_parse_from(["rs-tic-tac-toe", "replay", "game.tttrec"]).unwrap();
        let Some(Command::Replay {
            interval,
            paused,
            exit_on_end,
            print,
            ..
        }) = cli.command
        else {
            panic!("expected replay command");
        };
        assert_eq!(interval, 1_500);
        assert!(!paused);
        assert!(!exit_on_end);
        assert!(!print);

        let cli = Cli::try_parse_from([
            "rs-tic-tac-toe",
            "--record-dir",
            "records",
            "replay",
            "-",
            "--interval",
            "250",
            "--paused",
            "--exit-on-end",
            "--print",
            "--lang",
            "ja",
        ])
        .unwrap();
        assert_eq!(cli.record_dir, PathBuf::from("records"));
        assert_eq!(cli.lang, Lang::Ja);
        let Some(Command::Replay {
            file,
            interval,
            paused,
            exit_on_end,
            print,
        }) = cli.command
        else {
            panic!("expected replay command");
        };
        assert_eq!(file, PathBuf::from("-"));
        assert_eq!(interval, 250);
        assert!(paused && exit_on_end && print);
    }

    #[test]
    fn print_replay_contains_each_move_board_and_final_result() {
        let record = Record {
            first: Side::Human,
            names: [Some("Alice".into()), Some("Bob".into())],
            moves: vec![0, 3, 1, 4, 2],
        };
        let english = format_replay(&record, Lang::En);
        assert!(english.contains("Move 1: Alice X a1"), "{english}");
        assert!(english.contains("X . .\n. . .\n. . ."), "{english}");
        assert!(english.contains("Move 2: Bob O a2"), "{english}");
        assert!(english.contains("Winner: Alice"), "{english}");

        let japanese = format_replay(&record, Lang::Ja);
        assert!(japanese.contains("手数 1: Alice X a1"), "{japanese}");
        assert!(japanese.contains("勝者: Alice"), "{japanese}");
    }

    #[test]
    fn print_replay_distinguishes_draw_and_unfinished() {
        let draw = Record {
            first: Side::Human,
            names: [Some("Alice".into()), Some("Bob".into())],
            moves: vec![0, 1, 2, 4, 3, 5, 7, 6, 8],
        };
        assert!(format_replay(&draw, Lang::En).ends_with("Result: draw\n"));
        assert!(format_replay(&draw, Lang::Ja).ends_with("結果: 引き分け\n"));

        let unfinished = Record {
            first: Side::Cpu,
            names: [Some("Alice".into()), Some("Bob".into())],
            moves: vec![4, 0],
        };
        assert!(format_replay(&unfinished, Lang::En).ends_with("Result: unfinished\n"));
        assert!(format_replay(&unfinished, Lang::Ja).ends_with("結果: 未終局\n"));

        let defaults = parse("a1\n").unwrap();
        assert!(format_replay(&defaults, Lang::En).starts_with("Move 1: You X a1\n"));
        assert!(format_replay(&defaults, Lang::Ja).starts_with("手数 1: あなた X a1\n"));
    }
}
