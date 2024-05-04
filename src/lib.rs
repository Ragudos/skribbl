use rocket::tokio;

pub mod fairings;
pub mod model;
pub mod routes;
pub mod utils;

pub async fn init_rocket(
    rocket: rocket::Rocket<rocket::Build>,
) -> rocket::Rocket<rocket::Build> {
    let game_state = model::GameState::new();
    let cloned_game_state = game_state.clone();

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(10));
        loop {
            interval.tick().await;

            println!("Room {:#?}", cloned_game_state.rooms.lock().await);
            println!("User {:#?}", cloned_game_state.users.lock().await);
        }
    });

    rocket
        .mount("/", rocket::routes![routes::index::index_page,])
        .mount(
            "/ws",
            rocket::routes![
                routes::ws::ws_endpoint,
                routes::ws::handshake_endpoint
            ],
        )
        .mount("/dist", rocket::fs::FileServer::from("dist"))
        .attach(fairings::stage_templates())
        .manage(tokio::sync::broadcast::channel::<model::WebSocketMessage>(1024).0)
        .manage(tokio::sync::broadcast::channel::<model::WebSocketTick>(1024).0)
        .manage(game_state)
}
