use crate::{model, utils};

impl TryFrom<model::ServerToClientEvents> for Vec<u8> {
    type Error = Box<dyn std::error::Error>;

    fn try_from(
        value: model::ServerToClientEvents,
    ) -> Result<Vec<u8>, <Vec<u8> as TryFrom<model::ServerToClientEvents>>::Error> {
        let binary_event_type = server_to_client_event_to_u8(&value);

        match value {
            model::ServerToClientEvents::Error { message }
            | model::ServerToClientEvents::ConnectError { message } => {
                let message_as_bytes = message.as_bytes();
                let message_length = message_as_bytes.len();
                let message_length_into_vec_of_u8 =
                    utils::turn_usize_to_vec_of_u8(message_as_bytes.len());
                let message_length_vec_of_u8_length = message_length_into_vec_of_u8.len();

                let mut binary_message: Vec<u8> = Vec::with_capacity(
                    2 + 1 + message_length_vec_of_u8_length + message_length,
                );

                binary_message.push(utils::consts::BINARY_PROTOCOL_VERSION);
                binary_message.push(binary_event_type);
                binary_message.push(message_length_vec_of_u8_length as u8);
                binary_message.extend(message_length_into_vec_of_u8);
                binary_message.extend(message_as_bytes);

                Ok(binary_message)
            }
            model::ServerToClientEvents::UserJoined { user } => {
                let user_as_bytes = serde_json::to_vec(&user)?;
                let user_length = user_as_bytes.len();
                let user_length_into_vec_of_u8 =
                    utils::turn_usize_to_vec_of_u8(user_as_bytes.len());
                let user_length_vec_of_u8_length = user_length_into_vec_of_u8.len();

                let mut binary_message: Vec<u8> = Vec::with_capacity(
                    2 + 1 + user_length_vec_of_u8_length + user_length,
                );

                binary_message.push(utils::consts::BINARY_PROTOCOL_VERSION);
                binary_message.push(binary_event_type);
                binary_message.push(user_length_vec_of_u8_length as u8);
                binary_message.extend(user_length_into_vec_of_u8);
                binary_message.extend(user_as_bytes);

                Ok(binary_message)
            }
            model::ServerToClientEvents::UserLeft { user_id } => {
                let user_id_as_bytes = user_id.as_bytes();
                let user_id_length = user_id_as_bytes.len();
                let user_id_length_into_vec_of_u8 =
                    utils::turn_usize_to_vec_of_u8(user_id_as_bytes.len());
                let user_id_length_vec_of_u8_length = user_id_length_into_vec_of_u8.len();

                let mut binary_message: Vec<u8> = Vec::with_capacity(
                    2 + 1 + user_id_length_vec_of_u8_length + user_id_length,
                );

                binary_message.push(utils::consts::BINARY_PROTOCOL_VERSION);
                binary_message.push(binary_event_type);
                binary_message.push(user_id_length_vec_of_u8_length as u8);
                binary_message.extend(user_id_length_into_vec_of_u8);
                binary_message.extend(user_id_as_bytes);

                Ok(binary_message)
            }
            model::ServerToClientEvents::StartGame => {
                let mut binary_message: Vec<u8> = Vec::with_capacity(2 + 1);

                binary_message.push(utils::consts::BINARY_PROTOCOL_VERSION);
                binary_message.push(binary_event_type);

                Ok(binary_message)
            }
            model::ServerToClientEvents::EndGame => {
                let mut binary_message: Vec<u8> = Vec::with_capacity(2 + 1);

                binary_message.push(utils::consts::BINARY_PROTOCOL_VERSION);
                binary_message.push(binary_event_type);

                Ok(binary_message)
            }
            model::ServerToClientEvents::ResetRoom => {
                let mut binary_message: Vec<u8> = Vec::with_capacity(2 + 1);

                binary_message.push(utils::consts::BINARY_PROTOCOL_VERSION);
                binary_message.push(binary_event_type);

                Ok(binary_message)
            }
            model::ServerToClientEvents::NewRound {
                round,
                user_id_to_draw,
            } => {
                let user_id_to_draw_as_bytes = user_id_to_draw.as_bytes();
                let user_id_to_draw_length = user_id_to_draw_as_bytes.len();
                let user_id_to_draw_length_into_vec_of_u8 =
                    utils::turn_usize_to_vec_of_u8(user_id_to_draw_as_bytes.len());
                let user_id_to_draw_length_vec_of_u8_length =
                    user_id_to_draw_length_into_vec_of_u8.len();

                let mut binary_message: Vec<u8> = Vec::with_capacity(
                    2 + 3
                        + 1
                        + user_id_to_draw_length_vec_of_u8_length
                        + user_id_to_draw_length,
                );

                binary_message.push(utils::consts::BINARY_PROTOCOL_VERSION);
                binary_message.push(binary_event_type);
                binary_message.push(1);
                binary_message.push(1);
                binary_message.push(round);
                binary_message.push(user_id_to_draw_length_vec_of_u8_length as u8);
                binary_message.extend(user_id_to_draw_length_into_vec_of_u8);
                binary_message.extend(user_id_to_draw_as_bytes);

                Ok(binary_message)
            }
            model::ServerToClientEvents::NewUserToDraw { user_id } => {
                let user_id_as_bytes = user_id.as_bytes();
                let user_id_length = user_id_as_bytes.len();
                let user_id_length_into_vec_of_u8 =
                    utils::turn_usize_to_vec_of_u8(user_id_as_bytes.len());
                let user_id_length_vec_of_u8_length = user_id_length_into_vec_of_u8.len();

                let mut binary_message: Vec<u8> = Vec::with_capacity(
                    2 + 1 + user_id_length_vec_of_u8_length + user_id_length,
                );

                binary_message.push(utils::consts::BINARY_PROTOCOL_VERSION);
                binary_message.push(binary_event_type);
                binary_message.push(user_id_length_vec_of_u8_length as u8);
                binary_message.extend(user_id_length_into_vec_of_u8);
                binary_message.extend(user_id_as_bytes);

                Ok(binary_message)
            }
            model::ServerToClientEvents::NewHost { user_id } => {
                let user_id_as_bytes = user_id.as_bytes();
                let user_id_length = user_id_as_bytes.len();
                let user_id_length_into_vec_of_u8 =
                    utils::turn_usize_to_vec_of_u8(user_id_as_bytes.len());
                let user_id_length_vec_of_u8_length = user_id_length_into_vec_of_u8.len();

                let mut binary_message: Vec<u8> = Vec::with_capacity(
                    2 + 1 + user_id_length_vec_of_u8_length + user_id_length,
                );

                binary_message.push(utils::consts::BINARY_PROTOCOL_VERSION);
                binary_message.push(binary_event_type);
                binary_message.push(user_id_length_vec_of_u8_length as u8);
                binary_message.extend(user_id_length_into_vec_of_u8);
                binary_message.extend(user_id_as_bytes);

                Ok(binary_message)
            }
            model::ServerToClientEvents::NewWord { word } => {
                let word = match word {
                    model::WordToDraw::Word(word) => word,
                    model::WordToDraw::ObfuscatedWord(word) => {
                        utils::obfuscate_word(&word)
                    }
                };

                let word_as_bytes = word.as_bytes();
                let word_length = word_as_bytes.len();
                let word_length_into_vec_of_u8 =
                    utils::turn_usize_to_vec_of_u8(word_as_bytes.len());
                let word_length_vec_of_u8_length = word_length_into_vec_of_u8.len();

                let mut binary_message: Vec<u8> = Vec::with_capacity(
                    2 + 1 + word_length_vec_of_u8_length + word_length,
                );

                binary_message.push(utils::consts::BINARY_PROTOCOL_VERSION);
                binary_message.push(binary_event_type);
                binary_message.push(word_length_vec_of_u8_length as u8);
                binary_message.extend(word_length_into_vec_of_u8);
                binary_message.extend(word_as_bytes);

                Ok(binary_message)
            }
            model::ServerToClientEvents::PointerDown => {
                let mut binary_message: Vec<u8> = Vec::with_capacity(2 + 1);

                binary_message.push(utils::consts::BINARY_PROTOCOL_VERSION);
                binary_message.push(binary_event_type);

                Ok(binary_message)
            }
            model::ServerToClientEvents::PointerMove { direction } => {
                let direction = serde_json::to_vec(&direction)?;
                let direction_length_into_vec_of_u8 =
                    utils::turn_usize_to_vec_of_u8(direction.len());
                let direction_length_vec_of_u8_length =
                    direction_length_into_vec_of_u8.len();
                let mut binary_message: Vec<u8> = Vec::with_capacity(
                    2 + direction_length_vec_of_u8_length + direction.len(),
                );

                binary_message.push(utils::consts::BINARY_PROTOCOL_VERSION);
                binary_message.push(binary_event_type);
                binary_message.push(direction_length_vec_of_u8_length as u8);

                Ok(binary_message)
            }
            model::ServerToClientEvents::PointerUp => {
                let mut binary_message: Vec<u8> = Vec::with_capacity(2 + 1);

                binary_message.push(utils::consts::BINARY_PROTOCOL_VERSION);
                binary_message.push(binary_event_type);

                Ok(binary_message)
            }
            model::ServerToClientEvents::ChangeColor { color } => {
                let color_as_bytes = color.as_bytes();
                let color_length = color_as_bytes.len();
                let color_length_into_vec_of_u8 =
                    utils::turn_usize_to_vec_of_u8(color_as_bytes.len());
                let color_length_vec_of_u8_length = color_length_into_vec_of_u8.len();

                let mut binary_message: Vec<u8> = Vec::with_capacity(
                    2 + 1 + color_length_vec_of_u8_length + color_length,
                );

                binary_message.push(utils::consts::BINARY_PROTOCOL_VERSION);
                binary_message.push(binary_event_type);
                binary_message.push(color_length_vec_of_u8_length as u8);
                binary_message.extend(color_length_into_vec_of_u8);
                binary_message.extend(color_as_bytes);

                Ok(binary_message)
            }
            model::ServerToClientEvents::Tick { time_left } => {
                let mut binary_message: Vec<u8> = Vec::with_capacity(2 + 2);

                binary_message.push(utils::consts::BINARY_PROTOCOL_VERSION);
                binary_message.push(binary_event_type);
                binary_message.push(time_left);

                Ok(binary_message)
            }
            model::ServerToClientEvents::SendUserInfo { user } => {
                let user_as_bytes = serde_json::to_vec(&user)?;
                let user_length = user_as_bytes.len();
                let user_length_into_vec_of_u8 =
                    utils::turn_usize_to_vec_of_u8(user_as_bytes.len());
                let user_length_vec_of_u8_length = user_length_into_vec_of_u8.len();
                let mut binary_message: Vec<u8> = Vec::with_capacity(
                    2 + 1 + user_length_vec_of_u8_length + user_length,
                );

                binary_message.push(utils::consts::BINARY_PROTOCOL_VERSION);
                binary_message.push(binary_event_type);
                binary_message.push(user_length_vec_of_u8_length as u8);
                binary_message.extend(user_length_into_vec_of_u8);
                binary_message.extend(user_as_bytes);

                Ok(binary_message)
            }
            model::ServerToClientEvents::SendRoomInfo { room } => {
                let room_as_bytes = serde_json::to_vec(&room)?;
                let room_length = room_as_bytes.len();
                let room_length_into_vec_of_u8 =
                    utils::turn_usize_to_vec_of_u8(room_as_bytes.len());
                let room_length_vec_of_u8_length = room_length_into_vec_of_u8.len();
                let mut binary_message: Vec<u8> = Vec::with_capacity(
                    2 + 1 + room_length_vec_of_u8_length + room_length,
                );

                binary_message.push(utils::consts::BINARY_PROTOCOL_VERSION);
                binary_message.push(binary_event_type);
                binary_message.push(room_length_vec_of_u8_length as u8);
                binary_message.extend(room_length_into_vec_of_u8);
                binary_message.extend(room_as_bytes);

                Ok(binary_message)
            }
            model::ServerToClientEvents::SendUsersInRoomInfo { users } => {
                let users_as_bytes = serde_json::to_vec(&users)?;
                let users_length = users_as_bytes.len();
                let users_length_into_vec_of_u8 =
                    utils::turn_usize_to_vec_of_u8(users_as_bytes.len());
                let users_length_vec_of_u8_length = users_length_into_vec_of_u8.len();
                let mut binary_message: Vec<u8> = Vec::with_capacity(
                    2 + 1 + users_length_vec_of_u8_length + users_length,
                );

                binary_message.push(utils::consts::BINARY_PROTOCOL_VERSION);
                binary_message.push(binary_event_type);
                binary_message.push(users_length_vec_of_u8_length as u8);
                binary_message.extend(users_length_into_vec_of_u8);
                binary_message.extend(users_as_bytes);

                Ok(binary_message)
            }
            model::ServerToClientEvents::Message { message } => {
                let message_as_bytes = message.as_bytes();
                let message_length = message_as_bytes.len();
                let message_length_into_vec_of_u8 =
                    utils::turn_usize_to_vec_of_u8(message_as_bytes.len());
                let message_length_vec_of_u8_length = message_length_into_vec_of_u8.len();

                let mut binary_message: Vec<u8> = Vec::with_capacity(
                    2 + 1 + message_length_vec_of_u8_length + message_length,
                );

                binary_message.push(utils::consts::BINARY_PROTOCOL_VERSION);
                binary_message.push(binary_event_type);
                binary_message.push(message_length_vec_of_u8_length as u8);
                binary_message.extend(message_length_into_vec_of_u8);
                binary_message.extend(message_as_bytes);

                Ok(binary_message)
            }
        }
    }
}

