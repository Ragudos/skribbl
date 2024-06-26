use rand::Rng;
use rocket::tokio;
use rocket::{futures::StreamExt, tokio::sync::broadcast::error::RecvError};

use crate::state::TickerCommand;
use crate::{events, state, utils};

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
    ticker_msg: &rocket::State<tokio::sync::broadcast::Sender<state::TickerMsg>>,
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
                        ticker_msg,
                    )
                    .await?
                    {
                        WebSocketOperationResult::Continue => continue,
                        WebSocketOperationResult::Break => break,
                    },
                    events::ClientToServerEvents::PickAWord { word } => {
                        let _ = ticker_msg.send(state::TickerMsg {
                            room_id: room_id.to_string(),
                            command: state::TickerCommand::Delete,
                        });

                        let mut rooms = game_state.rooms.lock().await;
                        let Some(room) = rooms.iter_mut().find(|r| r.id == room_id)
                        else {
                            break;
                        };
                        let state::RoomState::Playing { playing_state, .. } =
                            &mut room.state
                        else {
                            break;
                        };

                        {
                            if let state::PlayingState::Drawing { .. } = playing_state {
                                eprintln!("Received event `PickAWord` but room's `PlayingState` is `Drawing`");

                                let _ = events::WebSocketMessageBuilder::default()
                                    .r#type(events::WebSocketMessageType::User {
                                        receiver_id: user_id.clone(),
                                    })
                                    .room_id(room_id.clone())
                                    .message(ws::Message::Binary(
                                        events::ServerToClientEvents::Error {
                                            message: "Something went wrong.".to_string(),
                                        }
                                        .try_into()?,
                                    ))
                                    .build()?
                                    .send(server_messages);

                                break;
                            }
                        }

                        *playing_state = state::PlayingState::Drawing {
                            current_word: word.clone(),
                            time_left: utils::consts::DRAW_IME_LIMIT,
                        };

                        // TODO: Add check if we need to send this by checking if someone is
                        // drawing.

                        let _ = events::WebSocketMessageBuilder::default()
                            .r#type(events::WebSocketMessageType::Broadcast {
                                sender_id: user_id.clone(),
                            })
                            .room_id(room_id.clone())
                            .message(ws::Message::Binary(
                                events::ServerToClientEvents::NewWord {
                                    word: utils::obfuscate_word(&word),
                                }
                                .try_into()?,
                            ))
                            .build()?
                            .send(server_messages);
                        let _ = events::WebSocketMessageBuilder::default()
                            .r#type(events::WebSocketMessageType::User {
                                receiver_id: user_id.clone(),
                            })
                            .room_id(room_id.clone())
                            .message(ws::Message::Binary(
                                events::ServerToClientEvents::NewWord { word }
                                    .try_into()?,
                            ))
                            .build()?
                            .send(server_messages);

                        create_ticker(
                            &room_id,
                            game_state.rooms.clone(),
                            game_state.users.clone(),
                            server_messages.inner().clone(),
                            ticker_msg.inner().clone(),
                        );
                    }
                    events::ClientToServerEvents::PointerUp => {
                        // TODO: Add check if we need to send this by checking if someone is
                        // drawing.

                        let _ = events::WebSocketMessageBuilder::default()
                            .r#type(events::WebSocketMessageType::Everyone)
                            .room_id(room_id.clone())
                            .message(ws::Message::Binary(
                                events::ServerToClientEvents::PointerUp.try_into()?,
                            ))
                            .build()?
                            .send(server_messages);
                    }
                    events::ClientToServerEvents::PointerDown => {
                        // TODO: Add check if we need to send this by checking if someone is
                        // drawing.

                        let _ = events::WebSocketMessageBuilder::default()
                            .r#type(events::WebSocketMessageType::Everyone)
                            .room_id(room_id.clone())
                            .message(ws::Message::Binary(
                                events::ServerToClientEvents::PointerDown.try_into()?,
                            ))
                            .build()?
                            .send(server_messages);
                    }
                    events::ClientToServerEvents::PointerLeave => {
                        // TODO: Add check if we need to send this by checking if someone is
                        // drawing.

                        let _ = events::WebSocketMessageBuilder::default()
                            .r#type(events::WebSocketMessageType::Everyone)
                            .room_id(room_id.clone())
                            .message(ws::Message::Binary(
                                events::ServerToClientEvents::PointerLeave.try_into()?,
                            ))
                            .build()?
                            .send(server_messages);
                    }
                    events::ClientToServerEvents::PointerMove { x, y } => {
                        // TODO: Add check if we need to send this by checking if someone is
                        // drawing.

                        let _ = events::WebSocketMessageBuilder::default()
                            .r#type(events::WebSocketMessageType::Everyone)
                            .room_id(room_id.clone())
                            .message(ws::Message::Binary(
                                events::ServerToClientEvents::PointerMove { x, y }
                                    .try_into()?,
                            ))
                            .build()?
                            .send(server_messages);
                    }
                    events::ClientToServerEvents::ChangeColor { color } => {
                        // TODO: Add check if we need to send this by checking if someone is
                        // drawing.
                        let _ = events::WebSocketMessageBuilder::default()
                            .r#type(events::WebSocketMessageType::Everyone)
                            .room_id(room_id.clone())
                            .message(ws::Message::Binary(
                                events::ServerToClientEvents::ChangeColor { color }
                                    .try_into()?,
                            ))
                            .build()?
                            .send(server_messages);
                    }
                    events::ClientToServerEvents::Message { message } => {
                        match on_message(
                            message,
                            &room_id,
                            &user_id,
                            server_messages,
                            game_state,
                            ticker_msg,
                        )
                        .await?
                        {
                            WebSocketOperationResult::Break => break,
                            WebSocketOperationResult::Continue => continue,
                        }
                    }
                }
            }
            ws::Message::Close(close_frame) => {
                if let Some(close_frame) = &close_frame {
                    println!("Closing connection: {:#?}", close_frame);
                }

                break;
            }
            _ => {}
        }
    }

    on_reader_close(&room_id, &user_id, game_state, server_messages, ticker_msg).await?;

    Ok(())
}

