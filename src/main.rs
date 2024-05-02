use rocket::Rocket;
use skribbl::init_rocket;

#[macro_use]
extern crate rocket;

#[launch]
async fn rocket() -> _ {
    init_rocket(Rocket::build()).await
}
