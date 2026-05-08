use std::{fs, time::Duration};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use serde::{Deserialize, Serialize};

use crate::{
    config::{config_dir, Config},
    engine::{
        stats::{self, Stats, TypedChar},
        timer::TestTimer,
        words,
    },
    events::AppEvent,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mode {
    Words(usize),
    Time(u64),
    Quote,
}

impl Mode {
    pub fn label(self) -> String {
        match self {
            Self::Words(n) => format!("{n} words"),
            Self::Time(s) => format!("{s}s"),
            Self::Quote => "quote".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum AppState {
    Home(HomeState),
    Test(TestState),
    Results(ResultsState),
    Quitting,
}

#[derive(Debug, Clone)]
pub struct HomeState {
    pub config: Config,
    pub selected_mode: Mode,
    pub best_wpm: f64,
}

#[derive(Debug, Clone)]
pub struct TestState {
    pub config: Config,
    pub mode: Mode,
    pub words: Vec<String>,
    pub typed: Vec<Vec<TypedChar>>,
    pub current_word: usize,
    pub cursor_in_word: usize,
    pub timer: TestTimer,
    pub stats: Stats,
    pub wpm_samples: Vec<f64>,
    pub chars_at_last_sample: usize,
    pub next_sample_at: Duration,
    pub tick: u64,
    pub caret_visible: bool,
    pub last_caret_flip: Duration,
    pub terminal_size: (u16, u16),
}

#[derive(Debug, Clone)]
pub struct ResultsState {
    pub config: Config,
    pub mode: Mode,
    pub stats: Stats,
    pub wpm_samples: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Score {
    pub mode: Mode,
    pub stats: Stats,
}

pub struct App {
    pub state: AppState,
    config: Config,
    mode: Mode,
    last_words: Vec<String>,
    scores: Vec<Score>,
}

impl App {
    pub fn new(config: Config, mode: Mode) -> Self {
        let scores = load_scores();
        let best_wpm = scores
            .iter()
            .map(|score| score.stats.wpm)
            .fold(0.0, f64::max);
        Self {
            state: AppState::Home(HomeState {
                config: config.clone(),
                selected_mode: mode,
                best_wpm,
            }),
            config,
            mode,
            last_words: Vec::new(),
            scores,
        }
    }

    pub fn should_quit(&self) -> bool {
        matches!(self.state, AppState::Quitting)
    }

    pub fn quit(&mut self) {
        self.state = AppState::Quitting;
    }

    pub fn handle_event(&mut self, event: AppEvent) {
        if let AppEvent::Key(key) = event {
            if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
                self.quit();
                return;
            }
            if matches!(self.state, AppState::Test(_))
                && key.modifiers.contains(KeyModifiers::CONTROL)
                && key.code == KeyCode::Char('r')
            {
                self.start_test(false);
                return;
            }
        }

        match event {
            AppEvent::Tick => self.on_tick(),
            AppEvent::Resize(width, height) => self.on_resize(width, height),
            AppEvent::Key(key) => self.on_key(key),
        }
    }

    pub fn save_outputs(&self) {
        let _ = self.config.save();
        let _ = save_scores(&self.scores);
    }

    fn on_key(&mut self, key: KeyEvent) {
        match &self.state {
            AppState::Home(_) => self.on_home_key(key),
            AppState::Test(_) => self.on_test_key(key),
            AppState::Results(_) => self.on_results_key(key),
            AppState::Quitting => {}
        }
    }

    fn on_home_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => self.quit(),
            KeyCode::Char('1') => self.set_home_mode(Mode::Words(25)),
            KeyCode::Char('2') => self.set_home_mode(Mode::Words(50)),
            KeyCode::Char('3') => self.set_home_mode(Mode::Words(100)),
            KeyCode::Char('4') => self.set_home_mode(Mode::Time(15)),
            KeyCode::Char('5') => self.set_home_mode(Mode::Time(30)),
            KeyCode::Char('6') => self.set_home_mode(Mode::Time(60)),
            KeyCode::Char('7') => self.set_home_mode(Mode::Quote),
            KeyCode::Enter | KeyCode::Char(' ') => self.start_test(false),
            _ => {}
        }
    }

    fn on_test_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => self.show_home(),
            KeyCode::Backspace => self.backspace(),
            KeyCode::Char(' ') => self.advance_word(),
            KeyCode::Char(ch) => self.type_char(ch),
            _ => {}
        }
    }

    fn on_results_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => self.show_home(),
            KeyCode::Tab => self.start_test(false),
            KeyCode::Enter => self.start_test(false),
            _ => {}
        }
    }

    fn on_tick(&mut self) {
        let AppState::Test(test) = &mut self.state else {
            return;
        };
        test.tick = test.tick.wrapping_add(1);
        if !test.timer.is_started() {
            return;
        }
        let elapsed = test.timer.elapsed();
        if test.config.behavior.smooth_caret
            && elapsed.saturating_sub(test.last_caret_flip) >= Duration::from_millis(530)
        {
            test.caret_visible = !test.caret_visible;
            test.last_caret_flip = elapsed;
        }
        update_test_stats(test);
        if elapsed >= test.next_sample_at {
            let delta = test
                .stats
                .total_typed
                .saturating_sub(test.chars_at_last_sample);
            let sample_wpm = (delta as f64 / 5.0) / (2.0 / 60.0);
            test.wpm_samples.push(sample_wpm);
            test.chars_at_last_sample = test.stats.total_typed;
            test.next_sample_at += Duration::from_secs(2);
            update_test_stats(test);
        }
        if test.timer.expired() {
            self.finish_test();
        }
    }

    fn on_resize(&mut self, width: u16, height: u16) {
        if let AppState::Test(test) = &mut self.state {
            test.terminal_size = (width, height);
        }
    }

    fn set_home_mode(&mut self, mode: Mode) {
        self.mode = mode;
        let best_wpm = self.best_wpm();
        self.state = AppState::Home(HomeState {
            config: self.config.clone(),
            selected_mode: mode,
            best_wpm,
        });
    }

    fn show_home(&mut self) {
        let best_wpm = self.best_wpm();
        self.state = AppState::Home(HomeState {
            config: self.config.clone(),
            selected_mode: self.mode,
            best_wpm,
        });
    }

    fn start_test(&mut self, reuse_words: bool) {
        let words = if reuse_words && !self.last_words.is_empty() {
            self.last_words.clone()
        } else {
            self.generate_words()
        };
        self.last_words = words.clone();
        let limit = match self.mode {
            Mode::Time(s) => Some(Duration::from_secs(s.max(1))),
            Mode::Words(_) | Mode::Quote => None,
        };
        let typed = vec![Vec::new(); words.len()];
        self.state = AppState::Test(TestState {
            config: self.config.clone(),
            mode: self.mode,
            words,
            typed,
            current_word: 0,
            cursor_in_word: 0,
            timer: TestTimer::new(limit),
            stats: Stats::default(),
            wpm_samples: Vec::new(),
            chars_at_last_sample: 0,
            next_sample_at: Duration::from_secs(2),
            tick: 0,
            caret_visible: true,
            last_caret_flip: Duration::ZERO,
            terminal_size: (0, 0),
        });
    }

    fn generate_words(&self) -> Vec<String> {
        match self.mode {
            Mode::Words(n) => words::random_words("english_200", n.max(1)),
            Mode::Time(_) => words::random_stream("english_200", 240),
            Mode::Quote => words::random_quote(),
        }
    }

    fn type_char(&mut self, ch: char) {
        let AppState::Test(test) = &mut self.state else {
            return;
        };
        if test.config.behavior.stop_on_error && has_current_error(test) {
            return;
        }
        test.timer.start();
        test.typed[test.current_word].push(TypedChar { ch });
        test.cursor_in_word += 1;
        update_test_stats(test);
        let should_finish = matches!(test.mode, Mode::Words(_) | Mode::Quote)
            && test.current_word + 1 >= test.words.len()
            && test
                .words
                .get(test.current_word)
                .is_some_and(|word| test.cursor_in_word >= word.chars().count());
        if should_finish {
            self.finish_test();
        }
    }

    fn backspace(&mut self) {
        let AppState::Test(test) = &mut self.state else {
            return;
        };
        if test.cursor_in_word == 0 {
            return;
        }
        if test.typed[test.current_word].pop().is_some() {
            test.cursor_in_word -= 1;
        }
        update_test_stats(test);
    }

    fn advance_word(&mut self) {
        let AppState::Test(test) = &mut self.state else {
            return;
        };
        if test.config.behavior.stop_on_error && has_current_error(test) {
            return;
        }
        test.timer.start();
        if test.current_word + 1 >= test.words.len() {
            self.finish_test();
            return;
        }
        test.current_word += 1;
        test.cursor_in_word = test.typed[test.current_word].len();
        update_test_stats(test);
    }

    fn finish_test(&mut self) {
        let AppState::Test(mut test) = std::mem::replace(&mut self.state, AppState::Quitting)
        else {
            return;
        };
        update_test_stats(&mut test);
        if test.stats.total_typed > 0 {
            self.scores.push(Score {
                mode: test.mode,
                stats: test.stats,
            });
            if self.scores.len() > 50 {
                let keep_from = self.scores.len() - 50;
                self.scores.drain(0..keep_from);
            }
        }
        self.state = AppState::Results(ResultsState {
            config: self.config.clone(),
            mode: test.mode,
            stats: test.stats,
            wpm_samples: test.wpm_samples,
        });
    }

    pub fn best_wpm(&self) -> f64 {
        self.scores
            .iter()
            .map(|score| score.stats.wpm)
            .fold(0.0, f64::max)
    }
}

