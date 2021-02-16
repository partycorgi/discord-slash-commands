use crate::discord::{
    reply_with, validate_discord_signature, DiscordEvent,
    EventType, InteractionResponse,
    InteractionResponseType,
};
use aws_lambda_events::encodings::Body;
use ed25519_dalek::PublicKey;
use http::{header::CONTENT_TYPE, StatusCode};
use lamedh_http::{IntoResponse, Request, Response};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::env;

lazy_static! {
    static ref PUB_KEY: PublicKey = PublicKey::from_bytes(
        &hex::decode(
            env::var("DISCORD_PUBLIC_KEY").unwrap()
        )
        .unwrap()
    )
    .unwrap();
}

impl IntoResponse for InteractionResponse {
    fn into_response(self) -> Response<Body> {
        Response::builder()
            .header(CONTENT_TYPE, "application/json")
            .body(
                serde_json::to_string(&self)
                    .expect("unable to serialize serde_json::Value")
                    .into(),
            )
            .expect("unable to build http::Response")
    }
}

// panics
pub fn reply(content: &str) -> InteractionResponse {
    reply_with(
        InteractionResponseType::ChannelMessageWithSource,
        content.to_string(),
    )
}

pub fn pong() -> InteractionResponse {
    reply_with(
        InteractionResponseType::Pong,
        "pong".to_string(),
    )
}

pub async fn handle_slash_command<
    'a,
    Error,
    UserEvent: Deserialize<'a>,
    Handler,
    Fut,
>(
    event: &'a Request,
    handle: Handler,
) -> Result<Response<Body>, Error>
where
    Handler: Fn(DiscordEvent<UserEvent>) -> Fut,
    Fut: std::future::Future<Output = InteractionResponse>,
{
    if validate_discord_signature(
        event.headers(),
        event.body(),
        &PUB_KEY,
    ) {
        let parsed: Result<DiscordEvent<UserEvent>, _> =
            serde_json::from_slice(event.body());
        let response = match parsed {
            Ok(discord_event) => {
                match discord_event.event_type {
                    EventType::Ping => {
                        pong().into_response()
                    }
                    EventType::ApplicationCommand => {
                        handle(discord_event)
                            .await
                            .into_response()
                    }
                    _ => panic!("unhandled"),
                }
            }
            Err(e) => {
                println!("{:?}", e);
                reply("failed to parse").into_response()
            }
        };
        Ok(response.into_response())
    } else {
        Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::Text(
                json!({
                    "error": "failed to verify the thing!"
                })
                .to_string(),
            ))
            .map_err(|_e| panic!("whatever"))
    }
}
