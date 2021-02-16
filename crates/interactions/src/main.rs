use aws_lambda_events::encodings::Body;
use discord_interactions::{
    handle_slash_command, reply, DiscordEvent,
    InteractionResponse,
};
use lamedh_http::{
    lambda::{lambda, run, Context},
    IntoResponse, Request, Response,
};
use lazy_static::lazy_static;
// use tracing_subscriber::prelude::;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, env};
use tracing::{info, instrument};
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::prelude::*;
use tracing_subscriber::registry;

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
    static ref ROLE_REVERSE_MAP: HashMap<&'static str, &'static str> = {
        let mut map = HashMap::new();
        map.insert("646518404030922772", "Streamer");
        map
    };
}

// #[lambda(http)]
#[tokio::main]
async fn main() -> Result<(), Error> {
    setup_tracing();
    run(lamedh_http::handler(handler)).await?;
    Ok(())
}

async fn handler(
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

#[tracing::instrument]
async fn handle_event(
    event: DiscordEvent<Event>,
) -> InteractionResponse {
    info!(test = "thing");
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
                    let client = reqwest::Client::new();
                    let res = client
                        .put(&format!("{}/guilds/{}/members/{}/roles/{}", DISCORD_API, event.guild_id.expect("expected a guild_id to exist in event"), user.id, value))
                        .header("User-Agent", USER_AGENT)
                        .header("Authorization", format!("Bot {}", DISCORD_BOT_TOKEN.clone()))
                        .send()
                        .await;

                    match res {
                        Err(e) => reply(&format!(
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

fn setup_tracing() {
    let honeycomb_key = String::from("");
    let honeycomb_config = libhoney::Config {
        options: libhoney::client::Options {
            api_key: honeycomb_key,
            dataset: "pcn".to_string(),
            ..libhoney::client::Options::default()
        },
        transmission_options:
            libhoney::transmission::Options::default(),
    };

    let telemetry_layer =
        tracing_honeycomb::new_honeycomb_telemetry_layer(
            "discord_slash_commands",
            honeycomb_config,
        );

    // NOTE: the underlying subscriber MUST be the Registry subscriber
    let subscriber = registry::Registry::default() // provide underlying span data store
        .with(LevelFilter::INFO) // filter out low-level debug tracing (eg tokio executor)
        .with(tracing_subscriber::fmt::Layer::default()) // log to stdout
        .with(telemetry_layer); // publish to honeycomb backend

    // &subscriber.init();

    tracing::subscriber::set_global_default(subscriber)
        .expect("setting global default failed");
}
