use crate::{state, utils, vec_with_slices};

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
    PointerLeave,
    ChangeColor { color: String },
    Message { message: String },
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
            5 => Ok(Self::PointerLeave),
            6 => {
                let length_of_color_length_indicator =
                    *value.get(2).ok_or("Data is too short")?;
                let length_of_color_start_position = 3;
                let length_of_color_end_position = length_of_color_start_position
                    + length_of_color_length_indicator as usize;
                let length_of_color: usize = value
                    .get(length_of_color_start_position..length_of_color_end_position)
                    .ok_or("Data is too short")?
                    .iter()
                    .fold(0, |acc, x| acc + *x as usize);
                let color_end_position = length_of_color_end_position + length_of_color;
                let color = String::from_utf8(
                    value
                        .get(length_of_color_end_position..color_end_position)
                        .ok_or("Data is too short")?
                        .to_vec(),
                )?;

                Ok(Self::ChangeColor { color })
            }
            7 => {
                let length_of_message_length_indicator =
                    *value.get(2).ok_or("Data is too short")?;
                let length_of_message_start_position = 3;
                let length_of_message_end_position = length_of_message_start_position
                    + length_of_message_length_indicator as usize;
                let length_of_message: usize = value
                    .get(length_of_message_start_position..length_of_message_end_position)
                    .ok_or("Data is too short")?
                    .iter()
                    .fold(0, |acc, x| acc + *x as usize);
                let message_end_position =
                    length_of_message_end_position + length_of_message;
                let message = String::from_utf8(
                    value
                        .get(length_of_message_end_position..message_end_position)
                        .ok_or("Data is too short")?
                        .to_vec(),
                )?;

                Ok(Self::Message { message })
            }
            _ => Err("Invalid event type".into()),
        }
    }
}

#[derive(Clone)]
pub enum ServerToClientEvents {
    Error {
        message: String,
    },
    ConnectError {
        message: String,
    },
    UserJoined {
        user: state::User,
    },
    UserLeft {
        user_id: String,
    },
    StartGame,
    PickAWord {
        words_to_pick: [String; 3],
    },
    EndGame,
    ResetRoom,
    NewTurn {
        user_id_to_draw: String,
    },
    NewWord {
        word: String,
    },
    NewHost {
        user_id: String,
    },
    NewRound {
        round: u8,
    },
    PointerDown,
    PointerMove {
        x: f64,
        y: f64,
    },
    PointerUp,
    PointerLeave,
    ChangeColor {
        color: String,
    },
    SendGameState {
        room: state::Room,
        user: state::User,
        users_in_room: Vec<state::User>,
    },
    Message {
        user_id: String,
        message: String,
    },
    AddScore {
        user_id: String,
        score: u16,
    },
    Tick {
        time_left: u8,
    },
    UserGuessed {
        user_id: String,
    },
    SystemMessage {
        message: String,
    },
}

impl TryFrom<ServerToClientEvents> for Vec<u8> {
    type Error = Box<dyn std::error::Error>;

