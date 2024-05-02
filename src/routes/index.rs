use rocket::get;
use rocket_dyn_templates::{context, Template};

#[get("/?<roomId>")]
pub async fn index_page(#[allow(non_snake_case)] roomId: Option<String>) -> Template {
    #[allow(non_snake_case)]
    Template::render("index", context! { roomId })
}
