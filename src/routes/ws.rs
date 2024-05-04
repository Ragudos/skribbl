use rand::Rng;
use rocket::futures::{SinkExt, StreamExt};
use rocket::tokio;

use crate::model::{self, WebSocketMessage};
use crate::utils;
use crate::utils::consts::BINARY_PROTOCOL_VERSION;

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

fn get_version_or_error_on_mismatch(
    data: &Vec<u8>,
) -> Result<&u8, Box<dyn std::error::Error>> {
    if data.len() < 1 {
        return Err("Invalid data length".into());
    }

    let version = &data[0];

    if *version != BINARY_PROTOCOL_VERSION {
        return Err("Invalid protocol version".into());
    }

    Ok(version)
}

fn get_event_type_or_error_on_mismatch(
    data: &Vec<u8>,
) -> Result<model::WebSocketEvents, Box<dyn std::error::Error>> {
    if data.len() < 2 {
        return Err("Invalid data length".into());
    }

    Ok(data[1].try_into()?)
}

async fn create_reader<'st>(
    user_id: String,
    room_id: String,
    messages: &'st rocket::State<tokio::sync::broadcast::Sender<WebSocketMessage>>,
    ticker: &'st rocket::State<tokio::sync::broadcast::Sender<model::WebSocketTick>>,
    game_state: &'st rocket::State<model::GameState>,
    mut stream: rocket::futures::stream::SplitStream<ws::stream::DuplexStream>,
    sink: std::sync::Arc<
        rocket::futures::lock::Mutex<
            rocket::futures::stream::SplitSink<ws::stream::DuplexStream, ws::Message>,
        >,
    >,
) -> Result<(), Box<dyn std::error::Error>> {
    while let Some(message) = stream.next().await {
        let message = message?;

        match &message {
            ws::Message::Close(close_frames) => {
                if let Some(close_frames) = &close_frames {
                    println!(
                        "Closing websocket: \n\tReason{:?} Code: {:?}",
                        close_frames.reason, close_frames.code
                    );
                }

                let mut users = game_state.users.lock().await;
                let mut rooms = game_state.rooms.lock().await;

                let Some(room) = rooms.iter_mut().find(|r| r.id == room_id) else {
                    unreachable!("Room should exist when a user leaves.");
                };

                if room.amount_of_users == 0 {
                    unreachable!("A room with no users should have been deleted")
                }

                if room.amount_of_users == 1 {
                    let Some(user_idx) = users.iter().position(|u| u.id == user_id)
                    else {
                        unreachable!("User should exist when leaving the room.");
                    };
                    let Some(room_idx) = rooms.iter().position(|r| r.id == room_id)
                    else {
                        unreachable!("Room should exist when a user leaves.");
                    };

                    users.remove(user_idx);
                    rooms.remove(room_idx);

                    break;
                } else if room.amount_of_users == 2 {
                    match &room.state {
                        &model::RoomState::Playing { .. } => {
                            if let Some(user) = users.iter_mut().find(|u| {
                                u.room_id == room_id && u.id != user_id && u.has_drawn
                            }) {
                                user.has_drawn = false;
                            }

                            room.state = model::RoomState::Waiting;
                        }
                        _ => {}
                    }
                }

                let Some(user_pos) = users.iter().position(|u| u.id == user_id) else {
                    unreachable!("User should exist when leaving the room.");
                };

                users.remove(user_pos);
                room.amount_of_users -= 1;

                let user_id_bytes = user_id.as_bytes();
                let user_id_bytes_length = user_id_bytes.len();
                let mut message = Vec::with_capacity(2 + 1 + 1 + user_id_bytes_length);

                message.push(BINARY_PROTOCOL_VERSION);
                message.push(
                    model::WebSocketEvents::UserLeft
                        .try_into()
                        .unwrap(),
                );
                message.push(1);
                message.push(user_id_bytes_length as u8);
                message.extend(user_id_bytes);

                let _ = messages.send(model::WebSocketMessage::new(
                    None,
                    room_id.clone(),
                    ws::Message::Binary(message),
                ));

                if user_id == room.host_id {
                    // We already removed the user who opened this connection, so
                    // we can just find the next possible user in the room who can be the host.
                    let Some(new_host) = users.iter().find(|u| u.room_id == room_id)
                    else {
                        unreachable!("There should be at least one user in the room.");
                    };
                    let user_id = new_host.id.clone();
                    let user_id_bytes = user_id.as_bytes();
                    let user_id_bytes_length = user_id_bytes.len();

                    let mut message =
                        Vec::with_capacity(2 + 1 + 1 + user_id_bytes_length);

                    message.push(BINARY_PROTOCOL_VERSION);
                    message.push(
                        model::WebSocketEvents::NewHost
                            .try_into()
                            .unwrap(),
                    );
                    message.push(1);
                    message.push(user_id_bytes_length as u8);
                    message.extend(user_id_bytes);

                    room.host_id = user_id;

                    let _ = messages.send(model::WebSocketMessage::new(
                        None,
                        room_id.clone(),
                        ws::Message::Binary(message),
                    ));

                    break;
                }

                (|| match &mut room.state {
                    model::RoomState::Playing {
                        user_to_draw,
                        current_round,
                        time_left,
                        current_word,
                    } => {
                        if user_id != *user_to_draw {
                            return;
                        }

                        if users
                            .iter()
                            .all(|user| user.room_id == room.id && user.has_drawn)
                        {
                            if *current_round == room.max_rounds {
                                room.state = model::RoomState::Finished;

                                let message = vec![
                                    BINARY_PROTOCOL_VERSION,
                                    model::WebSocketEvents::EndGame.try_into().expect(
                                        "EndGame event should be transformable to u8",
                                    ),
                                ];

                                let _ = messages.send(model::WebSocketMessage::new(
                                    None,
                                    room_id.clone(),
                                    ws::Message::Binary(message),
                                ));

                                return;
                            }

                            *current_round += 1;

                            users.iter_mut().for_each(|user| {
                                if user.room_id == room.id {
                                    user.has_drawn = false;
                                }
                            });
                            // TODO: Send new round event here
                            return;
                        }

                        let Some(user_left_to_draw) = users
                            .iter_mut()
                            .find(|u| u.room_id == room.id && !u.has_drawn)
                        else {
                            unreachable!(
                                "There should be at least one user who hasn't drawn yet."
                            );
                        };

                        user_left_to_draw.has_drawn = true;

                        let user_left_to_draw_id = user_left_to_draw.id.clone();
                        *user_to_draw = user_left_to_draw_id.clone();

                        let word = utils::get_random_word().to_string();
                        let duration = 60;

                        *current_word = word.clone();
                        *time_left = duration.clone();

                        let user_left_to_draw_id_bytes = user_left_to_draw_id.as_bytes();
                        let user_left_to_draw_id_bytes_length =
                            user_left_to_draw_id_bytes.len();
                        let current_word_bytes = word.as_bytes();
                        let current_word_bytes_length = current_word_bytes.len();
                        let time_left_bytes = vec![duration.clone()];
                        let time_left_bytes_length = time_left_bytes.len();

                        let mut message = Vec::with_capacity(
                            2 + 1
                                + 1
                                + user_left_to_draw_id_bytes_length
                                + 1
                                + 1
                                + current_word_bytes_length
                                + 1
                                + 1
                                + time_left_bytes_length,
                        );

                        message.push(BINARY_PROTOCOL_VERSION);
                        message.push(
                            model::WebSocketEvents::NewUserToDraw
                                .try_into()
                                .expect(
                                    "NewUserToDraw event should be transformable to u8",
                                ),
                        );
                        message.push(1);
                        message.push(user_left_to_draw_id_bytes_length as u8);
                        message.extend(user_left_to_draw_id_bytes);
                        message.push(1);
                        message.push(current_word_bytes_length as u8);
                        message.extend(current_word_bytes);
                        message.push(1);
                        message.push(time_left_bytes_length as u8);
                        message.extend(time_left_bytes);

                        let _ = messages.send(model::WebSocketMessage::new(
                            None,
                            room_id.clone(),
                            ws::Message::Binary(message),
                        ));
                    }
                    _ => {}
                })();

                break;
            }
            ws::Message::Pong(_) => {
                continue;
            }
            ws::Message::Ping(_) => {
                let _ = sink
                    .lock()
                    .await
                    .send(ws::Message::Pong(vec![]))
                    .await;
                continue;
            }
            _ => {}
        }

        let data = message.into_data();

        get_version_or_error_on_mismatch(&data)?;

        let event_type = get_event_type_or_error_on_mismatch(&data)?;

        match event_type {
            model::WebSocketEvents::StartGame => {
                let mut rooms = game_state.rooms.lock().await;
                let Some(room) = rooms.iter_mut().find(|r| r.id == room_id) else {
                    unreachable!("Room should exist when sending the StartGame event.");
                };

                if room.host_id != user_id {
                    let mut sink = sink.lock().await;

                    let error_msg = "Only the host can start the game";
                    let error_msg_bytes = error_msg.as_bytes();
                    let error_msg_bytes_length = error_msg_bytes.len();

                    let mut message =
                        Vec::with_capacity(2 + 1 + 1 + error_msg_bytes_length);

                    message.push(BINARY_PROTOCOL_VERSION);
                    message.push(model::WebSocketEvents::Error.try_into().unwrap());
                    message.push(1);
                    message.push(error_msg_bytes_length as u8);
                    message.extend(error_msg_bytes);

                    sink.send(ws::Message::Binary(message)).await?;

                    continue;
                }

                let mut users = game_state.users.lock().await;
                let mut users_in_room = users
                    .iter_mut()
                    .filter(|u| u.room_id == room_id)
                    .collect::<Vec<&mut model::User>>();
                let num_of_users_in_room = users_in_room.len();

                if num_of_users_in_room == 0 {
                    unreachable!("A room with no users should have been deleted");
                } else if num_of_users_in_room == 1 {
                    let mut sink = sink.lock().await;

                    let error_msg = "Not enough players to start the game";
                    let error_msg_bytes = error_msg.as_bytes();
                    let error_msg_bytes_length = error_msg_bytes.len();

                    let mut message =
                        Vec::with_capacity(2 + 1 + 1 + error_msg_bytes_length);

                    message.push(BINARY_PROTOCOL_VERSION);
                    message.push(model::WebSocketEvents::Error.try_into().unwrap());
                    message.push(1);
                    message.push(error_msg_bytes_length as u8);
                    message.extend(error_msg_bytes);

                    sink.send(ws::Message::Binary(message)).await?;

                    continue;
                }

                let random_idx = rand::thread_rng().gen_range(0..num_of_users_in_room);
                let user_to_draw = &mut *users_in_room[random_idx];

                user_to_draw.has_drawn = true;

                let user_to_draw_id = user_to_draw.id.clone();
                let word = utils::get_random_word().to_string();
                let duration = 60;

                room.state = model::RoomState::Playing {
                    user_to_draw: user_to_draw_id.clone(),
                    time_left: duration.clone(),
                    current_round: 1,
                    current_word: word.clone(),
                };

                let mut sink = sink.lock().await;

                let user_to_draw_id_bytes = user_to_draw_id.as_bytes();
                let user_to_draw_id_bytes_length = user_to_draw_id_bytes.len();
                let current_word_bytes = word.as_bytes();
                let current_word_bytes_length = current_word_bytes.len();
                let time_left_bytes = vec![duration.clone()];
                let time_left_bytes_length = time_left_bytes.len();

                let mut message = Vec::with_capacity(
                    2 + 1
                        + 1
                        + user_to_draw_id_bytes_length
                        + 1
                        + 1
                        + current_word_bytes_length
                        + 1
                        + 1
                        + time_left_bytes_length,
                );

                message.push(BINARY_PROTOCOL_VERSION);
                message.push(
                    model::WebSocketEvents::StartGame
                        .try_into()
                        .unwrap(),
                );
                message.push(1);
                message.push(user_to_draw_id_bytes_length as u8);
                message.extend(user_to_draw_id_bytes);
                message.push(1);
                message.push(current_word_bytes_length as u8);
                message.extend(current_word_bytes);
                message.push(1);
                message.push(time_left_bytes_length as u8);
                message.extend(time_left_bytes);

                sink.send(ws::Message::Binary(message)).await?;

                // We only want to send the raw current word to the current user who's
                // drawing. We don't want to send the current word to the other users.
                // So, we process the current word and replace all characters with '*'
                // in the socket writer of other clients.
                let _ = messages.send(model::WebSocketMessage::new(
                    Some(user_to_draw_id.clone()),
                    room_id.clone(),
                    ws::Message::Binary(data),
                ));

                spawn_timer(room_id.clone(), game_state, ticker).await;
            }
            _ => {
                let _ = messages.send(model::WebSocketMessage::new(
                    None,
                    room_id.clone(),
                    ws::Message::Binary(data),
                ));
            }
        }
    }

    Ok(())
}

