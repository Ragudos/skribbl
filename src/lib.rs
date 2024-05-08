use rocket::tokio;

pub mod events;
pub mod fairings;
pub mod routes;
pub mod state;
pub mod utils;

pub async fn init_rocket(
    rocket: rocket::Rocket<rocket::Build>,
) -> rocket::Rocket<rocket::Build> {
    let game_state = state::GameState::default();

    rocket
        .mount("/", rocket::routes![routes::index::index_page,])
        .mount(
            "/ws",
            rocket::routes![
                routes::realtime::socket::ws_endpoint,
                routes::realtime::binary_protocol_version_endpoint,
            ],
        )
        .mount("/dist", rocket::fs::FileServer::from("dist"))
        .attach(fairings::stage_templates())
        .manage(tokio::sync::broadcast::channel::<events::WebSocketMessage>(1024).0)
        .manage(game_state)
}
