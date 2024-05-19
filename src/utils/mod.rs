use rand::Rng;

use crate::state;

pub mod consts;

#[macro_export]
macro_rules! vec_with_slices {
    ($($x:expr),*; $($slice:expr),*) => {{
        let mut v = vec![$($x),*];
        $(
            v.extend_from_slice($slice);
        )*

        v
    }};
}

pub fn gen_random_id() -> String {
    random_string::generate(6, random_string::charsets::ALPHANUMERIC)
}

pub fn get_random_word() -> &'static str {
    consts::WORDS[rand::thread_rng().gen_range(0..consts::WORDS.len())]
}

pub fn choose_user_in_a_room_randomly<'st>(
    users: &'st [state::User],
    room_id: &str,
) -> Result<&'st state::User, Box<dyn std::error::Error>> {
    let users = users
        .iter()
        .filter(|u| u.room_id == room_id)
        .collect::<Vec<_>>();

    Ok(*users
        .get(rand::thread_rng().gen_range(0..users.len()))
        .ok_or("No user found in the room")?)
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

pub fn obfuscate_word(word: &str) -> String {
    word.chars()
        .map(|c| if c.is_alphabetic() { '*' } else { c })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_turn_usize_to_vec_of_u8() {
        let value = 300;
        let bytes = turn_usize_to_vec_of_u8(value);
        assert_eq!(bytes, vec![255, 45]);
    }

    #[test]
    fn test_obfuscate_word() {
        let word = "hello world";
        let obfuscated = obfuscate_word(word);
        assert_eq!(obfuscated, "***** *****");
    }
}
