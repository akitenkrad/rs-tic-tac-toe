//! 画面遷移・入力・アニメーションの状態管理．

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::ai::{self, Difficulty, Rng};
use crate::game::{Evaluation, Game, Mark, Outcome, Side, best_moves, solve};
use crate::record::{Record, to_text};

/// CPU が考えているふりをする時間．
const CPU_THINK: Duration = Duration::from_millis(650);
/// CPU が選んだマスを見せる時間．
const CPU_SHOW: Duration = Duration::from_millis(500);
/// マークが現れるアニメーションの長さ．
pub const PLACEMENT_ANIM: Duration = Duration::from_millis(480);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Screen {
    Title,
    Playing,
    GameOver,
}

/// 通常対局または棋譜リプレイの実行モード．
pub enum Mode {
    Normal,
    Replay(ReplayState),
}

pub struct ReplayState {
    pub record: Record,
    pub position: usize,
    pub paused: bool,
    pub interval: Duration,
    pub exit_on_end: bool,
    next_advance_at: Instant,
}

/// 置いたマークが現れるアニメーション．
pub struct Placement {
    pub cell: usize,
    pub mark: Mark,
    pub by: Side,
    pub started: Instant,
}

impl Placement {
    pub fn progress(&self) -> f32 {
        (self.started.elapsed().as_secs_f32() / PLACEMENT_ANIM.as_secs_f32()).min(1.0)
    }
}

pub struct CpuTurn {
    pub cell: usize,
    pub started: Instant,
    pub showing: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Hint {
    None,
    Moves { cells: Vec<usize>, evaluation: Evaluation },
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Stats {
    pub wins: u32,
    pub losses: u32,
    pub draws: u32,
}

/// 紙吹雪や雨粒．座標は画面全体を `[0, 1]` に正規化する．
pub struct Particle {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub glyph: char,
    pub color: (u8, u8, u8),
}

pub struct App {
    pub mode: Mode,
    pub screen: Screen,
    pub game: Game,
    pub difficulty: Difficulty,
    pub first: Side,
    pub cursor: usize,
    pub selected_number: Option<usize>,
    pub show_evaluation: bool,
    pub hint: Hint,
    pub log: Vec<(Side, Mark, usize)>,
    pub message: String,
    pub placement: Option<Placement>,
    pub cpu: Option<CpuTurn>,
    pub particles: Vec<Particle>,
    pub frame: u64,
    pub game_over_at: Option<Instant>,
    pub stats: Stats,
    pub should_quit: bool,
    record_dir: PathBuf,
    rng: Rng,
}

const CONFETTI: [(u8, u8, u8); 6] = [
    (236, 112, 99),
    (245, 176, 65),
    (72, 201, 176),
    (93, 173, 226),
    (165, 105, 189),
    (240, 98, 146),
];

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        Self {
            mode: Mode::Normal,
            screen: Screen::Title,
            game: Game::new(Side::Human),
            difficulty: Difficulty::Normal,
            first: Side::Human,
            cursor: 4,
            selected_number: None,
            show_evaluation: false,
            hint: Hint::None,
            log: Vec::new(),
            message: String::new(),
            placement: None,
            cpu: None,
            particles: Vec::new(),
            frame: 0,
            game_over_at: None,
            stats: Stats::default(),
            should_quit: false,
            record_dir: PathBuf::from("."),
            rng: Rng::seeded(),
        }
    }

    /// 棋譜を先頭局面から再生する App を作る．
    pub fn new_replay(record: Record, interval: Duration, paused: bool, exit_on_end: bool) -> Self {
        let interval = interval.max(PLACEMENT_ANIM);
        let game = record.game_at(0).expect("parsed records have an initial position");
        let mut app = Self::new();
        app.screen = Screen::Playing;
        app.first = record.first;
        app.game = game;
        app.message = "棋譜の先頭です".into();
        app.mode = Mode::Replay(ReplayState {
            record,
            position: 0,
            paused,
            interval,
            exit_on_end,
            next_advance_at: Instant::now() + interval,
        });
        app
    }

    pub fn set_record_dir(&mut self, dir: impl Into<PathBuf>) {
        self.record_dir = dir.into();
    }

    pub fn replay(&self) -> Option<&ReplayState> {
        match &self.mode {
            Mode::Normal => None,
            Mode::Replay(replay) => Some(replay),
        }
    }