async fn create_room_writer<'st>(
    user_id: String,
    room_id: String,
    messages: &'st rocket::State<tokio::sync::broadcast::Sender<WebSocketMessage>>,
    game_state: &'st rocket::State<model::GameState>,
    sink: std::sync::Arc<
        rocket::futures::lock::Mutex<
            rocket::futures::stream::SplitSink<ws::stream::DuplexStream, ws::Message>,
        >,
    >,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut receiver = messages.subscribe();

    while let Some(ws_message) = receiver.recv().await.ok() {
        if ws_message.room_id != room_id
            || ws_message.user_id_to_exclude.as_ref() == Some(&user_id)
        {
            continue;
        }

        let message = ws_message.message;
        let data = message.into_data();

        get_version_or_error_on_mismatch(&data)?;

        let event_type = get_event_type_or_error_on_mismatch(&data)?;

        match event_type {
            model::WebSocketEvents::StartGame => {
                let rooms = game_state.rooms.lock().await;
                let Some(room) = rooms.iter().find(|r| r.id == room_id) else {
                    unreachable!("Room should exist when sending the StartGame event.");
                };

                match &room.state {
                    model::RoomState::Playing {
                        user_to_draw,
                        current_word,
                        time_left,
                        ..
                    } => {
                        let user_to_draw_bytes = user_to_draw.as_bytes();
                        let user_to_draw_bytes_length = user_to_draw_bytes.len();
                        let current_word_transformed = current_word
                            .chars()
                            .map(|c| {
                                if c.is_whitespace() {
                                    return ' ';
                                }

                                '*'
                            })
                            .collect::<String>();
                        let current_word_bytes = current_word_transformed.as_bytes();
                        let current_word_bytes_length = current_word_bytes.len();
                        let time_left_bytes = vec![time_left.clone()];
                        let time_left_bytes_length = time_left_bytes.len();

                        let mut sink = sink.lock().await;

                        let mut message = Vec::with_capacity(
                            2 + 1
                                + 1
                                + user_to_draw_bytes_length
                                + 1
                                + 1
                                + current_word_bytes_length
                                + 1
                                + 1
                                + time_left_bytes_length,
                        );

                        message.push(BINARY_PROTOCOL_VERSION);
                        message.push(
                            model::WebSocketEvents::StartGame
                                .try_into()
                                .unwrap(),
                        );
                        message.push(1);
                        message.push(user_to_draw_bytes_length as u8);
                        message.extend(user_to_draw_bytes);
                        message.push(1);
                        message.push(current_word_bytes_length as u8);
                        message.extend(current_word_bytes);
                        message.push(1);
                        message.push(time_left_bytes_length as u8);
                        message.extend(time_left_bytes);

                        sink.send(ws::Message::Binary(message)).await?;
                    }
                    _ => {
                        unreachable!("Room should be in Playing state when sending the StartGame event.");
                    }
                }
            }
            _ => {
                let mut sink = sink.lock().await;

                sink.send(ws::Message::Binary(data)).await?;
            }
        }
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
    game_state: &'st rocket::State<model::GameState>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut receiver = ticker.subscribe();

    while let Some(ws_tick) = receiver.recv().await.ok() {
        if ws_tick.room_id != room_id {
            continue;
        }

        let rooms = game_state.rooms.lock().await;
        let Some(room) = rooms.iter().find(|r| r.id == room_id) else {
            unreachable!("Room should exist when sending the WebSocket Tick event.");
        };

        match &room.state {
            &model::RoomState::Playing { time_left, .. } => {
                let time_left_bytes = vec![time_left.clone()];
                let time_left_bytes_length = time_left_bytes.len();
                let mut message = Vec::with_capacity(2 + 1 + 1 + time_left_bytes_length);

                message.push(BINARY_PROTOCOL_VERSION);
                message.push(model::WebSocketEvents::Tick.try_into().unwrap());
                message.push(1);
                message.push(time_left_bytes_length as u8);
                message.extend(time_left_bytes);

                sink.lock()
                    .await
                    .send(ws::Message::Binary(message))
                    .await?;
            }
            _ => unreachable!(
                "Room should be in Playing state when sending the WebSocket Tick event."
            ),
        }
    }

    Ok(())
}

