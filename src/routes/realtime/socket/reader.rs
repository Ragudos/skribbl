use rand::Rng;
use rocket::tokio;
use rocket::{futures::StreamExt, http::hyper::server};

use crate::{events, state};

enum WebSocketOperationResult {
    Continue,
    Break,
}

pub async fn create_websocket_reader(
    room_id: String,
    user_id: String,
    game_state: &rocket::State<state::GameState>,
    mut stream: rocket::futures::stream::SplitStream<ws::stream::DuplexStream>,
    server_messages: &rocket::State<
        tokio::sync::broadcast::Sender<events::WebSocketMessage>,
    >,
) -> Result<(), Box<dyn std::error::Error>> {
    while let Some(message) = stream.next().await {
        let message = message?;

        match message {
            ws::Message::Binary(data) => {
                let borrowed = &data;
                let event_type = borrowed.try_into()?;

                match event_type {
                    events::ClientToServerEvents::StartGame => match start_game_event(
                        &room_id,
                        &user_id,
                        game_state,
                        server_messages,
                    )
                    .await?
                    {
                        WebSocketOperationResult::Continue => continue,
                        WebSocketOperationResult::Break => break,
                    },
                    _ => {}
                }

                // Put this outside so we don't have to clone the data
                let _ = events::WebSocketMessageBuilder::default()
                    .r#type(events::WebSocketMessageType::Broadcast {
                        sender_id: user_id.clone(),
                    })
                    .room_id(room_id.clone())
                    .message(ws::Message::Binary(data))
                    .build()?
                    .send(server_messages);
            }
            ws::Message::Close(close_frame) => {
                if let Some(close_frame) = &close_frame {
                    println!("Closing connection: {:#?}", close_frame);
                }

                on_reader_close(&room_id, &user_id, game_state, server_messages).await?;

                break;
            }
            _ => {}
        }
    }

    Ok(())
}