    fn replay_mut(&mut self) -> Option<&mut ReplayState> {
        match &mut self.mode {
            Mode::Normal => None,
            Mode::Replay(replay) => Some(replay),
        }
    }

    pub fn side_name(&self, side: Side) -> &str {
        match &self.mode {
            Mode::Normal => side.label(),
            Mode::Replay(replay) => match side {
                Side::Human => &replay.record.names[0],
                Side::Cpu => &replay.record.names[1],
            },
        }
    }

    pub fn human_mark(&self) -> Mark {
        self.game.mark_for(Side::Human)
    }

    pub fn human_can_act(&self) -> bool {
        matches!(self.mode, Mode::Normal)
            && self.screen == Screen::Playing
            && self.game.to_move == Side::Human
            && self.placement.is_none()
            && self.cpu.is_none()
            && self.game.outcome().is_none()
    }

    pub fn on_key(&mut self, key: KeyEvent) {
        let ctrl_c = key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c');
        if ctrl_c || key.code == KeyCode::Char('q') {
            self.should_quit = true;
            return;
        }
        if matches!(self.mode, Mode::Replay(_)) {
            if key.code == KeyCode::Esc {
                self.should_quit = true;
            } else {
                self.on_replay_key(key.code);
            }
            return;
        }
        match self.screen {
            Screen::Title => self.on_title_key(key.code),
            Screen::Playing => self.on_play_key(key.code),
            Screen::GameOver => match key.code {
                KeyCode::Enter | KeyCode::Char(' ') | KeyCode::Char('n') => self.start(),
                KeyCode::Char('t') | KeyCode::Esc => self.screen = Screen::Title,
                KeyCode::Char('s') => self.save_record(),
                _ => {}
            },
        }
    }

    fn on_title_key(&mut self, code: KeyCode) {
        match code {
            KeyCode::Enter | KeyCode::Char(' ') => self.start(),
            KeyCode::Char('d') => self.difficulty = self.difficulty.next(),
            KeyCode::Char('f') => self.first = self.first.other(),
            _ => {}
        }
    }

    fn on_play_key(&mut self, code: KeyCode) {
        match code {
            KeyCode::Char('e') => self.show_evaluation = !self.show_evaluation,
            KeyCode::Char('s') => self.save_record(),
            KeyCode::Char('n') => self.start(),
            KeyCode::Char('t') | KeyCode::Esc => self.screen = Screen::Title,
            _ if !self.human_can_act() => {}
            KeyCode::Left => self.move_cursor(-1, 0),
            KeyCode::Right => self.move_cursor(1, 0),
            KeyCode::Up => self.move_cursor(0, -1),
            KeyCode::Down => self.move_cursor(0, 1),
            KeyCode::Char(c @ '1'..='9') => {
                let cell = usize::from(c as u8 - b'1');
                if self.selected_number == Some(cell) && self.cursor == cell {
                    self.perform(cell);
                } else {
                    self.cursor = cell;
                    self.selected_number = Some(cell);
                }
            }
            KeyCode::Enter | KeyCode::Char(' ') => self.perform(self.cursor),
            KeyCode::Char('h') => self.show_hint(),
            _ => {}
        }
    }

    fn move_cursor(&mut self, dx: isize, dy: isize) {
        let col = (self.cursor % 3) as isize;
        let row = (self.cursor / 3) as isize;
        let col = (col + dx).rem_euclid(3);
        let row = (row + dy).rem_euclid(3);
        self.cursor = (row * 3 + col) as usize;
        self.selected_number = None;
    }

    fn show_hint(&mut self) {
        let cells = best_moves(&self.game);
        let evaluations = solve(&self.game);
        let Some(evaluation) = cells.first().and_then(|&cell| evaluations[cell]) else {
            return;
        };
        self.cursor = cells[0];
        self.hint = Hint::Moves { cells, evaluation };
    }

    pub fn start(&mut self) {
        self.mode = Mode::Normal;
        self.game = Game::new(self.first);
        self.cursor = 4;
        self.selected_number = None;
        self.hint = Hint::None;
        self.log.clear();
        self.placement = None;
        self.cpu = None;
        self.particles.clear();
        self.game_over_at = None;
        self.screen = Screen::Playing;
        self.update_message();
    }