impl TryFrom<Vec<u8>> for model::ServerToClientEvents {
    type Error = Box<dyn std::error::Error>;

    fn try_from(
        value: Vec<u8>,
    ) -> Result<Self, <model::ServerToClientEvents as TryFrom<Vec<u8>>>::Error> {
        if value.len() < 2 {
            return Err("Data is too short".into());
        }

        if *value.get(0).unwrap() != utils::consts::BINARY_PROTOCOL_VERSION {
            return Err("Binary protocol version is not correct".into());
        }

        let event_type = *value.get(1).unwrap();

        match event_type {
            0 | 1 => {
                let message_length_vec_of_u8_length =
                    *value.get(2).ok_or("Data is too short")? as usize;
                let message_length_vec_of_u8 = value
                    .get(3..3 + message_length_vec_of_u8_length)
                    .ok_or("Message length is not correct")?
                    .iter()
                    .fold(0, |acc, x| acc + x)
                    as usize;
                let message = String::from_utf8(
                    value
                        .get(
                            3 + message_length_vec_of_u8_length..message_length_vec_of_u8,
                        )
                        .ok_or("Incorrect length provided")?
                        .to_vec(),
                )?;

                if event_type == 0 {
                    Ok(model::ServerToClientEvents::Error { message })
                } else {
                    Ok(model::ServerToClientEvents::ConnectError { message })
                }
            }
            2 => {
                let user_length_vec_of_u8_length =
                    *value.get(2).ok_or("Data is too short")? as usize;
                let user_length_vec_of_u8 = value
                    .get(3..3 + user_length_vec_of_u8_length)
                    .ok_or("User length is not correct")?
                    .iter()
                    .fold(0, |acc, x| acc + x)
                    as usize;
                let user = serde_json::from_slice(
                    value
                        .get(3 + user_length_vec_of_u8_length..user_length_vec_of_u8)
                        .ok_or("Incorrect length provided")?,
                )?;

                Ok(model::ServerToClientEvents::UserJoined { user })
            }
            3 => {
                let user_id_length_vec_of_u8_length =
                    *value.get(2).ok_or("Data is too short")? as usize;
                let user_id_length_vec_of_u8 = value
                    .get(3..3 + user_id_length_vec_of_u8_length)
                    .ok_or("User ID length is not correct")?
                    .iter()
                    .fold(0, |acc, x| acc + x)
                    as usize;
                let user_id = String::from_utf8(
                    value
                        .get(
                            3 + user_id_length_vec_of_u8_length..user_id_length_vec_of_u8,
                        )
                        .ok_or("Incorrect length provided")?
                        .to_vec(),
                )?;

                Ok(model::ServerToClientEvents::UserLeft { user_id })
            }
            4 => Ok(model::ServerToClientEvents::StartGame),
            5 => Ok(model::ServerToClientEvents::EndGame),
            6 => Ok(model::ServerToClientEvents::ResetRoom),
            7 => {
                // 2nd index is how much bytes the round's length indicator is, and
                // 3rd index is the length of the round. But since it's u8, its length is always 1.
                let round = *value.get(4).ok_or("Data is too short")?;
                let user_id_to_draw_length_vec_of_u8_length =
                    *value.get(5).ok_or("Data is too short")? as usize;
                let user_id_to_draw_length_vec_of_u8 = value
                    .get(6..6 + user_id_to_draw_length_vec_of_u8_length)
                    .ok_or("User ID length is not correct")?
                    .iter()
                    .fold(0, |acc, x| acc + x)
                    as usize;
                let user_id_to_draw = String::from_utf8(
                    value
                        .get(
                            5 + user_id_to_draw_length_vec_of_u8_length
                                ..user_id_to_draw_length_vec_of_u8,
                        )
                        .ok_or("Incorrect length provided")?
                        .to_vec(),
                )?;

                Ok(model::ServerToClientEvents::NewRound {
                    round,
                    user_id_to_draw,
                })
            }
            8 => {
                let user_id_length_vec_of_u8_length =
                    *value.get(2).ok_or("Data is too short")? as usize;
                let user_id_length_vec_of_u8 = value
                    .get(3..3 + user_id_length_vec_of_u8_length)
                    .ok_or("User ID length is not correct")?
                    .iter()
                    .fold(0, |acc, x| acc + x)
                    as usize;
                let user_id = String::from_utf8(
                    value
                        .get(
                            3 + user_id_length_vec_of_u8_length..user_id_length_vec_of_u8,
                        )
                        .ok_or("Incorrect length provided")?
                        .to_vec(),
                )?;

                Ok(model::ServerToClientEvents::NewUserToDraw { user_id })
            }
            9 => {
                let user_id_length_vec_of_u8_length =
                    *value.get(2).ok_or("Data is too short")? as usize;
                let user_id_length_vec_of_u8 = value
                    .get(3..3 + user_id_length_vec_of_u8_length)
                    .ok_or("User ID length is not correct")?
                    .iter()
                    .fold(0, |acc, x| acc + x)
                    as usize;
                let user_id = String::from_utf8(
                    value
                        .get(
                            3 + user_id_length_vec_of_u8_length..user_id_length_vec_of_u8,
                        )
                        .ok_or("Incorrect length provided")?
                        .to_vec(),
                )?;

                Ok(model::ServerToClientEvents::NewHost { user_id })
            }
            10 => {
                let word_length_vec_of_u8_length =
                    *value.get(2).ok_or("Data is too short")? as usize;
                let word_length_vec_of_u8 = value
                    .get(3..3 + word_length_vec_of_u8_length)
                    .ok_or("Word length is not correct")?
                    .iter()
                    .fold(0, |acc, x| acc + x)
                    as usize;
                let word = String::from_utf8(
                    value
                        .get(3 + word_length_vec_of_u8_length..word_length_vec_of_u8)
                        .ok_or("Incorrect length provided")?
                        .to_vec(),
                )?;

                Ok(model::ServerToClientEvents::NewWord {
                    word: model::WordToDraw::Word(word),
                })
            }
            11 => Ok(model::ServerToClientEvents::PointerDown),
            12 => {
                let direction_length_vec_of_u8_length =
                    *value.get(2).ok_or("Data is too short")? as usize;
                let direction_length_vec_of_u8 = value
                    .get(3..3 + direction_length_vec_of_u8_length)
                    .ok_or("Direction length is not correct")?
                    .iter()
                    .fold(0, |acc, x| acc + x)
                    as usize;
                let direction = serde_json::from_slice(
                    value
                        .get(
                            3 + direction_length_vec_of_u8_length
                                ..direction_length_vec_of_u8,
                        )
                        .ok_or("Incorrect length provided")?,
                )?;

                Ok(model::ServerToClientEvents::PointerMove { direction })
            }
            13 => Ok(model::ServerToClientEvents::PointerUp),
            14 => {
                let color_length_vec_of_u8_length =
                    *value.get(2).ok_or("Data is too short")? as usize;
                let color_length_vec_of_u8 = value
                    .get(3..3 + color_length_vec_of_u8_length)
                    .ok_or("Color length is not correct")?
                    .iter()
                    .fold(0, |acc, x| acc + x)
                    as usize;
                let color = String::from_utf8(
                    value
                        .get(3 + color_length_vec_of_u8_length..color_length_vec_of_u8)
                        .ok_or("Incorrect length provided")?
                        .to_vec(),
                )?;

                Ok(model::ServerToClientEvents::ChangeColor { color })
            }
            15 => {
                // 2nd index is how mny bytes the time_left's length occupies.
                // 3rd index is the length of the time_left, but since it's u8, its length is always 1.
                let time_left = *value.get(4).ok_or("Data is too short")?;

                Ok(model::ServerToClientEvents::Tick { time_left })
            }
            16 => {
                let user_length_vec_of_u8_length =
                    *value.get(2).ok_or("Data is too short")? as usize;
                let user_length_vec_of_u8 = value
                    .get(3..3 + user_length_vec_of_u8_length)
                    .ok_or("User length is not correct")?
                    .iter()
                    .fold(0, |acc, x| acc + x)
                    as usize;
                let user = serde_json::from_slice(
                    value
                        .get(3 + user_length_vec_of_u8_length..user_length_vec_of_u8)
                        .ok_or("Incorrect length provided")?,
                )?;

                Ok(model::ServerToClientEvents::SendUserInfo { user })
            }
            17 => {
                let room_length_vec_of_u8_length =
                    *value.get(2).ok_or("Data is too short")? as usize;
                let room_length_vec_of_u8 = value
                    .get(3..3 + room_length_vec_of_u8_length)
                    .ok_or("Room length is not correct")?
                    .iter()
                    .fold(0, |acc, x| acc + x)
                    as usize;
                let room = serde_json::from_slice(
                    value
                        .get(3 + room_length_vec_of_u8_length..room_length_vec_of_u8)
                        .ok_or("Incorrect length provided")?,
                )?;

                Ok(model::ServerToClientEvents::SendRoomInfo { room })
            }
            18 => {
                let users_length_vec_of_u8_length =
                    *value.get(2).ok_or("Data is too short")? as usize;
                let users_length_vec_of_u8 = value
                    .get(3..3 + users_length_vec_of_u8_length)
                    .ok_or("Users length is not correct")?
                    .iter()
                    .fold(0, |acc, x| acc + x)
                    as usize;
                let users = serde_json::from_slice(
                    value
                        .get(3 + users_length_vec_of_u8_length..users_length_vec_of_u8)
                        .ok_or("Incorrect length provided")?,
                )?;

                Ok(model::ServerToClientEvents::SendUsersInRoomInfo { users })
            }
            _ => todo!(),
        }
    }
}