    fn try_from(value: ServerToClientEvents) -> Result<Self, Self::Error> {
        let event_as_borrowed = &value;

        match event_as_borrowed {
            ServerToClientEvents::Error { message }
            | ServerToClientEvents::ConnectError { message } => {
                let message_as_bytes = message.as_bytes();
                let length_of_message =
                    utils::turn_usize_to_vec_of_u8(message_as_bytes.len());
                let length_of_message_length_indicator = length_of_message.len();

                Ok(vec_with_slices![
                    utils::consts::BINARY_PROTOCOL_VERSION,
                    event_as_borrowed.into(),
                    length_of_message_length_indicator.try_into()?;
                    &length_of_message,
                    message_as_bytes
                ])
            }
            ServerToClientEvents::UserJoined { user } => {
                let stringified_user_as_bytes = serde_json::to_vec(&user)?;
                let length_of_stringified_user =
                    utils::turn_usize_to_vec_of_u8(stringified_user_as_bytes.len());
                let length_of_stringified_user_length_indicator =
                    length_of_stringified_user.len();

                Ok(vec_with_slices!(
                    utils::consts::BINARY_PROTOCOL_VERSION,
                    event_as_borrowed.into(),
                    length_of_stringified_user_length_indicator.try_into()?;
                    &length_of_stringified_user,
                    &stringified_user_as_bytes
                ))
            }
            ServerToClientEvents::UserLeft { user_id } => {
                let user_id_as_bytes = user_id.as_bytes();
                let length_of_user_id =
                    utils::turn_usize_to_vec_of_u8(user_id_as_bytes.len());
                let length_of_user_id_length_indicator = length_of_user_id.len();

                Ok(vec_with_slices!(
                    utils::consts::BINARY_PROTOCOL_VERSION,
                    event_as_borrowed.into(),
                    length_of_user_id_length_indicator.try_into()?;
                    &length_of_user_id,
                    user_id_as_bytes
                ))
            }
            ServerToClientEvents::StartGame => Ok(vec![
                utils::consts::BINARY_PROTOCOL_VERSION,
                event_as_borrowed.into(),
            ]),
            ServerToClientEvents::PickAWord { words_to_pick } => {
                let stringified_words_to_pick_as_bytes =
                    serde_json::to_vec(&words_to_pick)?;
                let length_of_stringified_words_to_pick = utils::turn_usize_to_vec_of_u8(
                    stringified_words_to_pick_as_bytes.len(),
                );
                let length_of_stringified_words_to_pick_length_indicator =
                    length_of_stringified_words_to_pick.len();

                Ok(vec_with_slices!(
                    utils::consts::BINARY_PROTOCOL_VERSION,
                    event_as_borrowed.into(),
                    length_of_stringified_words_to_pick_length_indicator.try_into()?;
                    &length_of_stringified_words_to_pick,
                    &stringified_words_to_pick_as_bytes
                ))
            }
            ServerToClientEvents::EndGame => Ok(vec![
                utils::consts::BINARY_PROTOCOL_VERSION,
                event_as_borrowed.into(),
            ]),
            ServerToClientEvents::ResetRoom => Ok(vec![
                utils::consts::BINARY_PROTOCOL_VERSION,
                event_as_borrowed.into(),
            ]),
            ServerToClientEvents::NewTurn { user_id_to_draw } => {
                let user_id_to_draw_as_bytes = user_id_to_draw.as_bytes();
                let length_of_user_id_to_draw =
                    utils::turn_usize_to_vec_of_u8(user_id_to_draw_as_bytes.len());
                let length_of_user_id_to_draw_length_indicator =
                    length_of_user_id_to_draw.len();

                Ok(vec_with_slices!(
                    utils::consts::BINARY_PROTOCOL_VERSION,
                    event_as_borrowed.into(),
                    length_of_user_id_to_draw_length_indicator.try_into()?;
                    &length_of_user_id_to_draw,
                    user_id_to_draw_as_bytes
                ))
            }
            ServerToClientEvents::NewWord { word } => {
                let word_as_bytes = word.as_bytes();
                let length_of_word = utils::turn_usize_to_vec_of_u8(word_as_bytes.len());
                let length_of_word_length_indicator = length_of_word.len();

                Ok(vec_with_slices!(
                    utils::consts::BINARY_PROTOCOL_VERSION,
                    event_as_borrowed.into(),
                    length_of_word_length_indicator.try_into()?;
                    &length_of_word,
                    word_as_bytes
                ))
            }
            ServerToClientEvents::NewRound { round } => {
                let round_as_bytes = round.to_be_bytes();
                let length_of_round =
                    utils::turn_usize_to_vec_of_u8(round_as_bytes.len());
                let length_of_round_length_indicator = length_of_round.len();

                Ok(vec_with_slices!(
                    utils::consts::BINARY_PROTOCOL_VERSION,
                    event_as_borrowed.into(),
                    length_of_round_length_indicator.try_into()?;
                    &length_of_round,
                    &round_as_bytes
                ))
            }
            ServerToClientEvents::NewHost { user_id } => {
                let user_id_as_bytes = user_id.as_bytes();
                let length_of_user_id =
                    utils::turn_usize_to_vec_of_u8(user_id_as_bytes.len());
                let length_of_user_id_length_indicator = length_of_user_id.len();

                Ok(vec_with_slices!(
                    utils::consts::BINARY_PROTOCOL_VERSION,
                    event_as_borrowed.into(),
                    length_of_user_id_length_indicator.try_into()?;
                    &length_of_user_id,
                    user_id_as_bytes
                ))
            }
            ServerToClientEvents::PointerDown => Ok(vec![
                utils::consts::BINARY_PROTOCOL_VERSION,
                event_as_borrowed.into(),
            ]),
            ServerToClientEvents::PointerMove { x, y } => {
                let x_as_bytes = x.to_be_bytes();
                let length_of_x = utils::turn_usize_to_vec_of_u8(x_as_bytes.len());
                let length_of_x_length_indicator = length_of_x.len();

                let y_as_bytes = y.to_be_bytes();
                let length_of_y = utils::turn_usize_to_vec_of_u8(y_as_bytes.len());
                let length_of_y_length_indicator = length_of_y.len();

                Ok(vec_with_slices!(
                    utils::consts::BINARY_PROTOCOL_VERSION,
                    event_as_borrowed.into();
                    vec_with_slices!(
                        length_of_x_length_indicator.try_into()?;
                        &length_of_x,
                        &x_as_bytes
                    ).as_slice(),
                    vec_with_slices!(
                        length_of_y_length_indicator.try_into()?;
                        &length_of_y,
                        &y_as_bytes
                    ).as_slice()
                ))
            }
            ServerToClientEvents::PointerUp | ServerToClientEvents::PointerLeave => {
                Ok(vec![
                    utils::consts::BINARY_PROTOCOL_VERSION,
                    event_as_borrowed.into(),
                ])
            }
            ServerToClientEvents::ChangeColor { color } => {
                let color_as_bytes = color.as_bytes();
                let length_of_color =
                    utils::turn_usize_to_vec_of_u8(color_as_bytes.len());
                let length_of_color_length_indicator = length_of_color.len();

                Ok(vec_with_slices!(
                    utils::consts::BINARY_PROTOCOL_VERSION,
                    event_as_borrowed.into(),
                    length_of_color_length_indicator.try_into()?;
                    &length_of_color,
                    color_as_bytes
                ))
            }
            ServerToClientEvents::SendGameState {
                room,
                user,
                users_in_room,
            } => {
                let room_as_bytes = serde_json::to_vec(room)?;
                let length_of_room = utils::turn_usize_to_vec_of_u8(room_as_bytes.len());
                let length_of_room_length_indicator = length_of_room.len();

                let user_as_bytes = serde_json::to_vec(user)?;
                let length_of_user = utils::turn_usize_to_vec_of_u8(user_as_bytes.len());
                let length_of_user_length_indicator = length_of_user.len();

                let users_in_room_as_bytes = serde_json::to_vec(users_in_room)?;
                let length_of_users_in_room =
                    utils::turn_usize_to_vec_of_u8(users_in_room_as_bytes.len());
                let length_of_users_in_room_length_indicator =
                    length_of_users_in_room.len();

                Ok(vec_with_slices![
                    utils::consts::BINARY_PROTOCOL_VERSION,
                    event_as_borrowed.into();
                    vec_with_slices!(
                        length_of_room_length_indicator.try_into()?;
                        &length_of_room,
                        &room_as_bytes
                    ).as_slice(),
                    vec_with_slices!(
                        length_of_user_length_indicator.try_into()?;
                        &length_of_user,
                        &user_as_bytes
                    ).as_slice(),
                    vec_with_slices!(
                        length_of_users_in_room_length_indicator.try_into()?;
                        &length_of_users_in_room,
                        &users_in_room_as_bytes
                    ).as_slice()
                ])
            }
            ServerToClientEvents::Message { user_id, message } => {
                let user_id_as_bytes = user_id.as_bytes();
                let length_of_user_id =
                    utils::turn_usize_to_vec_of_u8(user_id_as_bytes.len());
                let length_of_user_id_length_indicator = length_of_user_id.len();

                let message_as_bytes = message.as_bytes();
                let length_of_message =
                    utils::turn_usize_to_vec_of_u8(message_as_bytes.len());
                let length_of_message_length_indicator = length_of_message.len();

                Ok(vec_with_slices!(
                    utils::consts::BINARY_PROTOCOL_VERSION,
                    event_as_borrowed.into();
                    vec_with_slices!(
                        length_of_user_id_length_indicator.try_into()?;
                        &length_of_user_id,
                        user_id_as_bytes
                    ).as_slice(),
                    vec_with_slices!(
                        length_of_message_length_indicator.try_into()?;
                        &length_of_message,
                        message_as_bytes
                    ).as_slice()
                ))
            }
            ServerToClientEvents::SystemMessage { message } => {
                let message_as_bytes = message.as_bytes();
                let length_of_message =
                    utils::turn_usize_to_vec_of_u8(message_as_bytes.len());
                let length_of_message_length_indicator = length_of_message.len();

                Ok(vec_with_slices!(
                    utils::consts::BINARY_PROTOCOL_VERSION,
                    event_as_borrowed.into(),
                    length_of_message_length_indicator.try_into()?;
                    &length_of_message,
                    message_as_bytes
                ))
            }
            ServerToClientEvents::AddScore { user_id, score } => {
                let user_id_as_bytes = user_id.as_bytes();
                let length_of_user_id =
                    utils::turn_usize_to_vec_of_u8(user_id_as_bytes.len());
                let length_of_user_id_length_indicator = length_of_user_id.len();

                let score_as_bytes = score.to_be_bytes();
                let score_length = utils::turn_usize_to_vec_of_u8(score_as_bytes.len());
                let length_of_score_length_indicator = score_length.len();

                Ok(vec_with_slices!(
                    utils::consts::BINARY_PROTOCOL_VERSION,
                    event_as_borrowed.into();
                    vec_with_slices!(
                        length_of_user_id_length_indicator.try_into()?;
                        &length_of_user_id,
                        user_id_as_bytes
                    ).as_slice(),
                    vec_with_slices!(
                        length_of_score_length_indicator.try_into()?;
                        &score_length,
                        &score_as_bytes
                    ).as_slice()
                ))
            }
            ServerToClientEvents::Tick { time_left } => Ok(vec![
                utils::consts::BINARY_PROTOCOL_VERSION,
                event_as_borrowed.into(),
                1,
                1,
                time_left.clone(),
            ]),
            ServerToClientEvents::UserGuessed { user_id } => {
                let user_id_as_bytes = user_id.as_bytes();
                let user_id_length =
                    utils::turn_usize_to_vec_of_u8(user_id_as_bytes.len());
                let length_of_user_id_length_indicator = user_id_length.len();

                Ok(vec_with_slices!(
                    utils::consts::BINARY_PROTOCOL_VERSION,
                    event_as_borrowed.into(),
                    length_of_user_id_length_indicator as u8;
                    &user_id_length,
                    user_id_as_bytes
                ))
            }
        }
    }
}