async fn on_reader_close(
    room_id: &str,
    user_id_who_disconnected: &str,
    game_state: &rocket::State<state::GameState>,
    server_messages: &rocket::State<
        tokio::sync::broadcast::Sender<events::WebSocketMessage>,
    >,
    ticker_msg: &rocket::State<tokio::sync::broadcast::Sender<state::TickerMsg>>,
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

    let user_idx = users
        .iter()
        .position(|user| user.id == user_id_who_disconnected)
        .ok_or("User not found")?;

    users.remove(user_idx);
    room.amount_of_users -= 1;

    if room.amount_of_users == 0 {
        let room_idx = rooms
            .iter()
            .position(|room| room.id == room_id)
            .ok_or("Room not found")?;

        rooms.remove(room_idx);

        return Ok(());
    }

    if room.state != state::RoomState::Waiting && room.state != state::RoomState::Finished
    {
        if room.amount_of_users == 1 {
            reset_room(
                room,
                room_id,
                user_id_who_disconnected,
                server_messages,
                ticker_msg,
            )?;
        } else {
            handle_playing_room(
                &mut users,
                room,
                room_id,
                user_id_who_disconnected,
                server_messages,
                ticker_msg,
                game_state,
            )?;
        }
    }

    if user_id_who_disconnected == room.host_id {
        handle_new_host(
            &users,
            room,
            room_id,
            user_id_who_disconnected,
            server_messages,
        )?;
    }

    let _ = events::WebSocketMessageBuilder::default()
        .room_id(room_id.to_string())
        .r#type(events::WebSocketMessageType::Broadcast {
            sender_id: user_id_who_disconnected.to_string(),
        })
        .message(ws::Message::Binary(
            events::ServerToClientEvents::UserLeft {
                user_id: user_id_who_disconnected.to_string(),
            }
            .try_into()?,
        ))
        .build()?
        .send(server_messages);

    Ok(())
}

fn handle_new_host<'st>(
    users: &'st [state::User],
    room: &'st mut state::Room,
    room_id: &str,
    user_id_who_disconnected: &str,
    server_messages: &rocket::State<
        tokio::sync::broadcast::Sender<events::WebSocketMessage>,
    >,
) -> Result<(), Box<dyn std::error::Error>> {
    let new_host = users
        .iter()
        .find(|user| user.room_id == room_id)
        .ok_or("Cannot find any user to be the new host")?;

    room.host_id = new_host.id.clone();

    let _ = events::WebSocketMessageBuilder::default()
        .room_id(room_id.to_string())
        .r#type(events::WebSocketMessageType::Broadcast {
            sender_id: user_id_who_disconnected.to_string(),
        })
        .message(ws::Message::Binary(
            events::ServerToClientEvents::NewHost {
                user_id: new_host.id.clone(),
            }
            .try_into()?,
        ))
        .build()?
        .send(server_messages);

    Ok(())
}