#[rocket::get("/?<sid>")]
pub async fn ws_endpoint<'st>(
    sid: &str,
    game_state: &'st rocket::State<model::GameState>,
    messages: &'st rocket::State<tokio::sync::broadcast::Sender<model::WebSocketMessage>>,
    ticker: &'st rocket::State<tokio::sync::broadcast::Sender<model::WebSocketTick>>,
    ws: ws::WebSocket,
) -> Result<ws::Channel<'st>, String> {
    let users = game_state.users.lock().await;
    let user = utils::realtime::find_user_by_id(&users, sid)
        .ok_or("User not found")?
        .clone();

    game_state
        .rooms
        .lock()
        .await
        .iter_mut()
        .for_each(|room| {
            if room.id == user.room_id {
                room.amount_of_users += 1;
            }
        });

    let stringified_user = serde_json::to_string(&user).map_err(|err| {
        rocket::error!("Failed to serialize user: {:?}", err);
        "Something went wrong".to_string()
    })?;
    let stringified_user_bytes = stringified_user.as_bytes();
    let stringified_user_bytes_length = stringified_user_bytes.len();
    let mut message = Vec::with_capacity(2 + 1 + 1 + stringified_user_bytes_length);

    message.push(BINARY_PROTOCOL_VERSION);
    message.push(
        model::WebSocketEvents::UserJoined
            .try_into()
            .expect("UserJoined event should be transformable to u8"),
    );
    message.push(1);
    message.push(stringified_user_bytes_length as u8);
    message.extend(stringified_user_bytes);

    let _ = messages.send(WebSocketMessage::new(
        None,
        user.room_id.clone(),
        ws::Message::Binary(message),
    ));

    return Ok(ws.channel(move |duplex: ws::stream::DuplexStream| {
        Box::pin(async move {
            let (sink, stream) = duplex.split();
            let sink = std::sync::Arc::new(rocket::futures::lock::Mutex::new(sink));

            let reader = create_reader(
                user.id.clone(),
                user.room_id.clone(),
                messages,
                ticker,
                game_state,
                stream,
                sink.clone(),
            );
            let room_writer = create_room_writer(
                user.id.clone(),
                user.room_id.clone(),
                messages,
                game_state,
                sink.clone(),
            );
            let timer =
                create_timer(user.room_id.clone(), ticker, sink.clone(), game_state);

            // Wait for either the readers or writers to finish (They stop executing).
            // This allows us to both read and write to the websocket at the same time.
            tokio::select! {
                _ = reader => {},
                _ = room_writer => {},
                _ = timer => {},
            }

            println!("WebSocket connection closed");

            Ok(())
        })
    }));
}

