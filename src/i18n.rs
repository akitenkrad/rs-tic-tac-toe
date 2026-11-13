//! 画面とテキスト出力に用いる日本語・英語の文言．

use std::fmt::Display;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Lang {
    #[default]
    En,
    Ja,
}

impl Lang {
    pub fn toggle(self) -> Self {
        match self {
            Self::En => Self::Ja,
            Self::Ja => Self::En,
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(value: &str) -> Option<Self> {
        if value.eq_ignore_ascii_case("en") {
            Some(Self::En)
        } else if value.eq_ignore_ascii_case("ja") {
            Some(Self::Ja)
        } else {
            None
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::En => "English",
            Self::Ja => "日本語",
        }
    }

    pub fn default_human_name(self) -> &'static str {
        match self {
            Self::En => "You",
            Self::Ja => "あなた",
        }
    }

    pub fn default_cpu_name(self) -> &'static str {
        "CPU"
    }

    pub fn placed(self, name: &str, cell: &str) -> String {
        match self {
            Self::En => format!("{name} placed at {cell}"),
            Self::Ja => format!("{name} が {cell} に置きました"),
        }
    }

    pub fn showing_move(self, position: usize) -> String {
        match self {
            Self::En => format!("Showing move {position}"),
            Self::Ja => format!("{position} 手目を表示しています"),
        }
    }

    pub fn winner(self, name: &str) -> String {
        match self {
            Self::En => format!("{name} wins"),
            Self::Ja => format!("{name} の勝ち"),
        }
    }

    pub fn record_saved(self, path: &str) -> String {
        match self {
            Self::En => format!("Record saved: {path}"),
            Self::Ja => format!("棋譜を保存しました: {path}"),
        }
    }

    pub fn record_save_failed(self, error: impl Display) -> String {
        match self {
            Self::En => format!("Failed to save record: {error}"),
            Self::Ja => format!("棋譜の保存に失敗しました: {error}"),
        }
    }

    pub fn resize(self, min_width: u16, min_height: u16, width: u16, height: u16) -> String {
        match self {
            Self::En => format!("Resize the terminal to at least {min_width}x{min_height} (currently {width}x{height})"),
            Self::Ja => format!("端末を {min_width}x{min_height} 以上に広げてください (現在 {width}x{height})"),
        }
    }

    pub fn replay_progress(self, position: usize, total: usize) -> String {
        match self {
            Self::En => format!("Move {position}/{total}"),
            Self::Ja => format!("手数 {position}/{total}"),
        }
    }

    pub fn turn(self, name: &str, mark: &str) -> String {
        match self {
            Self::En if name == self.default_human_name() => format!("Your turn {mark}"),
            Self::En => format!("{name}'s turn {mark}"),
            Self::Ja => format!("{name}の番 {mark}"),
        }
    }

    pub fn stats(self, wins: u32, losses: u32, draws: u32) -> String {
        match self {
            Self::En => format!("  Wins {wins}   Losses {losses}   Draws {draws}"),
            Self::Ja => format!("  勝 {wins}   敗 {losses}   分 {draws}"),
        }
    }

    pub fn evaluation(self, result: Msg, turns: u8) -> String {
        match self {
            Self::En => format!("{} {turns}", result.text(self)),
            Self::Ja => format!("{} {turns}", result.text(self)),
        }
    }

    pub fn moves_count(self, count: usize) -> String {
        match self {
            Self::En => format!("Moves {count}"),
            Self::Ja => format!("手数 {count}"),
        }
    }

    pub fn move_line(self, index: usize, name: &str, mark: &str, cell: &str) -> String {
        match self {
            Self::En => format!("Move {index}: {name} {mark} {cell}"),
            Self::Ja => format!("手数 {index}: {name} {mark} {cell}"),
        }
    }