async fn on_reader_close(
    room_id: &str,
    user_id: &str,
    game_state: &rocket::State<state::GameState>,
    server_messages: &rocket::State<
        tokio::sync::broadcast::Sender<events::WebSocketMessage>,
    >,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut rooms = game_state.rooms.lock().await;
    let mut users = game_state.users.lock().await;

    let room = rooms
        .iter_mut()
        .find(|room| room.id == room_id)
        .ok_or("Room not found")?;

    if room.amount_of_users == 0 {
        panic!("Room should have at least one user");
    }

    if room.amount_of_users == 1 {
        let room_idx = rooms
            .iter()
            .position(|room| room.id == room_id)
            .ok_or("Room not found")?;
        let user_idx = users
            .iter()
            .position(|user| user.id == user_id)
            .ok_or("User not found")?;

        rooms.remove(room_idx);
        users.remove(user_idx);

        return Ok(());
    }

    match &mut room.state {
        state::RoomState::Playing {
            playing_state,
            current_user_id,
            current_round,
        } => {
            if room.amount_of_users == 2 {
                room.state = state::RoomState::Waiting;

                let _ = events::WebSocketMessageBuilder::default()
                    .room_id(room_id.to_string())
                    .r#type(events::WebSocketMessageType::Broadcast {
                        sender_id: user_id.to_string(),
                    })
                    .message(ws::Message::Binary(
                        events::ServerToClientEvents::ResetRoom.try_into()?,
                    ))
                    .build()?
                    .send(server_messages);
            } else if current_user_id == user_id {
                // If the user who disconnects is the one who's currently the one to draw,
                // then we need to handle certain scenarios, like whether to end the game,
                // go to a new round, or pick a new user who'll draw.
                if users.iter().fold(0, |acc, user| {
                    if user.room_id == room_id && user.id != user_id && !user.has_drawn {
                        acc + 1
                    } else {
                        acc
                    }
                }) == 0
                {
                    if *current_round == room.max_rounds {
                        room.state = state::RoomState::Finished;

                        let _ = events::WebSocketMessageBuilder::default()
                            .room_id(room_id.to_string())
                            .r#type(events::WebSocketMessageType::Broadcast {
                                sender_id: user_id.to_string(),
                            })
                            .message(ws::Message::Binary(
                                events::ServerToClientEvents::EndGame.try_into()?,
                            ))
                            .build()?
                            .send(server_messages);
                    } else {
                        *current_round += 1;

                        let _ = events::WebSocketMessageBuilder::default()
                            .room_id(room_id.to_string())
                            .r#type(events::WebSocketMessageType::Broadcast {
                                sender_id: user_id.to_string(),
                            })
                            .message(ws::Message::Binary(
                                events::ServerToClientEvents::NewRound {
                                    round: current_round.clone(),
                                }
                                .try_into()?,
                            ))
                            .build()?
                            .send(server_messages);

                        users.iter_mut().for_each(|user| {
                            if user.room_id == room_id {
                                user.has_drawn = false;
                            }
                        });

                        let users = users
                            .iter()
                            .filter(|user| room.id == user.room_id)
                            .collect::<Vec<_>>();
                        let rng = rand::thread_rng().gen_range(0..users.len());
                        let user_to_draw = *users
                            .get(rng)
                            .ok_or("Cannot find a new user to draw")?;

                        *current_user_id = user_to_draw.id.clone();

                        let _ = events::WebSocketMessageBuilder::default()
                            .room_id(room_id.to_string())
                            .r#type(events::WebSocketMessageType::Broadcast {
                                sender_id: user_id.to_string(),
                            })
                            .message(ws::Message::Binary(
                                events::ServerToClientEvents::NewTurn {
                                    user_id_to_draw: user_to_draw.id.clone(),
                                }
                                .try_into()?,
                            ))
                            .build()?
                            .send(server_messages);

                        let words_to_pick = state::WordToDraw::get_three_words();

                        *playing_state = state::PlayingState::PickingAWord {
                            words_to_pick: words_to_pick.clone(),
                            started_at: time::OffsetDateTime::now_utc(),
                        };

                        let _ = events::WebSocketMessageBuilder::default()
                            .room_id(room_id.to_string())
                            .r#type(events::WebSocketMessageType::User {
                                receiver_id: user_to_draw.id.clone(),
                            })
                            .message(ws::Message::Binary(
                                events::ServerToClientEvents::PickAWord { words_to_pick }
                                    .try_into()?,
                            ))
                            .build()?
                            .send(server_messages);
                    }
                } else {
                    let users_in_room_left_to_draw = users
                        .iter()
                        .filter(|user| {
                            room.id == user.room_id
                                && user.id != user_id
                                && !user.has_drawn
                        })
                        .collect::<Vec<_>>();
                    let rng =
                        rand::thread_rng().gen_range(0..users_in_room_left_to_draw.len());
                    let user_to_draw = *users_in_room_left_to_draw
                        .get(rng)
                        .ok_or("Cannot find a new user to draw")?;

                    *current_user_id = user_to_draw.id.clone();

                    let _ = events::WebSocketMessageBuilder::default()
                        .room_id(room_id.to_string())
                        .r#type(events::WebSocketMessageType::Broadcast {
                            sender_id: user_id.to_string(),
                        })
                        .message(ws::Message::Binary(
                            events::ServerToClientEvents::NewTurn {
                                user_id_to_draw: user_to_draw.id.clone(),
                            }
                            .try_into()?,
                        ))
                        .build()?
                        .send(server_messages);

                    let words_to_pick = state::WordToDraw::get_three_words();

                    *playing_state = state::PlayingState::PickingAWord {
                        words_to_pick: words_to_pick.clone(),
                        started_at: time::OffsetDateTime::now_utc(),
                    };

                    let _ = events::WebSocketMessageBuilder::default()
                        .room_id(room_id.to_string())
                        .r#type(events::WebSocketMessageType::User {
                            receiver_id: user_to_draw.id.clone(),
                        })
                        .message(ws::Message::Binary(
                            events::ServerToClientEvents::PickAWord { words_to_pick }
                                .try_into()?,
                        ))
                        .build()?
                        .send(server_messages);
                }
            }
        }
        _ => {}
    }

    if user_id == room.host_id {
        let new_host = users
            .iter()
            .find(|user| user.room_id == room_id && user.id != user_id)
            .ok_or("Cannot find any user to be the new host")?;

        room.host_id = new_host.id.clone();

        let _ = events::WebSocketMessageBuilder::default()
            .room_id(room_id.to_string())
            .r#type(events::WebSocketMessageType::Broadcast {
                sender_id: user_id.to_string(),
            })
            .message(ws::Message::Binary(
                events::ServerToClientEvents::NewHost {
                    user_id: new_host.id.clone(),
                }
                .try_into()?,
            ))
            .build()?
            .send(server_messages);
    }

    let _ = events::WebSocketMessageBuilder::default()
        .room_id(room_id.to_string())
        .r#type(events::WebSocketMessageType::Broadcast {
            sender_id: user_id.to_string(),
        })
        .message(ws::Message::Binary(
            events::ServerToClientEvents::UserLeft {
                user_id: user_id.to_string(),
            }
            .try_into()?,
        ))
        .build()?
        .send(server_messages);

    let user_idx = users
        .iter()
        .position(|user| user.id == user_id)
        .ok_or("User not found")?;

    users.remove(user_idx);

    Ok(())
}

