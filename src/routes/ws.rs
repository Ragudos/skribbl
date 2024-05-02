use rocket::futures::{SinkExt, StreamExt};
use rocket::tokio;

use crate::model::{self, WebSocketMessage};
use crate::utils;
use crate::utils::consts::BINARY_PROTOCOL_VERSION;

pub fn find_user_by_id<'a>(
    users: &'a Vec<model::User>,
    id: &str,
) -> Option<&'a model::User> {
    users.iter().find(|u| u.id == id)
}

pub fn find_room_by_id<'a>(
    rooms: &'a Vec<model::Room>,
    id: &str,
) -> Option<&'a model::Room> {
    rooms.iter().find(|r| r.id == id)
}

async fn spawn_timer<'st>(
    room_id: String,
    game_state: &'st rocket::State<model::GameState>,
    ticker: &'st rocket::State<tokio::sync::broadcast::Sender<model::WebSocketTick>>,
) -> () {
    let rooms = game_state.rooms.lock().await;
    //TODO: Make sure we don't need to use [`unwrap()`] here.
    let room_before_timer_began = rooms.iter().find(|r| r.id == room_id).unwrap();
    let Some(user_to_draw_before_timer_began) = (match &room_before_timer_began.state {
        model::RoomState::Playing { user_to_draw, .. } => Some(user_to_draw.clone()),
        _ => None,
    }) else {
        return;
    };

    let room_id = room_id.clone();
    let ticker = ticker.inner().clone();
    let rooms = game_state.rooms.clone();

    rocket::tokio::task::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(1));

        loop {
            interval.tick().await;

            let mut rooms = rooms.lock().await;
            // If we can't find the room, this means the room has been deleted.
            // Thus, we should stop the timer.
            let Some(room) = rooms.iter_mut().find(|r| r.id == room_id) else {
                break;
            };

            match &mut room.state {
                model::RoomState::Playing {
                    time_left,
                    user_to_draw,
                    ..
                } => {
                    // Since this timer has no awareness
                    // of when the user who's drawing suddenly leaves,
                    // we need to check if the user who's drawing has changed.
                    if user_to_draw_before_timer_began != *user_to_draw {
                        break;
                    }

                    if *time_left == 0 {
                        // TODO: Send new round, or new user to draw event here
                        break;
                    }

                    *time_left -= 1;
                }
                // Again, since this timer has no awareness of when the game's state changes in
                // realtime, we need to check if the game's state has changed.
                // This is sort of like dirty checking.
                _ => break,
            }

            let _ = ticker.send(model::WebSocketTick {
                room_id: room_id.clone(),
            });
        }
    });

    ()
}

async fn create_reader<'st>(
    room_id: String,
    messages: &'st rocket::State<tokio::sync::broadcast::Sender<WebSocketMessage>>,
    ticker: &'st rocket::State<tokio::sync::broadcast::Sender<model::WebSocketTick>>,
    game_state: &'st rocket::State<model::GameState>,
    mut stream: rocket::futures::stream::SplitStream<ws::stream::DuplexStream>,
) -> Result<(), Box<dyn std::error::Error>> {
    while let Some(message) = stream.next().await {
        let message = message?;

        spawn_timer(room_id.clone(), game_state, ticker).await;

        let _ = messages.send(WebSocketMessage {
            room_id: room_id.clone(),
            message,
        });
    }

    Ok(())
}

async fn create_room_writer<'st>(
    room_id: String,
    messages: &'st rocket::State<tokio::sync::broadcast::Sender<WebSocketMessage>>,
    sink: std::sync::Arc<
        rocket::futures::lock::Mutex<
            rocket::futures::stream::SplitSink<ws::stream::DuplexStream, ws::Message>,
        >,
    >,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut receiver = messages.subscribe();

    while let Some(ws_message) = receiver.recv().await.ok() {
        if ws_message.room_id != room_id {
            continue;
        }

        sink.lock().await.send(ws_message.message).await?;
    }

    Ok(())
}

async fn create_timer<'st>(
    room_id: String,
    ticker: &'st rocket::State<tokio::sync::broadcast::Sender<model::WebSocketTick>>,
    sink: std::sync::Arc<
        rocket::futures::lock::Mutex<
            rocket::futures::stream::SplitSink<ws::stream::DuplexStream, ws::Message>,
        >,
    >,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut receiver = ticker.subscribe();

    while let Some(ws_tick) = receiver.recv().await.ok() {
        if ws_tick.room_id != room_id {
            continue;
        }

        let mut message = Vec::with_capacity(2);

        message.push(BINARY_PROTOCOL_VERSION);

        sink.lock()
            .await
            .send(ws::Message::Text("tick".to_string()))
            .await?;
    }

    Ok(())
}