    pub fn winner_line(self, name: &str) -> String {
        match self {
            Self::En => format!("Winner: {name}"),
            Self::Ja => format!("勝者: {name}"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Msg {
    TitleSubtitle,
    StartGame,
    Difficulty,
    First,
    Language,
    Quit,
    Settings,
    StartPrompt,
    Easy,
    Normal,
    Perfect,
    HumanTurn,
    CpuThinking,
    CellUnavailable,
    HumanWins,
    CpuWins,
    Draw,
    RecordStart,
    EndOfRecord,
    Paused,
    Playing,
    Speed,
    Turn,
    BoardEvaluation,
    Board,
    EvalWin,
    EvalDraw,
    EvalLoss,
    Hint,
    HighlightWinning,
    OnlyDraw,
    CannotAvoidLoss,
    Status,
    SessionScore,
    MoveLog,
    PlayPause,
    PreviousNext,
    BeginningEnd,
    Evaluation,
    NewGame,
    Save,
    Title,
    Select,
    ArrowKeys,
    Place,
    New,
    Back,
    End,
    ResultDraw,
    ResultUnfinished,
}

impl Msg {
    pub const ALL: [Self; 50] = [
        Self::TitleSubtitle,
        Self::StartGame,
        Self::Difficulty,
        Self::First,
        Self::Language,
        Self::Quit,
        Self::Settings,
        Self::StartPrompt,
        Self::Easy,
        Self::Normal,
        Self::Perfect,
        Self::HumanTurn,
        Self::CpuThinking,
        Self::CellUnavailable,
        Self::HumanWins,
        Self::CpuWins,
        Self::Draw,
        Self::RecordStart,
        Self::EndOfRecord,
        Self::Paused,
        Self::Playing,
        Self::Speed,
        Self::Turn,
        Self::BoardEvaluation,
        Self::Board,
        Self::EvalWin,
        Self::EvalDraw,
        Self::EvalLoss,
        Self::Hint,
        Self::HighlightWinning,
        Self::OnlyDraw,
        Self::CannotAvoidLoss,
        Self::Status,
        Self::SessionScore,
        Self::MoveLog,
        Self::PlayPause,
        Self::PreviousNext,
        Self::BeginningEnd,
        Self::Evaluation,
        Self::NewGame,
        Self::Save,
        Self::Title,
        Self::Select,
        Self::ArrowKeys,
        Self::Place,
        Self::New,
        Self::Back,
        Self::End,
        Self::ResultDraw,
        Self::ResultUnfinished,
    ];

    pub fn text(self, lang: Lang) -> &'static str {
        match (self, lang) {
            (Self::TitleSubtitle, Lang::En) => "Tic-tac-toe — perfect play and record replay",
            (Self::TitleSubtitle, Lang::Ja) => "三目並べ — 完全読みと棋譜再生",
            (Self::StartGame, Lang::En) => "Start game",
            (Self::StartGame, Lang::Ja) => "ゲーム開始",
            (Self::Difficulty, Lang::En) => "Difficulty",
            (Self::Difficulty, Lang::Ja) => "難易度",
            (Self::First, Lang::En) => "First player",
            (Self::First, Lang::Ja) => "先手",
            (Self::Language, Lang::En) => "Language",
            (Self::Language, Lang::Ja) => "言語",
            (Self::Quit, Lang::En) => "Quit",
            (Self::Quit, Lang::Ja) => "終了",
            (Self::Settings, Lang::En) => "Settings",
            (Self::Settings, Lang::Ja) => "設定",
            (Self::StartPrompt, Lang::En) => "Press Enter / Space to start",
            (Self::StartPrompt, Lang::Ja) => "Enter / Space で開始",
            (Self::Easy, Lang::En) => "Easy",
            (Self::Easy, Lang::Ja) => "やさしい",
            (Self::Normal, Lang::En) => "Normal",
            (Self::Normal, Lang::Ja) => "ふつう",
            (Self::Perfect, Lang::En) => "Perfect (full search)",
            (Self::Perfect, Lang::Ja) => "最強 (完全読み)",
            (Self::HumanTurn, Lang::En) => "Your turn — pick a cell, then Enter",
            (Self::HumanTurn, Lang::Ja) => "あなたの番です．マスを選んで Enter",
            (Self::CpuThinking, Lang::En) => "CPU is thinking",
            (Self::CpuThinking, Lang::Ja) => "CPU が考えています",
            (Self::CellUnavailable, Lang::En) => "You cannot place there",
            (Self::CellUnavailable, Lang::Ja) => "そのマスには置けません",
            (Self::HumanWins, Lang::En) => "You win!",
            (Self::HumanWins, Lang::Ja) => "あなたの勝ち！",
            (Self::CpuWins, Lang::En) => "CPU wins",
            (Self::CpuWins, Lang::Ja) => "CPU の勝ち",
            (Self::Draw, Lang::En) => "Draw",
            (Self::Draw, Lang::Ja) => "引き分け",
            (Self::RecordStart, Lang::En) => "Start of record",
            (Self::RecordStart, Lang::Ja) => "棋譜の先頭です",
            (Self::EndOfRecord, Lang::En) => "End of record",
            (Self::EndOfRecord, Lang::Ja) => "棋譜の終わり",
            (Self::Paused, Lang::En) => "Paused",
            (Self::Paused, Lang::Ja) => "一時停止",
            (Self::Playing, Lang::En) => "Playing",
            (Self::Playing, Lang::Ja) => "再生中",
            (Self::Speed, Lang::En) => "Speed",
            (Self::Speed, Lang::Ja) => "速度",
            (Self::Turn, Lang::En) => "Turn",
            (Self::Turn, Lang::Ja) => "手番",
            (Self::BoardEvaluation, Lang::En) => "Position — Win / Draw / Loss",
            (Self::BoardEvaluation, Lang::Ja) => "局面評価 — 勝ち / 引分 / 負け",
            (Self::Board, Lang::En) => "Board",
            (Self::Board, Lang::Ja) => "盤面",
            (Self::EvalWin, Lang::En) => "Win",
            (Self::EvalWin, Lang::Ja) => "勝ち",
            (Self::EvalDraw, Lang::En) => "Draw",
            (Self::EvalDraw, Lang::Ja) => "引分",
            (Self::EvalLoss, Lang::En) => "Loss",
            (Self::EvalLoss, Lang::Ja) => "負け",
            (Self::Hint, Lang::En) => "Hint",
            (Self::Hint, Lang::Ja) => "ヒント",
            (Self::HighlightWinning, Lang::En) => "Highlighting winning moves",
            (Self::HighlightWinning, Lang::Ja) => "勝てる手を強調中",
            (Self::OnlyDraw, Lang::En) => "Only a draw is possible",
            (Self::OnlyDraw, Lang::Ja) => "引き分けしかありません",
            (Self::CannotAvoidLoss, Lang::En) => "A loss cannot be avoided",
            (Self::CannotAvoidLoss, Lang::Ja) => "負けを避けられません",
            (Self::Status, Lang::En) => "Status",
            (Self::Status, Lang::Ja) => "状態",
            (Self::SessionScore, Lang::En) => "Session score",
            (Self::SessionScore, Lang::Ja) => "セッション戦績",
            (Self::MoveLog, Lang::En) => "Move log",
            (Self::MoveLog, Lang::Ja) => "着手ログ",
            (Self::PlayPause, Lang::En) => "Play/Pause",
            (Self::PlayPause, Lang::Ja) => "再生/停止",
            (Self::PreviousNext, Lang::En) => "Previous/Next",
            (Self::PreviousNext, Lang::Ja) => "前後",
            (Self::BeginningEnd, Lang::En) => "Beginning/End",
            (Self::BeginningEnd, Lang::Ja) => "先頭/末尾",
            (Self::Evaluation, Lang::En) => "Evaluation",
            (Self::Evaluation, Lang::Ja) => "評価",
            (Self::NewGame, Lang::En) => "New game",
            (Self::NewGame, Lang::Ja) => "新しい対局",
            (Self::Save, Lang::En) => "Save",
            (Self::Save, Lang::Ja) => "保存",
            (Self::Title, Lang::En) => "Title",
            (Self::Title, Lang::Ja) => "タイトル",
            (Self::Select, Lang::En) => "Select",
            (Self::Select, Lang::Ja) => "選択",
            (Self::ArrowKeys, Lang::En) => "Arrows/1-9",
            (Self::ArrowKeys, Lang::Ja) => "矢印/1-9",
            (Self::Place, Lang::En) => "Place",
            (Self::Place, Lang::Ja) => "置く",
            (Self::New, Lang::En) => "New",
            (Self::New, Lang::Ja) => "新規",
            (Self::Back, Lang::En) => "Back",
            (Self::Back, Lang::Ja) => "戻る",
            (Self::End, Lang::En) => "End",
            (Self::End, Lang::Ja) => "終了",
            (Self::ResultDraw, Lang::En) => "Result: draw",
            (Self::ResultDraw, Lang::Ja) => "結果: 引き分け",
            (Self::ResultUnfinished, Lang::En) => "Result: unfinished",
            (Self::ResultUnfinished, Lang::Ja) => "結果: 未終局",
        }
    }
}