    fn perform(&mut self, cell: usize) {
        let by = self.game.to_move;
        let mark = self.game.mark_for(by);
        if self.game.apply(cell).is_err() {
            self.message = "そのマスには置けません".into();
            return;
        }
        self.log.push((by, mark, cell));
        self.cursor = cell;
        self.selected_number = None;
        self.hint = Hint::None;
        self.cpu = None;
        self.placement = Some(Placement {
            cell,
            mark,
            by,
            started: Instant::now(),
        });
        self.message = format!("{} が {} に置きました", self.side_name(by), crate::game::cell_name(cell));
    }

    fn update_message(&mut self) {
        self.message = match self.game.to_move {
            Side::Human => "あなたの番です．マスを選んで Enter".into(),
            Side::Cpu => "CPU が考えています".into(),
        };
    }

    pub fn on_tick(&mut self) {
        self.frame = self.frame.wrapping_add(1);
        self.update_particles();

        if self.screen == Screen::GameOver
            && self.replay().is_some_and(|replay| replay.exit_on_end)
            && self.game.outcome().is_some()
            && self.game_over_at.is_some_and(|at| at.elapsed() >= Duration::from_secs(3))
        {
            self.should_quit = true;
        }
        if self.screen != Screen::Playing {
            return;
        }
        if matches!(self.mode, Mode::Replay(_)) {
            self.on_replay_tick();
            return;
        }
        if let Some(placement) = &self.placement {
            if placement.started.elapsed() >= PLACEMENT_ANIM {
                self.placement = None;
                if self.game.outcome().is_some() {
                    self.finish_normal();
                } else {
                    self.update_message();
                }
            }
            return;
        }
        if self.game.to_move == Side::Cpu {
            match &mut self.cpu {
                None => {
                    if let Some(cell) = ai::choose(&self.game, self.difficulty, &mut self.rng) {
                        self.cpu = Some(CpuTurn {
                            cell,
                            started: Instant::now(),
                            showing: false,
                        });
                    }
                }
                Some(turn) => {
                    let elapsed = turn.started.elapsed();
                    if !turn.showing && elapsed >= CPU_THINK {
                        turn.showing = true;
                        self.cursor = turn.cell;
                    } else if turn.showing && elapsed >= CPU_THINK + CPU_SHOW {
                        let cell = turn.cell;
                        self.perform(cell);
                    }
                }
            }
        }
    }

    fn finish_normal(&mut self) {
        if self.screen == Screen::GameOver {
            return;
        }
        let Some(outcome) = self.game.outcome() else {
            return;
        };
        self.screen = Screen::GameOver;
        self.game_over_at = Some(Instant::now());
        match outcome {
            Outcome::Win { side: Side::Human, .. } => {
                self.stats.wins += 1;
                self.message = "あなたの勝ち！".into();
                for _ in 0..140 {
                    self.spawn_confetti(-0.6);
                }
            }
            Outcome::Win { side: Side::Cpu, .. } => {
                self.stats.losses += 1;
                self.message = "CPU の勝ち".into();
            }
            Outcome::Draw => {
                self.stats.draws += 1;
                self.message = "引き分け".into();
            }
        }
    }

    fn on_replay_key(&mut self, code: KeyCode) {
        match code {
            KeyCode::Char(' ') => {
                let now = Instant::now();
                let placement_active = self.placement.is_some();
                if let Some(replay) = self.replay_mut() {
                    replay.paused = !replay.paused;
                    if !replay.paused && !placement_active {
                        replay.next_advance_at = now + replay.interval;
                    }
                }
            }
            KeyCode::Right | KeyCode::Char('l') if self.placement.is_none() => self.replay_step_forward(),
            KeyCode::Left | KeyCode::Char('h') => {
                let position = self.replay().map_or(0, |state| state.position.saturating_sub(1));
                self.replay_go_to(position);
            }
            KeyCode::Home | KeyCode::Char('g') => self.replay_go_to(0),
            KeyCode::End | KeyCode::Char('G') => {
                let end = self.replay().map_or(0, |state| state.record.moves.len());
                self.replay_go_to(end);
            }
            KeyCode::Char('+') | KeyCode::Char('=') => self.change_replay_speed(true),
            KeyCode::Char('-') => self.change_replay_speed(false),
            KeyCode::Char('e') => self.show_evaluation = !self.show_evaluation,
            _ => {}
        }
    }

