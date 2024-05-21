use crate::{events, state, utils};
use rocket::{
    futures::{SinkExt, StreamExt},
    tokio,
};

pub mod reader;
pub mod writer;

#[derive(rocket::FromFormField)]
pub enum WsEndpointMode {
    #[field(value = "play")]
    Play,
    #[field(value = "create")]
    Create,
}

#[derive(rocket::FromForm)]
pub struct WsEndpointParams {
    #[field(validate = len(3..20))]
    #[field(name = "displayName")]
    pub display_name: String,
    #[field(name = "roomId")]
    pub room_id: String,
    pub mode: WsEndpointMode,
}

#[rocket::get("/?<params..>")]
pub async fn ws_endpoint<'st>(
    game_state: &'st rocket::State<state::GameState>,
    server_messages: &'st rocket::State<
        tokio::sync::broadcast::Sender<events::WebSocketMessage>,
    >,
    ticker_msg: &'st rocket::State<tokio::sync::broadcast::Sender<state::TickerMsg>>,
    params: Result<WsEndpointParams, rocket::form::Errors<'st>>,
    ws: ws::WebSocket,
) -> ws::Channel<'st> {
    use reader;
    use writer;

    ws.channel(move |duplex: ws::stream::DuplexStream| {
        Box::pin(async move {
            let (mut sink, stream) = duplex.split();
            let room_id: String;
            let user_id: String;

            match params {
                Ok(params) => {
                    match params.mode {
                        WsEndpointMode::Play => {
                            if params.room_id.is_empty() {
                                let mut rooms = game_state.rooms.lock().await;
                                let mut users = game_state.users.lock().await;

                                if let Some(available_room) = rooms.iter_mut().find(|room| {
                                    room.state == state::RoomState::Waiting
                                        && room.amount_of_users < room.max_users
                                }) {
                                    let new_user_id = utils::gen_random_id();
                                    let user = state::UserBuilder::default()
                                        .id(new_user_id.clone())
                                        .display_name(params.display_name)
                                        .room_id(available_room.id.clone())
                                        .build()
                                        .unwrap();

                                    users.push(user.clone());
                                    available_room.amount_of_users += 1;

                                    sink.send(
                                        ws::Message::Binary(
                                            events::ServerToClientEvents::SendGameState {
                                                room: available_room.clone(),
                                                user: user.clone(),
                                                users_in_room: users
                                                    .iter()
                                                    .filter(|user| {
                                                        user.room_id == available_room.id
                                                    })
                                                    .cloned()
                                                    .collect::<Vec<state::User>>()
                                            }
                                            .try_into()
                                            .unwrap()
                                        )
                                    ).await?;

                                    let _ = events::WebSocketMessageBuilder::default()
                                        .r#type(events::WebSocketMessageType::Broadcast { sender_id: new_user_id.clone() })
                                        .room_id(available_room.id.clone())
                                        .message(ws::Message::Binary(events::ServerToClientEvents::UserJoined { user }.try_into().unwrap()))
                                        .build()
                                        .unwrap().send(server_messages);

                                    room_id = available_room.id.clone();
                                    user_id = new_user_id;
                                } else {
                                    let new_room_id = utils::gen_random_id();
                                    let new_user_id = utils::gen_random_id();
                                    let room = state::RoomBuilder::default()
                                        .id(new_room_id.clone())
                                        .host_id(new_user_id.clone())
                                        .build()
                                        .unwrap();
                                    let user = state::UserBuilder::default()
                                        .id(new_user_id.clone())
                                        .display_name(params.display_name)
                                        .room_id(new_room_id.clone())
                                        .build()
                                        .unwrap();

                                    rooms.push(room.clone());
                                    users.push(user.clone());

                                    sink.send(
                                        ws::Message::Binary(
                                            events::ServerToClientEvents::SendGameState {
                                                room,
                                                user: user.clone(),
                                                users_in_room: vec![user]
                                            }
                                            .try_into()
                                            .unwrap()
                                        )
                                    ).await?;

                                    room_id = new_room_id;
                                    user_id = new_user_id;
                                };
                            } else {
                                let mut rooms = game_state.rooms.lock().await;
                                let Some(room) = rooms.iter_mut().find(|room| {
                                    room.id == params.room_id
                                }) else {
                                    sink.send(
                                        ws::Message::Binary(events::ServerToClientEvents::ConnectError {
                                            message: "Room not found".to_string(),
                                        }
                                        .try_into()
                                        .unwrap()),
                                    )
                                    .await?;
                                    sink.close().await?;
                                    return Ok(());
                                };

                                if room.state != state::RoomState::Waiting {
                                    sink.send(
                                        ws::Message::Binary(events::ServerToClientEvents::ConnectError {
                                            message: "Room is not available".to_string(),
                                        }
                                        .try_into()
                                        .unwrap())
                                    )
                                    .await?;
                                    sink.close().await?;
                                    return Ok(());
                                }

                                if room.amount_of_users == room.max_users {
                                    sink.send(
                                        ws::Message::Binary(events::ServerToClientEvents::ConnectError {
                                            message: "Room is full".to_string(),
                                        }
                                        .try_into()
                                        .unwrap()),
                                    )
                                    .await?;
                                    sink.close().await?;
                                    return Ok(());
                                }

                                let new_user_id = utils::gen_random_id();
                                let user = state::UserBuilder::default()
                                    .id(new_user_id.clone())
                                    .display_name(params.display_name)
                                    .room_id(room.id.clone())
                                    .build()
                                    .unwrap();
                                let mut users = game_state.users.lock().await;

                                users.push(user.clone());
                                room.amount_of_users += 1;

                                sink.send(
                                    ws::Message::Binary(
                                        events::ServerToClientEvents::SendGameState {
                                            room: room.clone(),
                                            user: user.clone(),
                                            users_in_room: users
                                                .iter()
                                                .filter(|user| {
                                                    user.room_id == room.id
                                                })
                                                .cloned()
                                                .collect::<Vec<state::User>>()
                                        }
                                        .try_into()
                                        .unwrap()
                                    )
                                ).await?;

                                let _ = events::WebSocketMessageBuilder::default()
                                    .r#type(events::WebSocketMessageType::Broadcast { sender_id: new_user_id.clone() })
                                    .room_id(room.id.clone())
                                    .message(ws::Message::Binary(events::ServerToClientEvents::UserJoined { user }.try_into().unwrap()))
                                    .build()
                                    .unwrap().send(server_messages);

                                room_id = room.id.clone();
                                user_id = new_user_id;
                            }
                        }
                        WsEndpointMode::Create => {
                            let new_room_id = utils::gen_random_id();
                            let new_user_id = utils::gen_random_id();
                            let room = state::RoomBuilder::default()
                                .id(new_room_id.clone())
                                .host_id(new_user_id.clone())
                                .visibility(state::Visibility::Private)
                                .build()
                                .unwrap();
                            let user = state::UserBuilder::default()
                                .id(new_user_id.clone())
                                .display_name(params.display_name)
                                .room_id(new_room_id.clone())
                                .build()
                                .unwrap();
                            let mut rooms = game_state.rooms.lock().await;
                            let mut users = game_state.users.lock().await;

                            rooms.push(room.clone());
                            users.push(user.clone());

                            sink.send(
                                ws::Message::Binary(
                                    events::ServerToClientEvents::SendGameState {
                                        room,
                                        user: user.clone(),
                                        users_in_room: vec![user]
                                    }
                                    .try_into()
                                    .unwrap()
                                )
                            ).await?;

                            room_id = new_room_id;
                            user_id = new_user_id;
                        }
                    }

                    // We wrap in Arc so many can use this sink.
                    let sink = std::sync::Arc::new(rocket::futures::lock::Mutex::new(sink));
                    let reader = reader::create_websocket_reader(
                        room_id.clone(),
                        user_id.clone(),
                        game_state,
                        stream,
                        server_messages,
                        ticker_msg
                    );
                    let writer = writer::create_websocket_writer(
                        room_id.clone(),
                        user_id.clone(),
                        game_state,
                        sink.clone(),
                        server_messages
                    );

                    tokio::select! {
                        _ = reader => {}
                        _ = writer => {}
                    }

                    println!("WebSocket connection closed");

                    Ok(())
                }
                Err(_) => {
                    sink.send(
                        ws::Message::Binary(events::ServerToClientEvents::ConnectError {
                            message: "Display name is required and must be between 3 and 20 characters long".to_string(),
                        }
                        .try_into()
                        .unwrap()),
                    )
                    .await?;
                    sink.close().await?;

                    return Ok(());
                }
            }
        })
    })
}