impl TryFrom<Vec<u8>> for model::ClientToServerEvents {
    type Error = Box<dyn std::error::Error>;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        if value.len() < 2 {
            return Err("Data is too short".into());
        }

        if *value.get(0).unwrap() != utils::consts::BINARY_PROTOCOL_VERSION {
            return Err("Binary protocol version is not correct".into());
        }

        let event_type = *value.get(1).unwrap();

        match event_type {
            0 => Ok(model::ClientToServerEvents::StartGame),
            1 => Ok(model::ClientToServerEvents::PointerDown),
            2 => {
                let direction_length_vec_of_u8_length =
                    *value.get(2).ok_or("Data is too short")? as usize;
                let direction_length_vec_of_u8 = value
                    .get(3..3 + direction_length_vec_of_u8_length)
                    .ok_or("Direction length is not correct")?
                    .iter()
                    .fold(0, |acc, x| acc + x)
                    as usize;
                let direction = serde_json::from_slice(
                    value
                        .get(
                            3 + direction_length_vec_of_u8_length
                                ..direction_length_vec_of_u8,
                        )
                        .ok_or("Incorrect length provided")?,
                )?;

                Ok(model::ClientToServerEvents::PointerMove { direction })
            }
            3 => Ok(model::ClientToServerEvents::PointerUp),
            4 => {
                let message_length_vec_of_u8_length =
                    *value.get(2).ok_or("Data is too short")? as usize;
                let message_length_vec_of_u8 = value
                    .get(3..3 + message_length_vec_of_u8_length)
                    .ok_or("Message length is not correct")?
                    .iter()
                    .fold(0, |acc, x| acc + x)
                    as usize;
                let message = String::from_utf8(
                    value
                        .get(
                            3 + message_length_vec_of_u8_length..message_length_vec_of_u8,
                        )
                        .ok_or("Incorrect length provided")?
                        .to_vec(),
                )?;

                Ok(model::ClientToServerEvents::Message { message })
            }
            _ => Err("Unknown event type".into()),
        }
    }
}

