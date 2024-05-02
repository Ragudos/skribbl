use rocket::fairing::Fairing;
use rocket_dyn_templates::Template;

pub fn stage_templates() -> impl Fairing {
    Template::custom(|_engines| {})
}