    fn change_replay_speed(&mut self, faster: bool) {
        const STEP: Duration = Duration::from_millis(250);
        let now = Instant::now();
        if let Some(replay) = self.replay_mut() {
            replay.interval = if faster {
                replay.interval.saturating_sub(STEP).max(PLACEMENT_ANIM)
            } else {
                replay.interval.saturating_add(STEP)
            };
            replay.next_advance_at = now + replay.interval;
        }
    }

    fn replay_step_forward(&mut self) {
        let Some((position, cell)) = self
            .replay()
            .and_then(|state| state.record.moves.get(state.position).copied().map(|cell| (state.position, cell)))
        else {
            return;
        };
        self.perform(cell);
        if let Some(replay) = self.replay_mut() {
            replay.position = position + 1;
        }
    }

    fn replay_go_to(&mut self, position: usize) {
        let Some((position, game, log, interval, at_end)) = self.replay().map(|state| {
            let position = position.min(state.record.moves.len());
            let game = state.record.game_at(position).expect("position is within the parsed record");
            let log = game.history.iter().map(|&(side, cell)| (side, game.mark_for(side), cell)).collect();
            (position, game, log, state.interval, position == state.record.moves.len())
        }) else {
            return;
        };
        self.game = game;
        self.log = log;
        self.cursor = self.game.history.last().map_or(4, |&(_, cell)| cell);
        self.placement = None;
        self.cpu = None;
        self.hint = Hint::None;
        self.particles.clear();
        self.game_over_at = None;
        self.screen = Screen::Playing;
        if let Some(replay) = self.replay_mut() {
            replay.position = position;
            replay.next_advance_at = Instant::now() + interval;
        }
        if at_end {
            self.finish_replay();
        } else {
            self.message = format!("{position} 手目を表示しています");
        }
    }

    fn on_replay_tick(&mut self) {
        if let Some(placement) = &self.placement {
            if placement.started.elapsed() < PLACEMENT_ANIM {
                return;
            }
            self.placement = None;
            let Some((at_end, interval)) = self.replay().map(|state| (state.position == state.record.moves.len(), state.interval)) else {
                return;
            };
            if at_end {
                self.finish_replay();
            } else if let Some(replay) = self.replay_mut() {
                replay.next_advance_at = Instant::now() + interval;
            }
            return;
        }

        let due = self
            .replay()
            .is_some_and(|state| !state.paused && Instant::now() >= state.next_advance_at);
        if !due {
            return;
        }
        let at_end = self.replay().is_some_and(|state| state.position == state.record.moves.len());
        if at_end {
            self.finish_replay();
        } else {
            self.replay_step_forward();
        }
    }

    fn finish_replay(&mut self) {
        self.screen = Screen::GameOver;
        self.game_over_at = Some(Instant::now());
        self.cpu = None;
        self.placement = None;
        self.message = match self.game.outcome() {
            Some(Outcome::Win { side, .. }) => format!("{} の勝ち", self.side_name(side)),
            Some(Outcome::Draw) => "引き分け".into(),
            None => "棋譜の終わり".into(),
        };
    }

