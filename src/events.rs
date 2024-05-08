use crate::{state, utils};

#[derive(Clone)]
pub enum WebSocketMessageType {
    /// A message that is sent to everyone in a room.
    Everyone,
    /// A message that is sent to everyone in a room except the sender that matches `sender_id`.
    Broadcast { sender_id: String },
    /// A message that is sent to a specific user in a room.
    User { receiver_id: String },
}

#[derive(derive_builder::Builder, Clone)]
pub struct WebSocketMessage {
    pub r#type: WebSocketMessageType,
    pub room_id: String,
    pub message: ws::Message,
}

impl WebSocketMessage {
    pub fn send(
        self,
        server_messages: &rocket::tokio::sync::broadcast::Sender<WebSocketMessage>,
    ) -> Result<(), rocket::tokio::sync::broadcast::error::SendError<WebSocketMessage>>
    {
        server_messages.send(self)?;

        Ok(())
    }
}

#[derive(Clone)]
pub enum ClientToServerEvents {
    StartGame,
    PickAWord { word: String },
    PointerDown,
    PointerMove { x: f64, y: f64 },
    PointerUp,
}

impl TryFrom<&Vec<u8>> for ClientToServerEvents {
    type Error = Box<dyn std::error::Error>;

    fn try_from(
        value: &Vec<u8>,
    ) -> Result<Self, <ClientToServerEvents as TryFrom<&Vec<u8>>>::Error> {
        if value.len() < 2 {
            return Err("Invalid message length".into());
        }

        if *value.get(0).unwrap() != utils::consts::BINARY_PROTOCOL_VERSION {
            return Err("Version mismatch".into());
        }

        match *value.get(1).unwrap() {
            0 => Ok(Self::StartGame),
            1 => {
                let length_of_word_length_indicator =
                    *value.get(2).ok_or("Data is too short")?;
                let length_of_word_start_position = 3;
                let length_of_word_end_position = length_of_word_start_position
                    + length_of_word_length_indicator as usize;
                let length_of_word: usize = value
                    .get(length_of_word_start_position..length_of_word_end_position)
                    .ok_or("Data is too short")?
                    .iter()
                    .fold(0, |acc, x| acc + *x as usize);
                let word_end_position = length_of_word_end_position + length_of_word;
                let word = String::from_utf8(
                    value
                        .get(length_of_word_end_position..word_end_position)
                        .ok_or("Data is too short")?
                        .to_vec(),
                )?;

                Ok(Self::PickAWord { word })
            }
            2 => Ok(Self::PointerDown),
            3 => {
                let length_of_x_length_indicator =
                    *value.get(2).ok_or("Data is too short")?;
                let length_of_x_end_position = 3 + length_of_x_length_indicator as usize;
                let length_of_x: usize = value
                    .get(3..length_of_x_end_position)
                    .ok_or("Data is too short")?
                    .iter()
                    .fold(0, |acc, x| acc + *x as usize);
                let x_end_position = length_of_x_end_position + length_of_x;
                let x = f64::from_be_bytes(
                    value
                        .get(length_of_x_end_position..x_end_position)
                        .ok_or("Data is too short")?
                        .try_into()
                        .ok()
                        .ok_or("Data is invalid")?,
                );

                let length_of_y_length_indicator = *value
                    .get(x_end_position)
                    .ok_or("Data is too short")?;
                let length_of_y_start_position = x_end_position + 1;
                let length_of_y_end_position =
                    length_of_y_start_position + length_of_y_length_indicator as usize;
                let length_of_y: usize = value
                    .get(length_of_y_start_position..length_of_y_end_position)
                    .ok_or("Data is too short")?
                    .iter()
                    .fold(0, |acc, x| acc + *x as usize);
                let y_end_position = length_of_y_end_position + length_of_y;
                let y = f64::from_be_bytes(
                    value
                        .get(length_of_y_end_position..y_end_position)
                        .ok_or("Data is too short")?
                        .try_into()
                        .ok()
                        .ok_or("Data is invalid")?,
                );

                Ok(Self::PointerMove { x, y })
            }
            4 => Ok(Self::PointerUp),
            _ => Err("Invalid event type".into()),
        }
    }
}

