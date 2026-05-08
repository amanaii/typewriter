use rand::{seq::SliceRandom, thread_rng};

const ENGLISH_200: &str = include_str!("../../assets/words/english_200.txt");
const ENGLISH_1000: &str = include_str!("../../assets/words/english_1000.txt");

const QUOTES: &[&str] = &[
    "The limits of my language mean the limits of my world",
    "Programs must be written for people to read and only incidentally for machines to execute",
    "Simplicity is prerequisite for reliability",
    "Make it work make it right make it fast",
    "The best way to predict the future is to invent it",
    "Premature optimization is the root of all evil",
];

pub fn random_words(wordlist: &str, count: usize) -> Vec<String> {
    let source = match wordlist {
        "english_1000" => ENGLISH_1000,
        _ => ENGLISH_200,
    };
    let mut rng = thread_rng();
    let words: Vec<&str> = source
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect();
    (0..count)
        .filter_map(|_| words.choose(&mut rng).copied())
        .map(str::to_string)
        .collect()
}

pub fn random_stream(wordlist: &str, count: usize) -> Vec<String> {
    random_words(wordlist, count)
}

pub fn random_quote() -> Vec<String> {
    let mut rng = thread_rng();
    QUOTES
        .choose(&mut rng)
        .unwrap_or(&QUOTES[0])
        .split_whitespace()
        .map(str::to_string)
        .collect()
}