    fn save_record(&mut self) {
        if !matches!(self.mode, Mode::Normal) {
            return;
        }
        let record = Record {
            first: self.first,
            names: ["あなた".into(), "CPU".into()],
            moves: self.game.history.iter().map(|&(_, cell)| cell).collect(),
        };
        let seconds = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |time| time.as_secs());
        let path = self.record_dir.join(format!("ttt-{seconds}.tttrec"));
        self.message = match std::fs::write(&path, to_text(&record)) {
            Ok(()) => format!("棋譜を保存しました: {}", display_path(&path)),
            Err(error) => format!("棋譜の保存に失敗しました: {error}"),
        };
    }

    fn spawn_confetti(&mut self, min_y: f32) {
        const GLYPHS: [char; 6] = ['*', '+', 'o', 'x', '%', '#'];
        let color = CONFETTI[self.rng.range(0, CONFETTI.len() - 1)];
        let glyph = GLYPHS[self.rng.range(0, GLYPHS.len() - 1)];
        let x = rng_unit(&mut self.rng);
        let y = min_y * rng_unit(&mut self.rng);
        let vx = (rng_unit(&mut self.rng) - 0.5) * 0.008;
        let vy = 0.004 + rng_unit(&mut self.rng) * 0.01;
        self.particles.push(Particle { x, y, vx, vy, glyph, color });
    }

    fn spawn_rain(&mut self) {
        let x = rng_unit(&mut self.rng);
        let vy = 0.01 + rng_unit(&mut self.rng) * 0.01;
        let glyph = if self.rng.chance(50) { '.' } else { ':' };
        self.particles.push(Particle {
            x,
            y: -0.05,
            vx: 0.0,
            vy,
            glyph,
            color: (90, 105, 140),
        });
    }

    fn update_particles(&mut self) {
        if self.screen == Screen::GameOver
            && matches!(self.mode, Mode::Normal)
            && self.game_over_at.is_some_and(|at| at.elapsed() < Duration::from_secs(4))
        {
            match self.game.outcome() {
                Some(Outcome::Win { side: Side::Human, .. }) if self.particles.len() < 260 => {
                    for _ in 0..3 {
                        self.spawn_confetti(-0.05);
                    }
                }
                Some(Outcome::Win { side: Side::Cpu, .. }) if self.particles.len() < 120 => {
                    self.spawn_rain();
                }
                _ => {}
            }
        } else if self.screen == Screen::Title {
            self.particles.clear();
        }
        for particle in &mut self.particles {
            particle.vy += 0.0004;
            particle.x += particle.vx;
            particle.y += particle.vy;
            particle.vx *= 0.99;
        }
        self.particles.retain(|particle| particle.y < 1.05 && (-0.1..1.1).contains(&particle.x));
    }
}