#[rocket::get("/?<sid>")]
pub async fn ws_endpoint<'st>(
    sid: &'st str,
    game_state: &'st rocket::State<model::GameState>,
    messages: &'st rocket::State<tokio::sync::broadcast::Sender<model::WebSocketMessage>>,
    ticker: &'st rocket::State<tokio::sync::broadcast::Sender<model::WebSocketTick>>,
    ws: ws::WebSocket,
) -> Result<ws::Channel<'st>, String> {
    let users = game_state.users.lock().await;
    let user = find_user_by_id(&users, sid)
        .ok_or("User not found")?
        .clone();

    return Ok(ws.channel(move |duplex: ws::stream::DuplexStream| {
        Box::pin(async move {
            let (sink, stream) = duplex.split();
            let sink = std::sync::Arc::new(rocket::futures::lock::Mutex::new(sink));

            let reader =
                create_reader(user.room_id.clone(), messages, ticker, game_state, stream);
            let room_writer =
                create_room_writer(user.room_id.clone(), messages, sink.clone());
            let timer = create_timer(user.room_id.clone(), ticker, sink.clone());

            // Wait for either the readers or writers to finish (They stop executing).
            // This allows us to both read and write to the websocket at the same time.
            tokio::select! {
                _ = reader => {},
                _ = room_writer => {},
                _ = timer => {},
            }

            Ok(())
        })
    }));
}

#[rocket::post("/handshake", data = "<form>")]
pub async fn handshake_endpoint(
    game_state: &rocket::State<model::GameState>,
    form: Result<rocket::form::Form<model::HandshakeData>, rocket::form::Errors<'_>>,
) -> Result<model::HandshakePayload, (rocket::http::Status, String)> {
    let form = form.map_err(|_err| {
        (
            rocket::http::Status::UnprocessableEntity,
            "Display name is required and must be between 3 and 20 characters long."
                .to_string(),
        )
    })?;

    let mut rooms = game_state.rooms.lock().await;
    let mut users = game_state.users.lock().await;

    if form.room_id.is_empty() {
        let available_room = rooms.iter().find(|room| {
            room.state == model::RoomState::Waiting
                && room.visibility == model::Visibility::Public
                && room.max_users
                    > users.iter().fold(0, |acc, user| {
                        if user.room_id == room.id {
                            return acc + 1;
                        }

                        acc
                    })
        });

        match available_room {
            Some(room) => {
                let user_id = utils::gen_random_id();
                let user =
                    model::User::new(user_id, form.display_name.clone(), room.id.clone());

                users.push(user.clone());

                let users_in_room = users
                    .iter()
                    .filter_map(|user| {
                        if user.room_id == room.id {
                            return Some(user.clone());
                        }

                        None
                    })
                    .collect::<Vec<model::User>>();

                return Ok(model::HandshakePayload {
                    user,
                    room: room.clone(),
                    users_in_room,
                });
            }
            None => {
                let room_id = utils::gen_random_id();
                let user_id = utils::gen_random_id();
                let room = model::Room::new(
                    room_id.clone(),
                    user_id.clone(),
                    model::Visibility::Public,
                );
                let user = model::User::new(user_id, form.display_name.clone(), room_id);
                let users_in_room = vec![user.clone()];

                rooms.push(room.clone());
                users.push(user.clone());

                return Ok(model::HandshakePayload {
                    user,
                    room,
                    users_in_room,
                });
            }
        }
    }

    let room = find_room_by_id(&rooms, &form.room_id)
        .ok_or((rocket::http::Status::NotFound, "Room not found".to_string()))?;

    if room.state != model::RoomState::Waiting {
        return Err((
            rocket::http::Status::Conflict,
            "Room is currently in game".to_string(),
        ));
    }

    let user_id = utils::gen_random_id();
    let user = model::User::new(user_id, form.display_name.clone(), room.id.clone());

    users.push(user.clone());

    let users_in_room = users
        .iter()
        .filter_map(|user| {
            if user.room_id == room.id {
                return Some(user.clone());
            }

            None
        })
        .collect::<Vec<model::User>>();

    Ok(model::HandshakePayload {
        user,
        room: room.clone(),
        users_in_room,
    })
}
