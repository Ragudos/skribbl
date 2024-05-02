pub mod consts;

pub fn gen_random_id() -> String {
    random_string::generate(6, random_string::charsets::ALPHANUMERIC)
}
