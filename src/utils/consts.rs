pub const BINARY_PROTOCOL_VERSION: u8 = 1;

lazy_static::lazy_static! {
    pub static ref WORDS: Vec<&'static str> = get_words();
}

fn get_words() -> Vec<&'static str> {
    include_str!("../words.txt")
        .lines()
        .map(|s| s.trim())
        .collect()
}
