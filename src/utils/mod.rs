use rand::Rng;

pub mod consts;
pub mod realtime;

pub fn gen_random_id() -> String {
    random_string::generate(6, random_string::charsets::ALPHANUMERIC)
}

pub fn get_random_word() -> &'static str {
    consts::WORDS[rand::thread_rng().gen_range(0..consts::WORDS.len())]
}

/// Turn a usize into a vector of u8
/// where each element is less than or equal to 255.
///
/// For example:
///
/// ```rust
/// let value = 300;
/// let bytes = turn_usize_to_vec_of_u8(value);
/// assert_eq!(bytes, vec![255, 45]);
/// ```
pub fn turn_usize_to_vec_of_u8(value: usize) -> Vec<u8> {
    let mut bytes = vec![];
    let mut value = value;

    while value > 0 {
        if value > 255 {
            value -= 255;
            bytes.push(255);
        } else {
            bytes.push(value as u8);
            break;
        }
    }

    bytes
}

pub fn obfuscate_word(word: String) -> String {
    word.chars()
        .map(|c| if c.is_alphabetic() { '*' } else { c })
        .collect()
}
