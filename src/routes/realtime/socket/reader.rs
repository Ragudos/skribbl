use rocket::futures::StreamExt;
use rocket::tokio;

use crate::{model, utils};

pub async fn create_websocket_reader(
    room_id: String,
    user_id: String,
    mut stream: rocket::futures::stream::SplitStream<ws::stream::DuplexStream>,
    sink: std::sync::Arc<
        rocket::futures::lock::Mutex<
            rocket::futures::stream::SplitSink<ws::stream::DuplexStream, ws::Message>,
        >,
    >,
    server_messages: &rocket::State<
        tokio::sync::broadcast::Sender<model::WebSocketMessage>,
    >,
) -> Result<(), Box<dyn std::error::Error>> {
    while let Some(message) = stream.next().await {
        let message = message?;

        match message {
            ws::Message::Binary(data) => {
                let event_type: model::ClientToServerEvents = data.clone().try_into()?;

                match event_type {
                    model::ClientToServerEvents::StartGame => todo!(),
                    model::ClientToServerEvents::Message { message } => todo!(),
                    _ => {
                        let _ = server_messages.send(
                            model::WebSocketMessageBuilder::default()
                                .r#type(model::WebSocketMessageType::Broadcast {
                                    sender_id: user_id.clone(),
                                })
                                .room_id(room_id.clone())
                                .message(ws::Message::Binary(data))
                                .build()?,
                        );
                    }
                }
            }
            ws::Message::Close(close_frame) => {}
            _ => {}
        }
    }

    Ok(())
}
