use discord_interactions::{
    reply_with, validate_discord_signature, DiscordEvent, EventType, InteractionResponseType,
};
use ed25519_dalek::PublicKey;
use http::StatusCode;
use lazy_static::lazy_static;
use netlify_lambda_http::{
    lambda::{lambda, Context},
    Body, IntoResponse, Request, Response,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
// use sodiumoxide::crypto::sign;
use std::env;

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

lazy_static! {
    static ref PUB_KEY: PublicKey =
        PublicKey::from_bytes(env::var("DISCORD_PUBLIC_KEY").unwrap().as_bytes()).unwrap();
}

#[lambda(http)]
#[tokio::main]
async fn main(event: Request, _: Context) -> Result<impl IntoResponse, Error> {
    if validate_discord_signature(event.headers(), event.body(), &PUB_KEY) {
        Ok(handle(event))
    } else {
        Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::Text(
                json!({ "error": format!("failed to verify the thing!") }).to_string(),
            ))
            .map_err(|_e| panic!("whatever"))
    }
}
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
struct ReplEvent {
    id: String,
    name: String,
    options: serde_json::Value,
}

fn handle(event: Request) -> Response<Body> {
    let parsed: Result<DiscordEvent<ReplEvent>, _> = serde_json::from_slice(event.body());
    match parsed {
        Ok(event) => match event.event_type {
            EventType::Ping => Response::builder()
                .body(Body::Text(
                    serde_json::to_string(&reply_with(
                        InteractionResponseType::Pong,
                        "pong".to_string(),
                    ))
                    .expect("expected valid json response"),
                ))
                .expect("body to have not failed"),
            EventType::ApplicationCommand => {
                println!("{:?}", event.data);
                Response::builder()
                    .body(Body::Text(
                        serde_json::to_string(&reply_with(
                            InteractionResponseType::ChannelMessageWithSource,
                            "uh, did this work?".to_string(),
                        ))
                        .expect("expected valid json response"),
                    ))
                    .expect("body to have not failed")
            }
            _ => panic!("unhandled"),
        },
        Err(e) => {
            println!("{:?}", e);
            Response::builder()
                .body(Body::Text(
                    serde_json::to_string(&reply_with(
                        InteractionResponseType::ChannelMessageWithSource,
                        "failed to parse".to_string(),
                    ))
                    .expect("expected valid json response"),
                ))
                .expect("body to have not failed")
        }
    };
    Response::builder()
        .body(Body::Text(
            json!({ "message": format!("Hello, {}!", "event.first_name") }).to_string(),
        ))
        .expect("body to have not failed")
}
