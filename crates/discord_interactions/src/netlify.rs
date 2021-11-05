use crate::discord::{
    reply_with, validate_discord_signature, DiscordEvent,
    EventType, InteractionResponse,
    InteractionResponseType,
};
use aws_lambda_events::{
    encodings::Body,
    event::apigw::{
        ApiGatewayProxyRequest, ApiGatewayProxyResponse,
    },
};
use ed25519_dalek::PublicKey;
use http::{header::CONTENT_TYPE, HeaderMap, HeaderValue};
use lazy_static::lazy_static;
use serde::Deserialize;
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

trait IntoResponse {
    fn into_response(self) -> ApiGatewayProxyResponse;
}
impl IntoResponse for InteractionResponse {
    fn into_response(self) -> ApiGatewayProxyResponse {
        let mut headers = HeaderMap::new();
        headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );
        ApiGatewayProxyResponse {
            status_code: 200,
            headers,
            multi_value_headers: HeaderMap::new(),
            body: Some(Body::Text(
                serde_json::to_string(&self)
                    .unwrap()
                    .to_string(),
            )),
            is_base64_encoded: Some(false),
        }
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
    event: &'a ApiGatewayProxyRequest,
    handle: Handler,
) -> Result<ApiGatewayProxyResponse, Error>
where
    Handler: Fn(DiscordEvent<UserEvent>) -> Fut,
    Fut: std::future::Future<Output = InteractionResponse>,
{
    if validate_discord_signature(
        &event.headers,
        &event.body,
        &PUB_KEY,
    ) {
        let parsed: Result<DiscordEvent<UserEvent>, _> =
            serde_json::from_str(
                event.body.as_ref().expect("to exist"),
            );
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
        Ok(response)
    } else {
        Ok(ApiGatewayProxyResponse {
            status_code: 404,
            headers: HeaderMap::new(),
            multi_value_headers: HeaderMap::new(),
            body: Some(Body::Text(
                json!({
                    "error": "failed to verify the thing!"
                })
                .to_string(),
            )),
            is_base64_encoded: Some(false),
        })
    }
}
