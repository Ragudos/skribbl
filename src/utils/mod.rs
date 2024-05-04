use rand::Rng;

pub mod consts;
pub mod realtime;

pub fn gen_random_id() -> String {
    random_string::generate(6, random_string::charsets::ALPHANUMERIC)
}

pub fn get_random_word() -> &'static str {
    consts::WORDS[rand::thread_rng().gen_range(0..consts::WORDS.len())]
}