// We borrow since we only need to see what variant is being converted.
impl From<&ServerToClientEvents> for u8 {
    fn from(value: &ServerToClientEvents) -> Self {
        match value {
            ServerToClientEvents::Error { .. } => 0,
            ServerToClientEvents::ConnectError { .. } => 1,
            ServerToClientEvents::UserJoined { .. } => 2,
            ServerToClientEvents::UserLeft { .. } => 3,
            ServerToClientEvents::StartGame => 4,
            ServerToClientEvents::PickAWord { .. } => 5,
            ServerToClientEvents::EndGame => 6,
            ServerToClientEvents::ResetRoom => 7,
            ServerToClientEvents::NewTurn { .. } => 8,
            ServerToClientEvents::NewWord { .. } => 9,
            ServerToClientEvents::NewRound { .. } => 10,
            ServerToClientEvents::NewHost { .. } => 11,
            ServerToClientEvents::PointerDown => 12,
            ServerToClientEvents::PointerMove { .. } => 13,
            ServerToClientEvents::PointerUp => 14,
            ServerToClientEvents::PointerLeave => 15,
            ServerToClientEvents::ChangeColor { .. } => 16,
            ServerToClientEvents::SendGameState { .. } => 17,
            ServerToClientEvents::Message { .. } => 18,
            ServerToClientEvents::AddScore { .. } => 19,
            ServerToClientEvents::Tick { .. } => 20,
            ServerToClientEvents::UserGuessed { .. } => 21,
            ServerToClientEvents::SystemMessage { .. } => 22,
        }
    }
}