fn handle_playing_room(
    users: &mut [state::User],
    room: &mut state::Room,
    room_id: &str,
    user_id_who_disconnected: &str,
    server_messages: &rocket::State<
        tokio::sync::broadcast::Sender<events::WebSocketMessage>,
    >,
    ticker_msg: &rocket::State<tokio::sync::broadcast::Sender<state::TickerMsg>>,
    game_state: &rocket::State<state::GameState>,
) -> Result<(), Box<dyn std::error::Error>> {
    if let state::RoomState::Playing {
        current_user_id, ..
    } = &room.state
    {
        if current_user_id != user_id_who_disconnected {
            return Ok(());
        }
    } else {
        unreachable!();
    }

    // We just delete an existing ticker if the user who disconnected is the one
    // who's drawing since we would be changing the timer no matter  what.
    let _ = ticker_msg.send(state::TickerMsg {
        room_id: room_id.to_string(),
        command: state::TickerCommand::Delete,
    });

    let amount_of_users_who_has_not_drawn = users.iter().fold(0, |acc, user| {
        if user.room_id == room_id && !user.has_drawn {
            acc + 1
        } else {
            acc
        }
    });
    let is_in_last_round = match &room.state {
        state::RoomState::Playing { current_round, .. } => {
            *current_round == room.max_rounds
        }
        _ => unreachable!(),
    };

    users.iter_mut().for_each(|user| {
        if user.room_id == room_id {
            user.has_guessed = false;
        }
    });

    if amount_of_users_who_has_not_drawn == 0 && is_in_last_round {
        room.state = state::RoomState::Finished;

        let _ = events::WebSocketMessageBuilder::default()
            .room_id(room_id.to_string())
            .r#type(events::WebSocketMessageType::Broadcast {
                sender_id: user_id_who_disconnected.to_string(),
            })
            .message(ws::Message::Binary(
                events::ServerToClientEvents::EndGame.try_into()?,
            ))
            .build()?
            .send(server_messages);

        return Ok(());
    }

    let state::RoomState::Playing {
        playing_state,
        current_user_id,
        current_round,
    } = &mut room.state
    else {
        unreachable!();
    };

    if amount_of_users_who_has_not_drawn == 0 && !is_in_last_round {
        *current_round += 1;

        let _ = events::WebSocketMessageBuilder::default()
            .room_id(room_id.to_string())
            .r#type(events::WebSocketMessageType::Broadcast {
                sender_id: user_id_who_disconnected.to_string(),
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

        handle_new_turn(
            users,
            current_user_id,
            playing_state,
            user_id_who_disconnected,
            room_id,
            server_messages,
        )?;

        create_ticker(
            room_id,
            game_state.rooms.clone(),
            game_state.users.clone(),
            server_messages.inner().clone(),
            ticker_msg.inner().clone(),
        );
    }

    if amount_of_users_who_has_not_drawn != 0 {
        handle_new_turn(
            users,
            current_user_id,
            playing_state,
            user_id_who_disconnected,
            room_id,
            server_messages,
        )?;

        create_ticker(
            room_id,
            game_state.rooms.clone(),
            game_state.users.clone(),
            server_messages.inner().clone(),
            ticker_msg.inner().clone(),
        );
    }

    Ok(())
}

fn handle_new_turn(
    users: &mut [state::User],
    current_user_to_draw_id: &mut String,
    playing_state: &mut state::PlayingState,
    user_id_who_disconnected: &str,
    room_id: &str,
    server_messages: &rocket::State<
        tokio::sync::broadcast::Sender<events::WebSocketMessage>,
    >,
) -> Result<(), Box<dyn std::error::Error>> {
    let user_to_draw = utils::choose_user_in_a_room_randomly(users, room_id)?;

    *current_user_to_draw_id = user_to_draw.id.clone();

    let _ = events::WebSocketMessageBuilder::default()
        .room_id(room_id.to_string())
        .r#type(events::WebSocketMessageType::Broadcast {
            sender_id: user_id_who_disconnected.to_string(),
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
        time_left: utils::consts::PICK_WORD_TIME_LIMIT,
    };

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

    Ok(())
}