#[rocket::post("/handshake", data = "<form>")]
pub async fn handshake_endpoint<'st>(
    game_state: &rocket::State<model::GameState>,
    form: Result<rocket::form::Form<model::HandshakeData>, rocket::form::Errors<'_>>,
) -> Result<model::HandshakePayload, (rocket::http::Status, &'st str)> {
    let form = form
        .map_err(|_err| {
            (
                rocket::http::Status::UnprocessableEntity,
                "Display name is required and must be between 3 and 20 characters long.",
            )
        })?
        .into_inner();

    if form.mode == model::HandshakeMode::Create {
        let room_id = utils::gen_random_id();
        let user_id = utils::gen_random_id();
        let room = model::RoomBuilder::default()
            .id(room_id.clone())
            .host_id(user_id.clone())
            .build()
            .unwrap();
        let user = model::UserBuilder::default()
            .id(user_id)
            .room_id(room_id)
            .display_name(form.display_name)
            .build()
            .unwrap();

        let users_in_room = vec![user.clone()];

        game_state.rooms.lock().await.push(room.clone());
        game_state.users.lock().await.push(user.clone());

        return Ok(model::HandshakePayloadBuilder::default()
            .user(user)
            .room(room)
            .users_in_room(users_in_room)
            .build()
            .unwrap());
    }

    if form.room_id.is_empty() {
        if let Some(room) =
            utils::realtime::find_available_public_room(&game_state.rooms.lock().await)
        {
            let user = model::UserBuilder::default()
                .id(utils::gen_random_id())
                .display_name(form.display_name)
                .room_id(room.id.clone())
                .build()
                .unwrap();
            let mut users = game_state.users.lock().await;

            users.push(user.clone());

            let users_in_room =
                utils::realtime::get_and_clone_users_in_room(&users, &room.id);

            return Ok(model::HandshakePayload {
                user,
                room: room.clone(),
                users_in_room,
                binary_protocol_version: BINARY_PROTOCOL_VERSION,
            });
        }

        let room_id = utils::gen_random_id();
        let user_id = utils::gen_random_id();
        let room = model::RoomBuilder::default()
            .id(room_id.clone())
            .host_id(user_id.clone())
            .build()
            .unwrap();
        let user = model::UserBuilder::default()
            .id(user_id)
            .room_id(room_id)
            .display_name(form.display_name)
            .build()
            .unwrap();

        let users_in_room = vec![user.clone()];

        game_state.rooms.lock().await.push(room.clone());
        game_state.users.lock().await.push(user.clone());

        return Ok(model::HandshakePayloadBuilder::default()
            .user(user)
            .room(room)
            .users_in_room(users_in_room)
            .build()
            .unwrap());
    }

    let rooms = game_state.rooms.lock().await;
    let room = utils::realtime::find_room_by_id(&rooms, &form.room_id)
        .ok_or((rocket::http::Status::NotFound, "Room not found"))?;

    if room.state != model::RoomState::Waiting {
        return Err((rocket::http::Status::Conflict, "Room is currently in game"));
    }

    let user = model::UserBuilder::default()
        .id(utils::gen_random_id())
        .display_name(form.display_name)
        .room_id(room.id.clone())
        .build()
        .unwrap();
    let mut users = game_state.users.lock().await;

    users.push(user.clone());

    let users_in_room = utils::realtime::get_and_clone_users_in_room(&users, &room.id);

    Ok(model::HandshakePayloadBuilder::default()
        .user(user)
        .room(room.clone())
        .users_in_room(users_in_room)
        .build()
        .unwrap())
}
