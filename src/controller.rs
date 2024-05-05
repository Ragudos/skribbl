use crate::{model, utils};

impl<'r> rocket::response::Responder<'r, 'static> for model::HandshakePayload {
    fn respond_to(
        self,
        _request: &'r rocket::Request<'_>,
    ) -> rocket::response::Result<'static> {
        let stringified_payload = serde_json::to_string(&self).map_err(|err| {
            rocket::error!("Failed to serialize HandshakePayload: {:?}", err);

            rocket::http::Status::InternalServerError
        })?;

        rocket::response::Response::build()
            .header(rocket::http::ContentType::JSON)
            .sized_body(
                stringified_payload.len(),
                std::io::Cursor::new(stringified_payload),
            )
            .ok()
    }
}

impl TryFrom<model::ServerToClientEvents> for ws::Message {
    type Error = Box<dyn std::error::Error>;

    fn try_from(
        value: model::ServerToClientEvents,
    ) -> Result<ws::Message, <ws::Message as TryFrom<model::ServerToClientEvents>>::Error>
    {
        match value {
            model::ServerToClientEvents::Error { message } => {
                let message_as_bytes = message.as_bytes();
                let message_length = message_as_bytes.len();
                let message_length_into_vec_of_u8 =
                    utils::turn_usize_to_vec_of_u8(message_as_bytes.len());
                let message_length_vec_of_u8_length = message_length_into_vec_of_u8.len();

                let mut binary_message: Vec<u8> = Vec::with_capacity(
                    2 + 1 + message_length_vec_of_u8_length + message_length,
                );

                binary_message.push(utils::consts::BINARY_PROTOCOL_VERSION);
                binary_message.push(value.try_into()?);
                binary_message.push(message_length_vec_of_u8_length as u8);
                binary_message.extend(message_length_into_vec_of_u8);
                binary_message.extend(message_as_bytes);

                Ok(ws::Message::Binary(binary_message))
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
                binary_message.push(value.try_into()?);
                binary_message.push(user_length_vec_of_u8_length as u8);
                binary_message.extend(user_length_into_vec_of_u8);
                binary_message.extend(user_as_bytes);

                Ok(ws::Message::Binary(binary_message))
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
                binary_message.push(value.try_into()?);
                binary_message.push(user_id_length_vec_of_u8_length as u8);
                binary_message.extend(user_id_length_into_vec_of_u8);
                binary_message.extend(user_id_as_bytes);

                Ok(ws::Message::Binary(binary_message))
            }
            model::ServerToClientEvents::StartGame => {
                let mut binary_message: Vec<u8> = Vec::with_capacity(2 + 1);

                binary_message.push(utils::consts::BINARY_PROTOCOL_VERSION);
                binary_message.push(value.try_into()?);

                Ok(ws::Message::Binary(binary_message))
            }
            model::ServerToClientEvents::EndGame => {
                let mut binary_message: Vec<u8> = Vec::with_capacity(2 + 1);

                binary_message.push(utils::consts::BINARY_PROTOCOL_VERSION);
                binary_message.push(value.try_into()?);

                Ok(ws::Message::Binary(binary_message))
            }
            model::ServerToClientEvents::ResetRoom => {
                let mut binary_message: Vec<u8> = Vec::with_capacity(2 + 1);

                binary_message.push(utils::consts::BINARY_PROTOCOL_VERSION);
                binary_message.push(value.try_into()?);

                Ok(ws::Message::Binary(binary_message))
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
                binary_message.push(value.try_into()?);
                binary_message.push(1);
                binary_message.push(1);
                binary_message.push(round);
                binary_message.push(user_id_to_draw_length_vec_of_u8_length as u8);
                binary_message.extend(user_id_to_draw_length_into_vec_of_u8);
                binary_message.extend(user_id_to_draw_as_bytes);

                Ok(ws::Message::Binary(binary_message))
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
                binary_message.push(value.try_into()?);
                binary_message.push(user_id_length_vec_of_u8_length as u8);
                binary_message.extend(user_id_length_into_vec_of_u8);
                binary_message.extend(user_id_as_bytes);

                Ok(ws::Message::Binary(binary_message))
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
                binary_message.push(value.try_into()?);
                binary_message.push(user_id_length_vec_of_u8_length as u8);
                binary_message.extend(user_id_length_into_vec_of_u8);
                binary_message.extend(user_id_as_bytes);

                Ok(ws::Message::Binary(binary_message))
            }
            model::ServerToClientEvents::NewWord { word } => {
                let word = match word {
                    model::WordToDraw::Word(word) => word,
                    model::WordToDraw::ObfuscatedWord(word) => {
                        utils::obfuscate_word(word)
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
                binary_message.push(value.try_into()?);
                binary_message.push(word_length_vec_of_u8_length as u8);
                binary_message.extend(word_length_into_vec_of_u8);
                binary_message.extend(word_as_bytes);

                Ok(ws::Message::Binary(binary_message))
            }
            model::ServerToClientEvents::PointerDown => {
                let mut binary_message: Vec<u8> = Vec::with_capacity(2 + 1);

                binary_message.push(utils::consts::BINARY_PROTOCOL_VERSION);
                binary_message.push(value.try_into()?);

                Ok(ws::Message::Binary(binary_message))
            }
            model::ServerToClientEvents::PointerMove => {
                let mut binary_message: Vec<u8> = Vec::with_capacity(2 + 1);

                binary_message.push(utils::consts::BINARY_PROTOCOL_VERSION);
                binary_message.push(value.try_into()?);

                Ok(ws::Message::Binary(binary_message))
            }
            model::ServerToClientEvents::PointerUp => {
                let mut binary_message: Vec<u8> = Vec::with_capacity(2 + 1);

                binary_message.push(utils::consts::BINARY_PROTOCOL_VERSION);
                binary_message.push(value.try_into()?);

                Ok(ws::Message::Binary(binary_message))
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
                binary_message.push(value.try_into()?);
                binary_message.push(color_length_vec_of_u8_length as u8);
                binary_message.extend(color_length_into_vec_of_u8);
                binary_message.extend(color_as_bytes);

                Ok(ws::Message::Binary(binary_message))
            }
            model::ServerToClientEvents::Tick { time_left } => {
                let mut binary_message: Vec<u8> = Vec::with_capacity(2 + 2);

                binary_message.push(utils::consts::BINARY_PROTOCOL_VERSION);
                binary_message.push(value.try_into()?);
                binary_message.push(time_left);

                Ok(ws::Message::Binary(binary_message))
            }
        }
    }
}

impl TryFrom<model::ServerToClientEvents> for u8 {
    type Error = &'static str;

    fn try_from(
        value: model::ServerToClientEvents,
    ) -> Result<Self, <u8 as TryFrom<model::ServerToClientEvents>>::Error> {
        match value {
            model::ServerToClientEvents::Error { .. } => Ok(0),
            model::ServerToClientEvents::UserJoined { .. } => Ok(1),
            model::ServerToClientEvents::UserLeft { .. } => Ok(2),
            model::ServerToClientEvents::StartGame => Ok(3),
            model::ServerToClientEvents::EndGame => Ok(4),
            model::ServerToClientEvents::ResetRoom => Ok(5),
            model::ServerToClientEvents::NewRound { .. } => Ok(6),
            model::ServerToClientEvents::NewUserToDraw { .. } => Ok(7),
            model::ServerToClientEvents::NewHost { .. } => Ok(8),
            model::ServerToClientEvents::NewWord { .. } => Ok(9),
            model::ServerToClientEvents::PointerDown => Ok(10),
            model::ServerToClientEvents::PointerMove => Ok(11),
            model::ServerToClientEvents::PointerUp => Ok(12),
            model::ServerToClientEvents::ChangeColor { .. } => Ok(13),
            model::ServerToClientEvents::Tick { .. } => Ok(14),
            _ => Err("Unknown ServerToClientEvents variant"),
        }
    }
}