fn reset_room(
    room: &mut state::Room,
    room_id: &str,
    user_id: &str,
    server_messages: &rocket::State<
        tokio::sync::broadcast::Sender<events::WebSocketMessage>,
    >,
    ticker_msg: &rocket::State<tokio::sync::broadcast::Sender<state::TickerMsg>>,
) -> Result<(), Box<dyn std::error::Error>> {
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

    let _ = ticker_msg.send(state::TickerMsg {
        room_id: room_id.to_string(),
        command: state::TickerCommand::Delete,
    });

    Ok(())
}

async fn start_game_event(
    room_id: &str,
    user_id: &str,
    game_state: &rocket::State<state::GameState>,
    server_messages: &rocket::State<
        tokio::sync::broadcast::Sender<events::WebSocketMessage>,
    >,
    ticker_msg: &rocket::State<tokio::sync::broadcast::Sender<state::TickerMsg>>,
) -> Result<WebSocketOperationResult, Box<dyn std::error::Error>> {
    let mut rooms = game_state.rooms.lock().await;
    let room = rooms
        .iter_mut()
        .find(|room| room.id == room_id)
        .ok_or("Room not found")?;

    if room.amount_of_users == 1 {
        let _ = events::WebSocketMessageBuilder::default()
            .room_id(room_id.to_string())
            .r#type(events::WebSocketMessageType::User {
                receiver_id: user_id.to_string(),
            })
            .message(ws::Message::Binary(
                events::ServerToClientEvents::Error {
                    message: "Need at least 2 players to start the game".to_string(),
                }
                .try_into()?,
            ))
            .build()?
            .send(server_messages);

        return Ok(WebSocketOperationResult::Continue);
    }

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

    let mut users = game_state.users.lock().await;
    let Ok(user_to_draw) = utils::choose_user_in_a_room_randomly(&mut users, room_id)
    else {
        println!("User to draw not found");

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
            time_left: utils::consts::PICK_WORD_TIME_LIMIT,
        },
        current_user_id: user_to_draw.id.clone(),
        current_round: 1,
    };

    user_to_draw.has_drawn = true;

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

    create_ticker(
        room_id,
        game_state.rooms.clone(),
        game_state.users.clone(),
        server_messages.inner().clone(),
        ticker_msg.inner().clone(),
    );

    Ok(WebSocketOperationResult::Continue)
}

fn create_ticker(
    room_id: &str,
    rooms: std::sync::Arc<rocket::futures::lock::Mutex<Vec<state::Room>>>,
    users: std::sync::Arc<rocket::futures::lock::Mutex<Vec<state::User>>>,
    server_messages: tokio::sync::broadcast::Sender<events::WebSocketMessage>,
    ticker_msg: tokio::sync::broadcast::Sender<state::TickerMsg>,
) {
    let mut ticker_msg_rx = ticker_msg.subscribe();
    let room_id = room_id.to_string();

    tokio::spawn(async move {
        let mut interval =
            tokio::time::interval(tokio::time::Duration::from_millis(1_000));

        loop {
            tokio::select! {
                _ = interval.tick() => match on_tick(
                    &room_id,
                    &rooms,
                    &users,
                    &server_messages,
                    &ticker_msg
                ).await {
                    Ok(WebSocketOperationResult::Break) => break,
                    Ok(WebSocketOperationResult::Continue) => continue,
                    Err(err) => {
                        eprintln!("{:?}", err);
                        break;
                    }
                },
                msg = ticker_msg_rx.recv() => {
                    match msg {
                        Ok(msg) => {
                            if room_id != msg.room_id {
                                continue;
                            }

                            match msg.command {
                                TickerCommand::Delete => break
                            }
                        },
                        Err(RecvError::Closed) => break,
                        Err(RecvError::Lagged(err)) => {
                            eprintln!("Lagged in ticker: {:?}", err);
                            continue;
                        }
                    }
                }
            }
        }

        println!("Exiting timer");
    });
}