#[derive(Clone)]
pub enum ServerToClientEvents {
    Error { message: String },
    ConnectError { message: String },
    UserJoined { user: state::User },
    UserLeft { user_id: String },
    StartGame,
    PickAWord { words_to_pick: [&'static str; 3] },
    EndGame,
    ResetRoom,
    NewTurn { user_id_to_draw: String },
    NewWord { word: String },
    NewHost { user_id: String },
    NewRound { round: u8 },
    PointerDown,
    PointerMove { x: f64, y: f64 },
    PointerUp,
    ChangeColor { color: String },
    SendUserInfo { user: state::User },
    SendRoomInfo { room: state::Room },
    SendUsersInRoomInfo { users: Vec<state::User> },
    Message { message: String },
}

impl TryFrom<ServerToClientEvents> for Vec<u8> {
    type Error = Box<dyn std::error::Error>;

    fn try_from(value: ServerToClientEvents) -> Result<Self, Self::Error> {
        let borrowed_value = &value;

        match borrowed_value {
            ServerToClientEvents::Error { message }
            | ServerToClientEvents::ConnectError { message } => {
                let message_as_bytes = message.as_bytes();
                let length_of_message =
                    utils::turn_usize_to_vec_of_u8(message_as_bytes.len());
                let length_of_message_length_indicator = length_of_message.len();
                let mut message = Vec::with_capacity(
                    2 + 1 + length_of_message_length_indicator + message_as_bytes.len(),
                );

                message.push(utils::consts::BINARY_PROTOCOL_VERSION);
                message.push(borrowed_value.try_into().unwrap());
                message.push(length_of_message_length_indicator as u8);
                message.extend_from_slice(&length_of_message);
                message.extend_from_slice(message_as_bytes);

                Ok(message)
            }
            ServerToClientEvents::UserJoined { user } => {
                let stringified_user = serde_json::to_string(&user)?;
                let stringified_user_as_bytes = stringified_user.as_bytes();
                let length_of_stringified_user =
                    utils::turn_usize_to_vec_of_u8(stringified_user_as_bytes.len());
                let length_of_stringified_user_length_indicator =
                    length_of_stringified_user.len();

                let mut message = Vec::with_capacity(
                    2 + 1
                        + length_of_stringified_user_length_indicator
                        + stringified_user_as_bytes.len(),
                );

                message.push(utils::consts::BINARY_PROTOCOL_VERSION);
                message.push(borrowed_value.try_into().unwrap());
                message.push(length_of_stringified_user_length_indicator as u8);
                message.extend_from_slice(&length_of_stringified_user);
                message.extend_from_slice(stringified_user_as_bytes);

                Ok(message)
            }
            ServerToClientEvents::UserLeft { user_id } => {
                let user_id_as_bytes = user_id.as_bytes();
                let length_of_user_id =
                    utils::turn_usize_to_vec_of_u8(user_id_as_bytes.len());
                let length_of_user_id_length_indicator = length_of_user_id.len();
                let mut message = Vec::with_capacity(
                    2 + 1 + length_of_user_id_length_indicator + user_id_as_bytes.len(),
                );

                message.push(utils::consts::BINARY_PROTOCOL_VERSION);
                message.push(borrowed_value.try_into().unwrap());
                message.push(length_of_user_id_length_indicator as u8);
                message.extend_from_slice(&length_of_user_id);
                message.extend_from_slice(user_id_as_bytes);

                Ok(message)
            }
            ServerToClientEvents::StartGame => {
                let message = vec![
                    utils::consts::BINARY_PROTOCOL_VERSION,
                    borrowed_value.try_into()?,
                ];
                Ok(message)
            }
            ServerToClientEvents::PickAWord { words_to_pick } => {
                let stringified_words_to_pick = serde_json::to_string(&words_to_pick)?;
                let stringified_words_to_pick_as_bytes =
                    stringified_words_to_pick.as_bytes();
                let length_of_stringified_words_to_pick = utils::turn_usize_to_vec_of_u8(
                    stringified_words_to_pick_as_bytes.len(),
                );
                let length_of_stringified_words_to_pick_length_indicator =
                    length_of_stringified_words_to_pick.len();

                let mut message = Vec::with_capacity(
                    2 + 1
                        + length_of_stringified_words_to_pick_length_indicator
                        + stringified_words_to_pick_as_bytes.len(),
                );

                message.push(utils::consts::BINARY_PROTOCOL_VERSION);
                message.push(borrowed_value.try_into()?);
                message.push(length_of_stringified_words_to_pick_length_indicator as u8);
                message.extend_from_slice(&length_of_stringified_words_to_pick);
                message.extend_from_slice(stringified_words_to_pick_as_bytes);

                Ok(message)
            }
            ServerToClientEvents::EndGame => {
                let message = vec![
                    utils::consts::BINARY_PROTOCOL_VERSION,
                    borrowed_value.try_into()?,
                ];
                Ok(message)
            }
            ServerToClientEvents::ResetRoom => {
                let message = vec![
                    utils::consts::BINARY_PROTOCOL_VERSION,
                    borrowed_value.try_into()?,
                ];
                Ok(message)
            }
            ServerToClientEvents::NewTurn { user_id_to_draw } => {
                let user_id_to_draw_as_bytes = user_id_to_draw.as_bytes();
                let length_of_user_id_to_draw =
                    utils::turn_usize_to_vec_of_u8(user_id_to_draw_as_bytes.len());
                let length_of_user_id_to_draw_length_indicator =
                    length_of_user_id_to_draw.len();
                let mut message = Vec::with_capacity(
                    2 + 1
                        + length_of_user_id_to_draw_length_indicator
                        + user_id_to_draw_as_bytes.len(),
                );

                message.push(utils::consts::BINARY_PROTOCOL_VERSION);
                message.push(borrowed_value.try_into().unwrap());
                message.push(length_of_user_id_to_draw_length_indicator as u8);
                message.extend_from_slice(&length_of_user_id_to_draw);
                message.extend_from_slice(user_id_to_draw_as_bytes);

                Ok(message)
            }
            ServerToClientEvents::NewWord { word } => {
                let word_as_bytes = word.as_bytes();
                let length_of_word = utils::turn_usize_to_vec_of_u8(word_as_bytes.len());
                let length_of_word_length_indicator = length_of_word.len();
                let mut message = Vec::with_capacity(
                    2 + 1 + length_of_word_length_indicator + word_as_bytes.len(),
                );

                message.push(utils::consts::BINARY_PROTOCOL_VERSION);
                message.push(borrowed_value.try_into().unwrap());
                message.push(length_of_word_length_indicator as u8);
                message.extend_from_slice(&length_of_word);
                message.extend_from_slice(word_as_bytes);

                Ok(message)
            }
            ServerToClientEvents::NewHost { user_id } => {
                let user_id_as_bytes = user_id.as_bytes();
                let length_of_user_id =
                    utils::turn_usize_to_vec_of_u8(user_id_as_bytes.len());
                let length_of_user_id_length_indicator = length_of_user_id.len();
                let mut message = Vec::with_capacity(
                    2 + 1 + length_of_user_id_length_indicator + user_id_as_bytes.len(),
                );

                message.push(utils::consts::BINARY_PROTOCOL_VERSION);
                message.push(borrowed_value.try_into().unwrap());
                message.push(length_of_user_id_length_indicator as u8);
                message.extend_from_slice(&length_of_user_id);
                message.extend_from_slice(user_id_as_bytes);

                Ok(message)
            }
            ServerToClientEvents::NewRound { round } => {
                let round_as_bytes = round.to_be_bytes();
                let length_of_round =
                    utils::turn_usize_to_vec_of_u8(round_as_bytes.len());
                let length_of_round_length_indicator = length_of_round.len();
                let mut message = Vec::with_capacity(
                    2 + 1 + length_of_round_length_indicator + round_as_bytes.len(),
                );

                message.push(utils::consts::BINARY_PROTOCOL_VERSION);
                message.push(borrowed_value.try_into().unwrap());
                message.push(length_of_round_length_indicator as u8);
                message.extend_from_slice(&length_of_round);
                message.extend_from_slice(&round_as_bytes);

                Ok(message)
            }
            ServerToClientEvents::PointerDown => {
                let message = vec![
                    utils::consts::BINARY_PROTOCOL_VERSION,
                    borrowed_value.try_into()?,
                ];
                Ok(message)
            }
            ServerToClientEvents::PointerMove { x, y } => {
                let x_as_bytes = x.to_be_bytes();
                let length_of_x = utils::turn_usize_to_vec_of_u8(x_as_bytes.len());
                let length_of_x_length_indicator = length_of_x.len();
                let y_as_bytes = y.to_be_bytes();
                let length_of_y = utils::turn_usize_to_vec_of_u8(y_as_bytes.len());
                let length_of_y_length_indicator = length_of_y.len();
                let mut message = Vec::with_capacity(
                    2 + 1
                        + length_of_x_length_indicator
                        + x_as_bytes.len()
                        + length_of_y_length_indicator
                        + y_as_bytes.len(),
                );

                message.push(utils::consts::BINARY_PROTOCOL_VERSION);
                message.push(borrowed_value.try_into().unwrap());
                message.push(length_of_x_length_indicator as u8);
                message.extend_from_slice(&length_of_x);
                message.extend_from_slice(&x_as_bytes);
                message.push(length_of_y_length_indicator as u8);
                message.extend_from_slice(&length_of_y);
                message.extend_from_slice(&y_as_bytes);

                Ok(message)
            }
            ServerToClientEvents::PointerUp => {
                let message = vec![
                    utils::consts::BINARY_PROTOCOL_VERSION,
                    borrowed_value.try_into()?,
                ];
                Ok(message)
            }
            ServerToClientEvents::ChangeColor { color } => {
                let color_as_bytes = color.as_bytes();
                let length_of_color =
                    utils::turn_usize_to_vec_of_u8(color_as_bytes.len());
                let length_of_color_length_indicator = length_of_color.len();
                let mut message = Vec::with_capacity(
                    2 + 1 + length_of_color_length_indicator + color_as_bytes.len(),
                );

                message.push(utils::consts::BINARY_PROTOCOL_VERSION);
                message.push(borrowed_value.try_into().unwrap());
                message.push(length_of_color_length_indicator as u8);
                message.extend_from_slice(&length_of_color);
                message.extend_from_slice(color_as_bytes);

                Ok(message)
            }
            ServerToClientEvents::SendUserInfo { user } => {
                let stringified_user = serde_json::to_string(&user)?;
                let stringified_user_as_bytes = stringified_user.as_bytes();
                let length_of_stringified_user =
                    utils::turn_usize_to_vec_of_u8(stringified_user_as_bytes.len());
                let length_of_stringified_user_length_indicator =
                    length_of_stringified_user.len();
                let mut message = Vec::with_capacity(
                    2 + 1
                        + length_of_stringified_user_length_indicator
                        + stringified_user_as_bytes.len(),
                );

                message.push(utils::consts::BINARY_PROTOCOL_VERSION);
                message.push(borrowed_value.try_into().unwrap());
                message.push(length_of_stringified_user_length_indicator as u8);
                message.extend_from_slice(&length_of_stringified_user);
                message.extend_from_slice(stringified_user_as_bytes);

                Ok(message)
            }
            ServerToClientEvents::SendRoomInfo { room } => {
                let stringified_room = serde_json::to_string(&room)?;
                let stringified_room_as_bytes = stringified_room.as_bytes();
                let length_of_stringified_room =
                    utils::turn_usize_to_vec_of_u8(stringified_room_as_bytes.len());
                let length_of_stringified_room_length_indicator =
                    length_of_stringified_room.len();
                let mut message = Vec::with_capacity(
                    2 + 1
                        + length_of_stringified_room_length_indicator
                        + stringified_room_as_bytes.len(),
                );

                message.push(utils::consts::BINARY_PROTOCOL_VERSION);
                message.push(borrowed_value.try_into().unwrap());
                message.push(length_of_stringified_room_length_indicator as u8);
                message.extend_from_slice(&length_of_stringified_room);
                message.extend_from_slice(stringified_room_as_bytes);

                Ok(message)
            }
            ServerToClientEvents::SendUsersInRoomInfo { users } => {
                let stringified_users = serde_json::to_string(&users)?;
                let stringified_users_as_bytes = stringified_users.as_bytes();
                let length_of_stringified_users =
                    utils::turn_usize_to_vec_of_u8(stringified_users_as_bytes.len());
                let length_of_stringified_users_length_indicator =
                    length_of_stringified_users.len();
                let mut message = Vec::with_capacity(
                    2 + 1
                        + length_of_stringified_users_length_indicator
                        + stringified_users_as_bytes.len(),
                );

                message.push(utils::consts::BINARY_PROTOCOL_VERSION);
                message.push(borrowed_value.try_into().unwrap());
                message.push(length_of_stringified_users_length_indicator as u8);
                message.extend_from_slice(&length_of_stringified_users);
                message.extend_from_slice(stringified_users_as_bytes);

                Ok(message)
            }
            ServerToClientEvents::Message { message } => {
                let message_as_bytes = message.as_bytes();
                let length_of_message =
                    utils::turn_usize_to_vec_of_u8(message_as_bytes.len());
                let length_of_message_length_indicator = length_of_message.len();
                let mut message = Vec::with_capacity(
                    2 + 1 + length_of_message_length_indicator + message_as_bytes.len(),
                );

                message.push(utils::consts::BINARY_PROTOCOL_VERSION);
                message.push(borrowed_value.try_into().unwrap());
                message.push(length_of_message_length_indicator as u8);
                message.extend_from_slice(&length_of_message);
                message.extend_from_slice(message_as_bytes);

                Ok(message)
            }
        }
    }
}

impl TryFrom<&ServerToClientEvents> for u8 {
    type Error = Box<dyn std::error::Error>;

    fn try_from(
        value: &ServerToClientEvents,
    ) -> Result<Self, <u8 as TryFrom<&ServerToClientEvents>>::Error> {
        match value {
            ServerToClientEvents::Error { .. } => Ok(0),
            ServerToClientEvents::ConnectError { .. } => Ok(1),
            ServerToClientEvents::UserJoined { .. } => Ok(2),
            ServerToClientEvents::UserLeft { .. } => Ok(3),
            ServerToClientEvents::StartGame => Ok(4),
            ServerToClientEvents::PickAWord { .. } => Ok(5),
            ServerToClientEvents::EndGame => Ok(6),
            ServerToClientEvents::ResetRoom => Ok(7),
            ServerToClientEvents::NewTurn { .. } => Ok(8),
            ServerToClientEvents::NewWord { .. } => Ok(9),
            ServerToClientEvents::NewHost { .. } => Ok(10),
            ServerToClientEvents::NewRound { .. } => Ok(11),
            ServerToClientEvents::PointerDown => Ok(12),
            ServerToClientEvents::PointerMove { .. } => Ok(13),
            ServerToClientEvents::PointerUp => Ok(14),
            ServerToClientEvents::ChangeColor { .. } => Ok(15),
            ServerToClientEvents::SendUserInfo { .. } => Ok(16),
            ServerToClientEvents::SendRoomInfo { .. } => Ok(17),
            ServerToClientEvents::SendUsersInRoomInfo { .. } => Ok(18),
            ServerToClientEvents::Message { .. } => Ok(19),
        }
    }
}
