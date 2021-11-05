use aws_lambda_events::event::apigw::{
    ApiGatewayProxyRequest, ApiGatewayProxyResponse,
};
use discord_interactions::{
    handle_slash_command, reply, DiscordEvent,
    InteractionResponse,
};

use lambda_runtime::{handler_fn, Context, Error};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, env};
// use tracing::{info, instrument};

const DISCORD_API: &str = "https://discord.com/api/v8";
const USER_AGENT: &str = concat!(
	"DiscordBot (https://github.com/partycorgi/discord-slash-commands, ",
	env!("CARGO_PKG_VERSION"),
	")"
);

lazy_static! {
    static ref DISCORD_BOT_TOKEN: String =
        env::var("DISCORD_BOT_TOKEN")
            .expect("Expected a DISCORD_BOT_TOKEN");
    static ref ROLE_REVERSE_MAP: HashMap<&'static str, &'static str> = {
        let mut map = HashMap::new();
        map.insert("646518404030922772", "Streamer");
        map.insert("906236413694079006", "Fortnite");
        map
    };
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();
    let processor = handler_fn(handler);
    lambda_runtime::run(processor).await?;
    Ok(())
}

#[tracing::instrument]
async fn handler(
    event: ApiGatewayProxyRequest,
    _: Context,
) -> Result<ApiGatewayProxyResponse, Error> {
    let res =
        handle_slash_command(&event, handle_event).await;
    res
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(tag = "name", rename_all = "lowercase")]
enum Event {
    Role {
        id: String,
        options: Vec<RoleOption>,
    },
    #[serde(other)]
    Unknown,
}
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(tag = "name")]
enum RoleOption {
    #[serde(rename = "i-want-to-be-a")]
    IWantToBeA { value: String },
}

#[tracing::instrument]
async fn handle_event(
    event: DiscordEvent<Event>,
) -> InteractionResponse {
    match event.data {
        Some(Event::Role { id: _, options }) => {
            let role_requested =
                options.iter().find(|value| match value {
                    RoleOption::IWantToBeA { .. } => true,
                });
            match (role_requested, event.member) {
                (
                    Some(RoleOption::IWantToBeA { value }),
                    Some(
                        discord_interactions::GuildMember {
                            user,
                            ..
                        },
                    ),
                ) => {
                    let client = reqwest::Client::new();
                    let res = client
                        .put(&format!("{}/guilds/{}/members/{}/roles/{}", DISCORD_API, event.guild_id.expect("expected a guild_id to exist in event"), user.id, value))
                        .header("User-Agent", USER_AGENT)
                        .header("Authorization", format!("Bot {}", DISCORD_BOT_TOKEN.clone()))
                        .send()
                        .await;

                    match res {
                        Err(_e) => reply(&format!(
                            "failed to set role for {}",
                            user.username
                        )),
                        Ok(response) => {
                            if response
                                .status()
                                .is_success()
                            {
                                match   ROLE_REVERSE_MAP.get(value.as_str())                              {
                                       Some(name) =>                                 reply(&format!("{} has accepted the {} role", user.username, name)),
None =>                                 reply(&format!("{} has accepted a role", user.username))

}
                            } else {
                                reply(&format!("Failed with status code {}", response.status()))
                            }
                        }
                    }
                }
                (None, _) => reply("must request a role"),
                (_, None) => {
                    reply("no member to apply role to")
                }
            }
        }
        Some(Event::Unknown) => reply("unknown_command"),
        None => reply("no data for command"),
    }
}