fn display_path(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn rng_unit(rng: &mut Rng) -> f32 {
    (rng.next_u64() >> 40) as f32 / (1u64 << 24) as f32
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{Game, Mark, Outcome, Side};
    use crate::record::{Record, parse};
    use ratatui::crossterm::event::{KeyCode, KeyEvent};
    use std::fs;
    use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::from(code)
    }

    fn complete_record() -> Record {
        Record {
            first: Side::Human,
            names: ["Alice".into(), "Bob".into()],
            moves: vec![0, 3, 1, 4, 2],
        }
    }

    fn unfinished_record() -> Record {
        Record {
            first: Side::Cpu,
            names: ["Alice".into(), "Bob".into()],
            moves: vec![4, 0, 8],
        }
    }

    fn assert_game_matches(actual: &Game, expected: &Game) {
        assert_eq!(actual.board, expected.board);
        assert_eq!(actual.first, expected.first);
        assert_eq!(actual.to_move, expected.to_move);
        assert_eq!(actual.history, expected.history);
    }

    #[test]
    fn number_keys_select_then_confirm_the_same_cell() {
        let mut app = App::new();
        app.start();

        app.on_key(key(KeyCode::Char('5')));
        assert_eq!(app.cursor, 4);
        assert!(app.game.board[4].is_none());

        app.on_key(key(KeyCode::Char('5')));
        assert_eq!(app.game.board[4], Some(Mark::X));
        assert!(app.placement.is_some());
    }

    #[test]
    fn replay_navigation_reconstructs_board_and_history() {
        let record = complete_record();
        let mut app = App::new_replay(record.clone(), Duration::from_millis(10), true, false);

        assert_game_matches(&app.game, &record.game_at(0).unwrap());
        app.on_key(key(KeyCode::Right));
        assert_eq!(app.replay().unwrap().position, 1);
        assert_game_matches(&app.game, &record.game_at(1).unwrap());

        app.on_key(key(KeyCode::Left));
        assert_eq!(app.replay().unwrap().position, 0);
        assert_game_matches(&app.game, &record.game_at(0).unwrap());

        app.on_key(key(KeyCode::End));
        assert_eq!(app.replay().unwrap().position, record.moves.len());
        assert_game_matches(&app.game, &record.game_at(record.moves.len()).unwrap());

        app.on_key(key(KeyCode::Home));
        assert_eq!(app.replay().unwrap().position, 0);
        assert_game_matches(&app.game, &record.game_at(0).unwrap());
    }

    #[test]
    fn autoplay_waits_for_interval_and_placement_without_cpu_thinking() {
        let mut app = App::new_replay(complete_record(), PLACEMENT_ANIM + Duration::from_millis(200), false, false);
        app.replay_mut().unwrap().next_advance_at = Instant::now() - Duration::from_millis(1);

        app.on_tick();
        assert_eq!(app.replay().unwrap().position, 1);
        assert!(app.placement.is_some());
        assert!(app.cpu.is_none());

        app.placement.as_mut().unwrap().started = Instant::now() - PLACEMENT_ANIM;
        app.on_tick();
        assert!(app.placement.is_none());
        assert_eq!(app.replay().unwrap().position, 1);
        app.on_tick();
        assert_eq!(app.replay().unwrap().position, 1);

        app.replay_mut().unwrap().next_advance_at = Instant::now() - Duration::from_millis(1);
        app.on_tick();
        assert_eq!(app.replay().unwrap().position, 2);
        assert!(app.cpu.is_none());
    }

    #[test]
    fn replay_interval_is_clamped_to_placement_animation() {
        let mut app = App::new_replay(complete_record(), Duration::from_millis(1), true, false);
        assert_eq!(app.replay().unwrap().interval, PLACEMENT_ANIM);
        app.on_key(key(KeyCode::Char('+')));
        assert_eq!(app.replay().unwrap().interval, PLACEMENT_ANIM);
        app.on_key(key(KeyCode::Char('-')));
        assert!(app.replay().unwrap().interval > PLACEMENT_ANIM);
    }

    #[test]
    fn replay_end_distinguishes_winner_and_unfinished_record() {
        let mut complete = App::new_replay(complete_record(), Duration::from_millis(1_500), true, false);
        complete.on_key(key(KeyCode::End));
        assert_eq!(complete.screen, Screen::GameOver);
        assert_eq!(complete.message, "Alice の勝ち");

        let mut unfinished = App::new_replay(unfinished_record(), Duration::from_millis(1_500), true, false);
        unfinished.on_key(key(KeyCode::End));
        assert_eq!(unfinished.screen, Screen::GameOver);
        assert_eq!(unfinished.message, "棋譜の終わり");
        assert_eq!(unfinished.game.outcome(), None);
    }

    #[test]
    fn exit_on_end_only_quits_after_a_finished_record() {
        let mut complete = App::new_replay(complete_record(), Duration::from_millis(1_500), true, true);
        complete.on_key(key(KeyCode::End));
        complete.game_over_at = Some(Instant::now() - Duration::from_secs(3));
        complete.on_tick();
        assert!(complete.should_quit);

        let mut unfinished = App::new_replay(unfinished_record(), Duration::from_millis(1_500), true, true);
        unfinished.on_key(key(KeyCode::End));
        unfinished.game_over_at = Some(Instant::now() - Duration::from_secs(3));
        unfinished.on_tick();
        assert!(!unfinished.should_quit);
    }

    #[test]
    fn saving_with_s_writes_a_parseable_record() {
        let nonce = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let dir = std::env::temp_dir().join(format!("rs-ttt-app-test-{}-{nonce}", std::process::id()));
        fs::create_dir(&dir).unwrap();

        let mut app = App::new();
        app.set_record_dir(&dir);
        app.start();
        app.on_key(key(KeyCode::Char('1')));
        app.on_key(key(KeyCode::Char('1')));
        app.on_key(key(KeyCode::Char('s')));

        let path = fs::read_dir(&dir).unwrap().next().unwrap().unwrap().path();
        let saved = parse(&fs::read_to_string(path).unwrap()).unwrap();
        assert_eq!(saved.first, Side::Human);
        assert_eq!(saved.moves, vec![0]);
        assert_eq!(saved.names, ["あなた", "CPU"]);

        let not_a_directory = dir.join("file");
        fs::write(&not_a_directory, "x").unwrap();
        app.set_record_dir(not_a_directory);
        app.on_key(key(KeyCode::Char('s')));
        assert!(app.message.contains("保存に失敗"), "{}", app.message);

        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn normal_finish_updates_session_stats_once() {
        let mut app = App::new();
        app.start();
        app.game = complete_record().game_at(5).unwrap();
        app.finish_normal();
        app.finish_normal();
        assert_eq!(app.stats.wins, 1);
        assert_eq!(app.stats.losses, 0);
        assert_eq!(app.stats.draws, 0);
        assert!(matches!(app.game.outcome(), Some(Outcome::Win { side: Side::Human, .. })));
    }
}