impl TestState {
    pub fn time_label(&self) -> String {
        match self.mode {
            Mode::Time(_) => {
                let secs = self.timer.remaining().unwrap_or_default().as_secs();
                format!("{secs}s left")
            }
            Mode::Words(_) | Mode::Quote => format!("{:.0}s", self.timer.elapsed().as_secs_f64()),
        }
    }

    pub fn started(&self) -> bool {
        self.timer.is_started()
    }
}

fn update_test_stats(test: &mut TestState) {
    test.stats = stats::calculate(
        &test.words,
        &test.typed,
        test.timer.elapsed(),
        &test.wpm_samples,
        test.current_word,
    );
}

fn has_current_error(test: &TestState) -> bool {
    let Some(target) = test.words.get(test.current_word) else {
        return false;
    };
    let expected: Vec<char> = target.chars().collect();
    test.typed[test.current_word]
        .iter()
        .enumerate()
        .any(|(idx, typed)| {
            expected
                .get(idx)
                .is_none_or(|expected| *expected != typed.ch)
        })
}

pub fn mode_from_config(config: &Config) -> Mode {
    match config.test.default_mode.as_str() {
        "time" => Mode::Time(config.test.duration.max(1)),
        "quote" => Mode::Quote,
        _ => Mode::Words(config.test.word_count.max(1)),
    }
}

fn scores_path() -> std::path::PathBuf {
    config_dir().join("scores.json")
}

fn load_scores() -> Vec<Score> {
    let path = scores_path();
    let Ok(raw) = fs::read_to_string(path) else {
        return Vec::new();
    };
    serde_json::from_str(&raw).unwrap_or_default()
}

fn save_scores(scores: &[Score]) -> std::io::Result<()> {
    let path = scores_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let raw = serde_json::to_string_pretty(scores).map_err(std::io::Error::other)?;
    fs::write(path, raw)
}
