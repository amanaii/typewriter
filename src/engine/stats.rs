use std::time::Duration;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Stats {
    pub wpm: f64,
    pub raw_wpm: f64,
    pub accuracy: f64,
    pub consistency: f64,
    pub correct_chars: usize,
    pub incorrect_chars: usize,
    pub extra_chars: usize,
    pub missed_chars: usize,
    pub total_typed: usize,
    pub elapsed_secs: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TypedChar {
    pub ch: char,
}

pub fn calculate(
    targets: &[String],
    typed_words: &[Vec<TypedChar>],
    elapsed: Duration,
    wpm_samples: &[f64],
    include_word_index: usize,
) -> Stats {
    let elapsed_secs = elapsed.as_secs_f64().max(0.001);
    let mut correct_chars = 0;
    let mut incorrect_chars = 0;
    let mut extra_chars = 0;
    let mut missed_chars = 0;
    let mut total_typed = 0;

    let last = include_word_index.min(targets.len().saturating_sub(1));
    for word_idx in 0..=last {
        let Some(target) = targets.get(word_idx) else {
            continue;
        };
        let target_chars: Vec<char> = target.chars().collect();
        let typed = typed_words.get(word_idx).map(Vec::as_slice).unwrap_or(&[]);
        total_typed += typed.len();

        for (char_idx, typed_char) in typed.iter().enumerate() {
            match target_chars.get(char_idx) {
                Some(expected) if *expected == typed_char.ch => correct_chars += 1,
                Some(_) => incorrect_chars += 1,
                None => extra_chars += 1,
            }
        }

        if typed.len() < target_chars.len() {
            missed_chars += target_chars.len() - typed.len();
        }
    }

    let minutes = elapsed_secs / 60.0;
    let wpm = (correct_chars as f64 / 5.0) / minutes;
    let raw_wpm = (total_typed as f64 / 5.0) / minutes;
    let accuracy = if total_typed == 0 {
        100.0
    } else {
        correct_chars as f64 / total_typed as f64 * 100.0
    };

    Stats {
        wpm,
        raw_wpm,
        accuracy,
        consistency: consistency(wpm_samples),
        correct_chars,
        incorrect_chars,
        extra_chars,
        missed_chars,
        total_typed,
        elapsed_secs,
    }
}

pub fn consistency(samples: &[f64]) -> f64 {
    if samples.len() < 2 {
        return 100.0;
    }
    let mean = samples.iter().sum::<f64>() / samples.len() as f64;
    if mean <= f64::EPSILON {
        return 100.0;
    }
    let variance = samples
        .iter()
        .map(|sample| {
            let diff = *sample - mean;
            diff * diff
        })
        .sum::<f64>()
        / samples.len() as f64;
    (100.0 - (variance.sqrt() / mean * 100.0)).clamp(0.0, 100.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculates_wpm_raw_accuracy_breakdown() {
        let targets = vec!["abcd".to_string(), "xy".to_string()];
        let typed = vec![
            vec![
                TypedChar { ch: 'a' },
                TypedChar { ch: 'b' },
                TypedChar { ch: 'z' },
                TypedChar { ch: 'd' },
                TypedChar { ch: '!' },
            ],
            vec![TypedChar { ch: 'x' }],
        ];

        let stats = calculate(&targets, &typed, Duration::from_secs(60), &[10.0, 20.0], 1);

        assert_eq!(stats.correct_chars, 4);
        assert_eq!(stats.incorrect_chars, 1);
        assert_eq!(stats.extra_chars, 1);
        assert_eq!(stats.missed_chars, 1);
        assert_eq!(stats.total_typed, 6);
        assert!((stats.wpm - 0.8).abs() < f64::EPSILON);
        assert!((stats.raw_wpm - 1.2).abs() < f64::EPSILON);
        assert!((stats.accuracy - 66.66666666666666).abs() < 0.0001);
    }

    #[test]
    fn consistency_uses_coefficient_of_variation() {
        let value = consistency(&[60.0, 60.0, 60.0]);
        assert!((value - 100.0).abs() < f64::EPSILON);

        let variable = consistency(&[30.0, 60.0, 90.0]);
        assert!(variable < 100.0);
    }
}
