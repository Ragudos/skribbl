pub const BINARY_PROTOCOL_VERSION: u8 = 1;
/// The amount of time a user has to draw a word in seconds.
pub const DRAW_IME_LIMIT: u8 = 5;
/// The amount of time a user has to pick a word in seconds.
pub const PICK_WORD_TIME_LIMIT: u8 = 5;

lazy_static::lazy_static! {
    pub static ref WORDS: Vec<&'static str> = get_words();
}

fn get_words() -> Vec<&'static str> {
    include_str!("../words.txt")
        .lines()
        .map(|s| s.trim())
        .collect()
}