async fn on_tick(
    room_id: &str,
    rooms: &std::sync::Arc<rocket::futures::lock::Mutex<Vec<state::Room>>>,
    users: &std::sync::Arc<rocket::futures::lock::Mutex<Vec<state::User>>>,
    server_messages: &tokio::sync::broadcast::Sender<events::WebSocketMessage>,
    ticker_msg: &tokio::sync::broadcast::Sender<state::TickerMsg>,
) -> Result<WebSocketOperationResult, Box<dyn std::error::Error>> {
    let mut mut_rooms = rooms.lock().await;
    let Some(room) = mut_rooms.iter_mut().find(|r| r.id == room_id) else {
        return Ok(WebSocketOperationResult::Break);
    };

    let state::RoomState::Playing { playing_state, .. } = &mut room.state else {
        return Ok(WebSocketOperationResult::Break);
    };

    let time_left = match playing_state {
        state::PlayingState::Drawing { time_left, .. } => time_left,
        state::PlayingState::PickingAWord { time_left, .. } => time_left,
    };

    let _ = server_messages.send(
        events::WebSocketMessageBuilder::default()
            .room_id(room_id.to_string())
            .r#type(events::WebSocketMessageType::Everyone)
            .message(ws::Message::Binary(
                events::ServerToClientEvents::Tick {
                    time_left: time_left.clone(),
                }
                .try_into()
                .unwrap(),
            ))
            .build()
            .unwrap(),
    );

    if *time_left == 0 {
        if on_timer_reached_zero(room_id, room, users, server_messages, ticker_msg)
            .await?
        {
            create_ticker(
                room_id,
                rooms.clone(),
                users.clone(),
                server_messages.clone(),
                ticker_msg.clone(),
            );
        }

        return Ok(WebSocketOperationResult::Break);
    }

    if *time_left != 0 {
        *time_left -= 1;
    }

    Ok(WebSocketOperationResult::Continue)
}

/// Returns true if a ticker should be created again.
async fn on_timer_reached_zero(
    room_id: &str,
    room: &mut state::Room,
    users: &std::sync::Arc<rocket::futures::lock::Mutex<Vec<state::User>>>,
    server_messages: &tokio::sync::broadcast::Sender<events::WebSocketMessage>,
    ticker_msg: &tokio::sync::broadcast::Sender<state::TickerMsg>,
) -> Result<bool, Box<dyn ::std::error::Error>> {
    let state::RoomState::Playing {
        playing_state,
        current_round,
        ..
    } = &mut room.state
    else {
        unreachable!()
    };

    match *playing_state {
        state::PlayingState::PickingAWord { .. } => {
            match start_drawing(room_id, room, server_messages) {
                Ok(WebSocketOperationResult::Break) => Ok(false),
                Ok(WebSocketOperationResult::Continue) => Ok(true),
                Err(err) => {
                    eprintln!("{:?}", err);

                    Ok(false)
                }
            }
        }
        state::PlayingState::Drawing { .. } => {
            if *current_round == room.max_rounds {
                match end_game(
                    room_id,
                    server_messages,
                    room,
                    &mut users.lock().await,
                    ticker_msg,
                ) {
                    Err(err) => eprintln!("{:?}", err),
                    _ => {}
                };

                return Ok(false);
            }

            let mut users = users.lock().await;

            if users
                .iter()
                .any(|user| user.room_id == room_id && !user.has_drawn)
            {
                return match next_turn(room_id, server_messages, &mut users, room) {
                    Ok(WebSocketOperationResult::Break) => Ok(false),
                    Ok(WebSocketOperationResult::Continue) => Ok(true),
                    Err(err) => {
                        eprintln!("{:?}", err);

                        Ok(false)
                    }
                };
            }

            return match next_round(room_id, server_messages, &mut users, room) {
                Ok(WebSocketOperationResult::Break) => Ok(false),
                Ok(WebSocketOperationResult::Continue) => Ok(true),
                Err(err) => {
                    eprintln!("{:?}", err);

                    Ok(false)
                }
            };
        }
    }
}

