use aws_lambda_events::encodings::Body;
use discord_interactions::{
    handle_slash_command, reply, DiscordEvent,
};

use lazy_static::lazy_static;
use netlify_lambda_http::{
    lambda::{lambda, Context},
    IntoResponse, Request, Response,
};
use serde::{Deserialize, Serialize};
use std::env;

type Error =
    Box<dyn std::error::Error + Send + Sync + 'static>;

const DISCORD_API: &str = "https://discord.com/api/v8";
const USER_AGENT: &str = concat!(
	"DiscordBot (https://github.com/partycorgi/discord-slash-commands, ",
	env!("CARGO_PKG_VERSION"),
	")"
);
const SAFELIST_ROLES_TO_ASSUME: [&str; 1] =
    ["646518404030922772"];

lazy_static! {
    static ref DISCORD_BOT_TOKEN: String =
        env::var("DISCORD_BOT_TOKEN")
            .expect("Expected a DISCORD_BOT_TOKEN");
}

#[lambda(http)]
#[tokio::main]
async fn main(
    event: Request,
    _: Context,
) -> Result<impl IntoResponse, Error> {
    handle_slash_command(&event, handle_event).await
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

async fn handle_event(
    event: DiscordEvent<Event>,
) -> Response<Body> {
    println!("{:?}", event);
    match event.data {
        Some(Event::Role { id, options }) => {
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
                    // if the user is allowed to self-assign
                    if SAFELIST_ROLES_TO_ASSUME
                        .contains(&value.as_str())
                    {
                        let client = reqwest::Client::new();
                        let res = client
                        .put(&format!("{}/guilds/{}/members/{}/roles/{}", DISCORD_API, event.guild_id.expect("expected a guild_id to exist in event"), user.id, value))
                        .header("User-Agent", USER_AGENT)
                        .header("Authorization", format!("Bot {}", DISCORD_BOT_TOKEN.clone()))
                        .send()
                        .await;

                        match res {
                            Err(e) => reply(format!(
                                "failed to set role for {}",
                                user.username
                            )),
                            Ok(response) => {
                                if response
                                    .status()
                                    .is_success()
                                {
                                    reply(format!("{} has accepted a role", user.username).to_string())
                                } else {
                                    reply(format!("Failed with status code {}", response.status()))
                                }
                            }
                        }
                    } else {
                        // otherwise send role request to mod channel?
                        reply("Role wasn't in the safelist for self-assignment.".to_string())
                    }
                }
                (None, _) => {
                    reply("must request a role".to_string())
                }
                (_, None) => reply(
                    "no member to apply role to"
                        .to_string(),
                ),
            }
        }
        Some(Event::Unknown) => {
            reply("unknown_command".to_string())
        }
        None => reply("no data for command".to_string()),
    }
}
