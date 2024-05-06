use crate::{model, utils};
use rocket::{
    futures::{SinkExt, StreamExt},
    tokio,
};

pub mod reader;
pub mod writer;

#[derive(rocket::FromForm)]
pub struct WsEndpointParams {
    #[field(validate = len(3..20))]
    #[field(name = "displayName")]
    pub display_name: String,
    #[field(name = "roomId")]
    pub room_id: String,
}

#[rocket::get("/?<params..>")]
pub async fn ws_endpoint<'st>(
    game_state: &'st rocket::State<model::GameState>,
    server_messages: &'st rocket::State<
        tokio::sync::broadcast::Sender<model::WebSocketMessage>,
    >,
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
                    if params.room_id.is_empty() {
                        let mut rooms = game_state.rooms.lock().await;
                        let mut users = game_state.users.lock().await;

                        if let Some(available_room) = utils::realtime::find_available_public_room(&mut rooms) {
                            let new_user_id = utils::gen_random_id();
                            let user = model::UserBuilder::default()
                                .id(new_user_id.clone())
                                .display_name(params.display_name)
                                .room_id(available_room.id.clone())
                                .build()
                                .unwrap();

                            users.push(user.clone());
                            available_room.amount_of_users += 1;

                            sink.send(ws::Message::Binary(model::ServerToClientEvents::SendRoomInfo { room: available_room.clone() }.try_into().unwrap())).await?;
                            sink.send(ws::Message::Binary(model::ServerToClientEvents::SendUserInfo { user: user.clone() }.try_into().unwrap())).await?;
                            sink.send(ws::Message::Binary(model::ServerToClientEvents::SendUsersInRoomInfo { users: utils::realtime::get_users_in_room(&users, &available_room.id) }.try_into().unwrap())).await?;

                            let message = model::WebSocketMessageBuilder::default()
                                .r#type(model::WebSocketMessageType::Broadcast { sender_id: new_user_id.clone() })
                                .room_id(available_room.id.clone())
                                .message(ws::Message::Binary(model::ServerToClientEvents::UserJoined { user }.try_into().unwrap()))
                                .build()
                                .unwrap();
                            let _ = message.send(server_messages);

                            room_id = available_room.id.clone();
                            user_id = new_user_id;
                        } else {
                            let new_room_id = utils::gen_random_id();
                            let new_user_id = utils::gen_random_id();
                            let room = model::RoomBuilder::default()
                                .id(new_room_id.clone())
                                .host_id(new_user_id.clone())
                                .build()
                                .unwrap();
                            let user = model::UserBuilder::default()
                                .id(new_user_id.clone())
                                .display_name(params.display_name)
                                .room_id(new_room_id.clone())
                                .build()
                                .unwrap();

                            rooms.push(room.clone());
                            users.push(user.clone());

                            sink.send(ws::Message::Binary(model::ServerToClientEvents::SendRoomInfo { room: room.clone() }.try_into().unwrap())).await?;
                            sink.send(ws::Message::Binary(model::ServerToClientEvents::SendUserInfo { user: user.clone() }.try_into().unwrap())).await?;
                            sink.send(ws::Message::Binary(model::ServerToClientEvents::SendUsersInRoomInfo { users: vec![user.clone()] }.try_into().unwrap())).await?;

                            let message = model::WebSocketMessageBuilder::default()
                                .r#type(model::WebSocketMessageType::Broadcast { sender_id: new_user_id.clone() })
                                .room_id(new_room_id.clone())
                                .message(ws::Message::Binary(model::ServerToClientEvents::UserJoined { user }.try_into().unwrap()))
                                .build()
                                .unwrap();
                            let _ = message.send(server_messages);

                            room_id = new_room_id;
                            user_id = new_user_id;
                        };
                    } else {
                        let mut rooms = game_state.rooms.lock().await;
                        let Some(room) = utils::realtime::find_room_by_id(
                            &mut rooms,
                            &params.room_id,
                        ) else {
                            sink.send(
                                ws::Message::Binary(model::ServerToClientEvents::ConnectError {
                                    message: "Room not found".to_string(),
                                }
                                .try_into()
                                .unwrap()),
                            )
                            .await?;
                            sink.close().await?;
                            return Ok(());
                        };

                        if room.state != model::RoomState::Waiting {
                            sink.send(
                                ws::Message::Binary(model::ServerToClientEvents::ConnectError {
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
                                ws::Message::Binary(model::ServerToClientEvents::ConnectError {
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
                        let user = model::UserBuilder::default()
                            .id(new_user_id.clone())
                            .display_name(params.display_name)
                            .room_id(room.id.clone())
                            .build()
                            .unwrap();
                        let mut users = game_state.users.lock().await;

                        users.push(user.clone());
                        room.amount_of_users += 1;

                        sink.send(ws::Message::Binary(model::ServerToClientEvents::SendRoomInfo { room: room.clone() }.try_into().unwrap())).await?;
                        sink.send(ws::Message::Binary(model::ServerToClientEvents::SendUserInfo { user: user.clone() }.try_into().unwrap())).await?;
                        sink.send(ws::Message::Binary(model::ServerToClientEvents::SendUsersInRoomInfo { users: utils::realtime::get_users_in_room(&users, &room.id) }.try_into().unwrap())).await?;

                        let message = model::WebSocketMessageBuilder::default()
                            .r#type(model::WebSocketMessageType::Broadcast { sender_id: new_user_id.clone() })
                            .room_id(room.id.clone())
                            .message(ws::Message::Binary(model::ServerToClientEvents::UserJoined { user }.try_into().unwrap()))
                            .build()
                            .unwrap();
                        let _ = message.send(server_messages);

                        room_id = room.id.clone();
                        user_id = new_user_id;
                    }

                    // We wrap in Arc so both reader and writer can use this.
                    let sink = std::sync::Arc::new(rocket::futures::lock::Mutex::new(sink));
                    let reader = reader::create_websocket_reader(room_id.clone(), user_id.clone(), stream, sink.clone(), server_messages);

                    tokio::select! {
                        _ = reader => {}
                    }

                    Ok(())
                }
                Err(_) => {
                    sink.send(
                        ws::Message::Binary(model::ServerToClientEvents::ConnectError {
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