async fn on_message(
    message: String,
    room_id: &str,
    user_id: &str,
    server_messages: &rocket::State<
        tokio::sync::broadcast::Sender<events::WebSocketMessage>,
    >,
    game_state: &rocket::State<state::GameState>,
    ticker_msg: &rocket::State<tokio::sync::broadcast::Sender<state::TickerMsg>>,
) -> Result<WebSocketOperationResult, Box<dyn std::error::Error>> {
    let mut rooms = game_state.rooms.lock().await;
    let Some(room) = rooms.iter_mut().find(|room| room.id == room_id) else {
        eprintln!("Received `Message` event but room does not exist");

        return Ok(WebSocketOperationResult::Break);
    };

    if let state::RoomState::Playing {
        playing_state,
        current_round,
        current_user_id,
    } = &mut room.state
    {
        if let state::PlayingState::Drawing { current_word, .. } = playing_state {
            if *current_word == message {
                if user_id == &*current_user_id {
                    let _ = events::WebSocketMessageBuilder::default()
                        .r#type(events::WebSocketMessageType::User {
                            receiver_id: user_id.to_string(),
                        })
                        .room_id(room_id.to_string())
                        .message(ws::Message::Binary(
                            events::ServerToClientEvents::Error {
                                message: "You cannot expose the word being drawn"
                                    .to_string(),
                            }
                            .try_into()?,
                        ))
                        .build()?
                        .send(server_messages);

                    return Ok(WebSocketOperationResult::Continue);
                }

                let mut users = game_state.users.lock().await;

                {
                    // We do this (getting the user twice, on here and on `user_guessed`)
                    // because in the future, `user_guessed` will need to borrow the `users`
                    // for the scoring system.
                    let Some(user) = users.iter().find(|user| user.id == user_id) else {
                        eprintln!("Received `Message` event but user does not exist");

                        return Ok(WebSocketOperationResult::Break);
                    };

                    if user.has_guessed {
                        user_already_guessed(room_id, user_id, server_messages)?;

                        return Ok(WebSocketOperationResult::Continue);
                    }
                }

                user_guessed(
                    room_id,
                    user_id,
                    &current_word,
                    server_messages,
                    &mut users,
                )?;

                if !users.iter().any(|user| {
                    if current_user_id == &(*user).id {
                        return false;
                    }

                    return user.room_id == room_id && !user.has_guessed;
                }) {
                    if *current_round == room.max_rounds {
                        return end_game(
                            room_id,
                            server_messages,
                            room,
                            &mut users,
                            ticker_msg,
                        );
                    }

                    let _ = ticker_msg.send(state::TickerMsg {
                        room_id: room_id.to_string(),
                        command: state::TickerCommand::Delete,
                    });

                    if users
                        .iter()
                        .any(|user| user.room_id == room_id && !user.has_drawn)
                    {
                        let res = next_turn(room_id, server_messages, &mut users, room);

                        create_ticker(
                            room_id,
                            game_state.rooms.clone(),
                            game_state.users.clone(),
                            server_messages.inner().clone(),
                            ticker_msg.inner().clone(),
                        );

                        return res;
                    }

                    let res = next_round(room_id, server_messages, &mut users, room);

                    create_ticker(
                        room_id,
                        game_state.rooms.clone(),
                        game_state.users.clone(),
                        server_messages.inner().clone(),
                        ticker_msg.inner().clone(),
                    );

                    return res;
                }

                return Ok(WebSocketOperationResult::Continue);
            }
        }
    }

    let _ = events::WebSocketMessageBuilder::default()
        .r#type(events::WebSocketMessageType::Everyone)
        .room_id(room_id.to_string())
        .message(ws::Message::Binary(
            events::ServerToClientEvents::Message {
                user_id: user_id.to_string(),
                message,
            }
            .try_into()?,
        ))
        .build()?
        .send(server_messages);

    Ok(WebSocketOperationResult::Continue)
}

