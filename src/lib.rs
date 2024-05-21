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

    /* {
        let cloned_game_state = game_state.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(10));

            loop {
                interval.tick().await;

                println!("Rooms: {:#?}", cloned_game_state.rooms.lock().await);
                println!("Users: {:#?}", cloned_game_state.users.lock().await);
            }
        });
    } */

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
        .manage(tokio::sync::broadcast::channel::<state::TickerMsg>(1024).0)
        .manage(game_state)
}