async fn start_game_event(
    room_id: &str,
    user_id: &str,
    game_state: &rocket::State<state::GameState>,
    server_messages: &rocket::State<
        tokio::sync::broadcast::Sender<events::WebSocketMessage>,
    >,
) -> Result<WebSocketOperationResult, Box<dyn std::error::Error>> {
    let mut rooms = game_state.rooms.lock().await;
    let room = rooms
        .iter_mut()
        .find(|room| room.id == room_id)
        .ok_or("Room not found")?;

    if room.host_id != user_id {
        let _ = events::WebSocketMessageBuilder::default()
            .room_id(room_id.to_string())
            .r#type(events::WebSocketMessageType::User {
                receiver_id: user_id.to_string(),
            })
            .message(ws::Message::Binary(
                events::ServerToClientEvents::Error {
                    message: "Only the host can start the game".to_string(),
                }
                .try_into()?,
            ))
            .build()?
            .send(server_messages);

        return Ok(WebSocketOperationResult::Continue);
    }

    if room.state != state::RoomState::Waiting {
        let _ = events::WebSocketMessageBuilder::default()
            .room_id(room_id.to_string())
            .r#type(events::WebSocketMessageType::User {
                receiver_id: user_id.to_string(),
            })
            .message(ws::Message::Binary(
                events::ServerToClientEvents::Error {
                    message: "Game has already started".to_string(),
                }
                .try_into()?,
            ))
            .build()?
            .send(server_messages);

        return Ok(WebSocketOperationResult::Continue);
    }

    let users = game_state.users.lock().await;
    let users = users
        .iter()
        .filter(|user| room.id == user.room_id)
        .collect::<Vec<_>>();
    let rng = rand::thread_rng().gen_range(0..users.len());
    let Some(user_to_draw) = users.get(rng) else {
        let Some(room_idx) = rooms.iter().position(|room| room.id == room_id) else {
            return Ok(WebSocketOperationResult::Continue);
        };

        rooms.remove(room_idx);

        return Ok(WebSocketOperationResult::Break);
    };
    let words_to_pick = state::WordToDraw::get_three_words();

    room.state = state::RoomState::Playing {
        playing_state: state::PlayingState::PickingAWord {
            words_to_pick: words_to_pick.clone(),
            started_at: time::OffsetDateTime::now_utc(),
        },
        current_user_id: user_to_draw.id.clone(),
        current_round: 1,
    };

    let _ = events::WebSocketMessageBuilder::default()
        .room_id(room_id.to_string())
        .r#type(events::WebSocketMessageType::Everyone)
        .message(ws::Message::Binary(
            events::ServerToClientEvents::StartGame.try_into()?,
        ))
        .build()?
        .send(server_messages);

    let _ = events::WebSocketMessageBuilder::default()
        .room_id(room_id.to_string())
        .r#type(events::WebSocketMessageType::Everyone)
        .message(ws::Message::Binary(
            events::ServerToClientEvents::NewTurn {
                user_id_to_draw: user_to_draw.id.clone(),
            }
            .try_into()?,
        ))
        .build()?
        .send(server_messages);

    let _ = events::WebSocketMessageBuilder::default()
        .room_id(room_id.to_string())
        .r#type(events::WebSocketMessageType::User {
            receiver_id: user_to_draw.id.clone(),
        })
        .message(ws::Message::Binary(
            events::ServerToClientEvents::PickAWord { words_to_pick }.try_into()?,
        ))
        .build()?
        .send(server_messages);

    Ok(WebSocketOperationResult::Continue)
}