fn user_guessed(
    room_id: &str,
    user_id: &str,
    word_to_draw: &str,
    server_messages: &tokio::sync::broadcast::Sender<events::WebSocketMessage>,
    users: &mut [state::User],
) -> Result<(), Box<dyn std::error::Error>> {
    let Some(user) = users.iter_mut().find(|user| user.id == user_id) else {
        panic!("Calling `user_guessed` but user does not exist");
    };

    user.has_guessed = true;
    user.score += 10;
    // TODO: Add a scoring system. For now, we add +10

    let _ = events::WebSocketMessageBuilder::default()
        .r#type(events::WebSocketMessageType::Everyone)
        .room_id(room_id.to_string())
        .message(ws::Message::Binary(
            events::ServerToClientEvents::AddScore {
                user_id: user_id.to_string(),
                score: 10,
            }
            .try_into()?,
        ))
        .build()?
        .send(server_messages);

    let _ = events::WebSocketMessageBuilder::default()
        .r#type(events::WebSocketMessageType::Everyone)
        .room_id(room_id.to_string())
        .message(ws::Message::Binary(
            events::ServerToClientEvents::UserGuessed {
                user_id: user_id.to_string(),
            }
            .try_into()?,
        ))
        .build()?
        .send(server_messages);

    let _ = events::WebSocketMessageBuilder::default()
        .r#type(events::WebSocketMessageType::Everyone)
        .room_id(room_id.to_string())
        .message(ws::Message::Binary(
            events::ServerToClientEvents::SystemMessage {
                message: format!("{} has guessed the word!", user.display_name.clone()),
            }
            .try_into()?,
        ))
        .build()?
        .send(server_messages);

    let _ = events::WebSocketMessageBuilder::default()
        .r#type(events::WebSocketMessageType::User {
            receiver_id: user_id.to_string(),
        })
        .room_id(room_id.to_string())
        .message(ws::Message::Binary(
            events::ServerToClientEvents::RevealWord {
                word: word_to_draw.to_string(),
            }
            .try_into()?,
        ))
        .build()?
        .send(server_messages);

    Ok(())
}

fn user_already_guessed(
    room_id: &str,
    user_id: &str,
    server_messages: &tokio::sync::broadcast::Sender<events::WebSocketMessage>,
) -> Result<(), Box<dyn std::error::Error>> {
    let _ = events::WebSocketMessageBuilder::default()
        .r#type(events::WebSocketMessageType::User {
            receiver_id: user_id.to_string(),
        })
        .room_id(room_id.to_string())
        .message(ws::Message::Binary(
            events::ServerToClientEvents::Error {
                message: "You cannot expose the word being drawn.".to_string(),
            }
            .try_into()?,
        ))
        .build()?
        .send(server_messages);

    Ok(())
}

fn start_drawing(
    room_id: &str,
    room: &mut state::Room,
    server_messages: &tokio::sync::broadcast::Sender<events::WebSocketMessage>,
) -> Result<WebSocketOperationResult, Box<dyn std::error::Error>> {
    let state::RoomState::Playing {
        playing_state,
        current_user_id,
        ..
    } = &mut room.state
    else {
        panic!("Calling `start_drawing` but room is not in playing state.");
    };

    let word_to_draw = match playing_state {
        state::PlayingState::Drawing { .. } => panic!(
            "Calling `start_drawing` but room is already in a drawing playing state."
        ),
        state::PlayingState::PickingAWord { words_to_pick, .. } => words_to_pick
            [rand::thread_rng().gen_range(0..words_to_pick.len())]
        .to_string(),
    };

    *playing_state = state::PlayingState::Drawing {
        current_word: word_to_draw.clone(),
        time_left: utils::consts::DRAW_IME_LIMIT,
    };

    let _ = events::WebSocketMessageBuilder::default()
        .r#type(events::WebSocketMessageType::User {
            receiver_id: current_user_id.clone(),
        })
        .room_id(room_id.to_string())
        .message(ws::Message::Binary(
            events::ServerToClientEvents::NewWord {
                word: word_to_draw.clone(),
            }
            .try_into()?,
        ))
        .build()?
        .send(server_messages);

    let _ = events::WebSocketMessageBuilder::default()
        .r#type(events::WebSocketMessageType::Broadcast {
            sender_id: current_user_id.clone(),
        })
        .room_id(room_id.to_string())
        .message(ws::Message::Binary(
            events::ServerToClientEvents::NewWord {
                word: utils::obfuscate_word(&word_to_draw),
            }
            .try_into()?,
        ))
        .build()?
        .send(server_messages);

    Ok(WebSocketOperationResult::Continue)
}

