use crate::discord::{
    reply_with, validate_discord_signature, DiscordEvent,
    EventType, InteractionResponseType,
};
use aws_lambda_events::encodings::Body;
use ed25519_dalek::PublicKey;
use http::StatusCode;
use lazy_static::lazy_static;
use netlify_lambda_http::{Request, Response};
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

// panics
pub fn reply(content: String) -> Response<Body> {
    Response::builder()
        .body(Body::Text(
            serde_json::to_string(&reply_with(
                InteractionResponseType::ChannelMessageWithSource,
                content,
            ))
            .expect("expected valid json response"),
        ))
        .expect("body to have not failed")
}

pub fn pong() -> Response<Body> {
    Response::builder()
        .body(Body::Text(
            serde_json::to_string(&reply_with(
                InteractionResponseType::Pong,
                "pong".to_string(),
            ))
            .expect("expected valid json response"),
        ))
        .expect("body to have not failed")
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
    Fut: std::future::Future<Output = Response<Body>>,
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
                    EventType::Ping => pong(),
                    EventType::ApplicationCommand => {
                        handle(discord_event).await
                    }
                    _ => panic!("unhandled"),
                }
            }
            Err(e) => {
                println!("{:?}", e);
                reply("failed to parse".to_string())
            }
        };
        Ok(response)
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
