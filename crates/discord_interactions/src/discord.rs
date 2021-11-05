use aws_lambda_events::encodings::Body;
use chrono::{DateTime, Utc};
use ed25519_dalek::{PublicKey, Signature, Verifier};
use http::HeaderMap;
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_repr::{Deserialize_repr, Serialize_repr};

/// Verify the discord signature using your
/// application's publickey,
/// the `X-Signature-Ed25519` and
/// `X-Signature-Timestamp` headers,
/// and the request body.
///
/// This is required because discord will send you
/// a ping when you set up your webhook URL, as
/// well as random invalid input periodically that
/// has to be rejected.
pub fn validate_discord_signature(
    headers: &HeaderMap,
    body: &Option<String>,
    pub_key: &PublicKey,
) -> bool {
    let mut sig_arr: [u8; 64] = [0; 64];
    let sig_ed25519 =
        headers.get("X-Signature-Ed25519").map(|sig| {
            let s = hex::decode(sig)
                .expect("decoded signature");
            for (i, byte) in s.into_iter().enumerate() {
                sig_arr[i] = byte;
            }
            Signature::new(sig_arr)
        });
    let sig_timestamp =
        headers.get("X-Signature-Timestamp");

    if let (Some(body), Some(timestamp), Some(sig_bytes)) =
        (body, sig_timestamp, sig_ed25519)
    {
        let content = timestamp
            .as_bytes()
            .into_iter()
            .chain(body.as_bytes().into_iter())
            .cloned()
            .collect::<Vec<u8>>();

        pub_key
            .verify(&content.as_slice(), &sig_bytes)
            .is_ok()
    } else {
        false
    }
}

// snowflake
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub struct DiscordEvent<T> {
    #[serde(rename = "type")]
    pub event_type: EventType,
    pub data: Option<T>,
    pub guild_id: Option<Snowflake>,
    pub channel_id: Option<Snowflake>,
    pub member: Option<GuildMember>,
    pub token: String,
    pub version: usize,
}

#[derive(
    Serialize_repr, Deserialize_repr, PartialEq, Eq, Debug,
)]
#[repr(u8)]
pub enum EventType {
    Ping = 1,
    ApplicationCommand = 2,
}
type Snowflake = String;
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GuildMember {
    pub deaf: bool,
    // pub guild_id: Snowflake,
    pub joined_at: Option<DateTime<Utc>>,
    pub mute: bool,
    pub nick: Option<String>,
    pub roles: Vec<String>,
    /// Attached User struct.
    pub user: User,
}
/// Information about a user.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct User {
    pub id: Snowflake,
    pub avatar: Option<String>,
    #[serde(default)]
    pub bot: bool,
    pub discriminator: String,
    pub username: String,
}
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct ApplicationCommandInteractionData(
    serde_json::Value,
);

// struct ApplicationCommandInteractionDataOption
// {     name: String,
//     value: Option,
//     options: Option,
// }

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct InteractionResponse {
    #[serde(rename = "type")]
    ir_type: InteractionResponseType,
    data: Option<InteractionApplicationCommandCallbackData>,
}
#[derive(
    Serialize_repr, Deserialize_repr, PartialEq, Eq, Debug,
)]
#[repr(u8)]
pub enum InteractionResponseType {
    Pong = 1,
    Acknowledge = 2,
    ChannelMessage = 3,
    ChannelMessageWithSource = 4,
    ACKWithSource = 5,
}

pub fn reply_with(
    ir_type: InteractionResponseType,
    content: String,
) -> InteractionResponse {
    InteractionResponse {
        ir_type,
        data: Some(
            InteractionApplicationCommandCallbackData {
                tts: false,
                content,
            },
        ),
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct InteractionApplicationCommandCallbackData {
    tts: bool,
    content: String,
    /* embeds: Option<Vec<Embed>>,                   //
     * TODO, allowed_mentions:
     * Option<Vec<AllowedMention>>, // TODO */
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn it_works() {
        let parsed: DiscordEvent<serde_json::Value> = serde_json::from_str(
            r#"{
            "type": 2,
            "token": "A_UNIQUE_TOKEN",
            "member": {
                "user": {
                    "id": 53908232506183680,
                    "username": "Mason",
                    "avatar": "a_d5efa99b3eeaa7dd43acca82f5692432",
                    "discriminator": "1337",
                    "public_flags": 131141
                },
                "roles": ["539082325061836999"],
                "premium_since": null,
                "permissions": "2147483647",
                "pending": false,
                "nick": null,
                "mute": false,
                "joined_at": "2017-03-13T19:19:14.040000+00:00",
                "is_pending": false,
                "deaf": false
            },
            "id": "786008729715212338",
            "guild_id": "290926798626357999",
            "data": {
                "options": [{
                    "name": "cardname",
                    "value": "The Gitrog Monster"
                }],
                "name": "cardsearch",
                "id": "771825006014889984"
            },
            "channel_id": "645027906669510667"
        }"#,
        )
        .unwrap();
        assert_eq!(
            match parsed {
                DiscordEvent { .. } => true,
                _ => false,
            },
            true
        );
    }
}