fn next_round(
    room_id: &str,
    server_messages: &tokio::sync::broadcast::Sender<events::WebSocketMessage>,
    users: &mut [state::User],
    room: &mut state::Room,
) -> Result<WebSocketOperationResult, Box<dyn std::error::Error>> {
    let state::RoomState::Playing { current_round, .. } = &mut room.state else {
        panic!("Called `next_round` despite room not in playing state");
    };

    assert_eq!(users.iter().any(|user| user.room_id == room_id && !user.has_drawn), false, "Allow a call to `next_round` if all users in a room has drawn for the current round.");
    assert_ne!(*current_round, room.max_rounds, "Allow a call to `next_round` if the current round has not reached the maximum round set in a room");

    users.iter_mut().for_each(|user| {
        if user.room_id == room_id {
            (*user).has_drawn = false;
        }
    });

    *current_round += 1;

    let _ = events::WebSocketMessageBuilder::default()
        .r#type(events::WebSocketMessageType::Everyone)
        .room_id(room_id.to_string())
        .message(ws::Message::Binary(
            events::ServerToClientEvents::NewRound {
                round: current_round.clone(),
            }
            .try_into()?,
        ))
        .build()?
        .send(server_messages);

    next_turn(room_id, server_messages, users, room)
}

fn next_turn(
    room_id: &str,
    server_messages: &tokio::sync::broadcast::Sender<events::WebSocketMessage>,
    users: &mut [state::User],
    room: &mut state::Room,
) -> Result<WebSocketOperationResult, Box<dyn std::error::Error>> {
    println!("{:?}", room);

    let state::RoomState::Playing {
        playing_state,
        current_user_id,
        ..
    } = &mut room.state
    else {
        panic!("Calling `next_turn` despite room not in playing state.");
    };

    users.iter_mut().for_each(|user| {
        if user.room_id == room_id {
            (*user).has_guessed = false;
        }
    });

    let mut users_in_room_who_has_not_drawn = users
        .iter_mut()
        .filter(|user| user.room_id == room_id && !user.has_drawn)
        .collect::<Vec<&mut state::User>>();

    let users_in_room_who_has_not_drawn_length = users_in_room_who_has_not_drawn.len();
    let user_to_draw = &mut *users_in_room_who_has_not_drawn
        [rand::thread_rng().gen_range(0..users_in_room_who_has_not_drawn_length)];
    let words_to_pick = state::WordToDraw::get_three_words();

    *playing_state = state::PlayingState::PickingAWord {
        words_to_pick: words_to_pick.clone(),
        time_left: utils::consts::PICK_WORD_TIME_LIMIT,
    };
    user_to_draw.has_drawn = true;
    *current_user_id = user_to_draw.id.clone();

    let _ = events::WebSocketMessageBuilder::default()
        .r#type(events::WebSocketMessageType::Everyone)
        .room_id(room_id.to_string())
        .message(ws::Message::Binary(
            events::ServerToClientEvents::NewTurn {
                user_id_to_draw: user_to_draw.id.clone(),
            }
            .try_into()?,
        ))
        .build()?
        .send(server_messages);

    let _ = events::WebSocketMessageBuilder::default()
        .r#type(events::WebSocketMessageType::User {
            receiver_id: user_to_draw.id.clone(),
        })
        .room_id(room_id.to_string())
        .message(ws::Message::Binary(
            events::ServerToClientEvents::PickAWord { words_to_pick }.try_into()?,
        ))
        .build()?
        .send(server_messages);

    Ok(WebSocketOperationResult::Continue)
}

fn end_game(
    room_id: &str,
    server_messages: &tokio::sync::broadcast::Sender<events::WebSocketMessage>,
    room: &mut state::Room,
    users: &mut [state::User],
    ticker_msg: &tokio::sync::broadcast::Sender<state::TickerMsg>,
) -> Result<WebSocketOperationResult, Box<dyn std::error::Error>> {
    let _ = ticker_msg.send(state::TickerMsg {
        room_id: room_id.to_string(),
        command: state::TickerCommand::Delete,
    });

    room.state = state::RoomState::Finished;

    users
        .iter_mut()
        .filter(|user| user.room_id == room_id)
        .for_each(|user| {
            user.has_drawn = false;
            user.has_guessed = false;
            user.score = 0;
        });

    let _ = events::WebSocketMessageBuilder::default()
        .r#type(events::WebSocketMessageType::Everyone)
        .room_id(room_id.to_string())
        .message(ws::Message::Binary(
            events::ServerToClientEvents::EndGame.try_into()?,
        ))
        .build()?
        .send(server_messages);

    Ok(WebSocketOperationResult::Continue)
}