pub fn server_to_client_event_to_u8(value: &model::ServerToClientEvents) -> u8 {
    match value {
        model::ServerToClientEvents::Error { .. } => 0,
        model::ServerToClientEvents::ConnectError { .. } => 1,
        model::ServerToClientEvents::UserJoined { .. } => 2,
        model::ServerToClientEvents::UserLeft { .. } => 3,
        model::ServerToClientEvents::StartGame => 4,
        model::ServerToClientEvents::EndGame => 5,
        model::ServerToClientEvents::ResetRoom => 6,
        model::ServerToClientEvents::NewRound { .. } => 7,
        model::ServerToClientEvents::NewUserToDraw { .. } => 8,
        model::ServerToClientEvents::NewHost { .. } => 9,
        model::ServerToClientEvents::NewWord { .. } => 10,
        model::ServerToClientEvents::PointerDown => 11,
        model::ServerToClientEvents::PointerMove { .. } => 12,
        model::ServerToClientEvents::PointerUp => 13,
        model::ServerToClientEvents::ChangeColor { .. } => 14,
        model::ServerToClientEvents::Tick { .. } => 15,
        model::ServerToClientEvents::SendUserInfo { .. } => 16,
        model::ServerToClientEvents::SendRoomInfo { .. } => 17,
        model::ServerToClientEvents::SendUsersInRoomInfo { .. } => 18,
        model::ServerToClientEvents::Message { .. } => 19,
    }
}

impl model::GameState {
    pub fn new() -> Self {
        Self {
            rooms: std::sync::Arc::new(rocket::futures::lock::Mutex::new(Vec::new())),
            users: std::sync::Arc::new(rocket::futures::lock::Mutex::new(Vec::new())),
        }
    }
}
