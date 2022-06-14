#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
#[macro_use] extern crate serde;

mod discord;
mod ical;

const API_BASE: &'static str = "https://discord.com/api/v10";

#[derive(Debug, Deserialize)]
struct Config {
    discord_token: String,
    root_url: String
}

#[get("/guilds/<guild_id>/calendar.ics")]
async fn calendar(client: &rocket::State<reqwest::Client>, config: &rocket::State<Config>, guild_id: String)
    -> Result<(rocket::http::ContentType, String), rocket::http::Status> {
    let discord_guild: discord::Guild = client.get(format!("{}/guilds/{guild_id}", API_BASE, guild_id = guild_id))
        .send().await.map_err(|_| rocket::http::Status::InternalServerError)?
        .error_for_status().map_err(|e| match e.status() {
            Some(reqwest::StatusCode::FORBIDDEN) | Some(reqwest::StatusCode::NOT_FOUND) => rocket::http::Status::NotFound,
            _ => rocket::http::Status::InternalServerError
        })?
        .json().await.map_err(|_| rocket::http::Status::InternalServerError)?;

    let discord_events: Vec<discord::GuildEvent> = client.get(format!("{}/guilds/{guild_id}/scheduled-events", API_BASE, guild_id = guild_id))
        .send().await.map_err(|_| rocket::http::Status::InternalServerError)?
        .error_for_status().map_err(|e| match e.status() {
            Some(reqwest::StatusCode::FORBIDDEN) | Some(reqwest::StatusCode::NOT_FOUND) => rocket::http::Status::NotFound,
            _ => rocket::http::Status::InternalServerError
        })?
        .json().await.map_err(|_| rocket::http::Status::InternalServerError)?;

    let mut events = vec![];

    for event in discord_events {
        let discord_channel = match event.channel_id {
            Some(i) => {
                match client.get(format!("{}/channels/{channel_id}", API_BASE, channel_id = i))
                    .send().await.ok()
                    .and_then(|r| r.error_for_status().ok())
                    .map(|r| r.json::<discord::Channel>()) {
                    Some(c) => c.await.ok(),
                    None => None,
                }
            },
            None => None
        };

        events.push(ical::Event {
            uid: format!("{}@e.discord-events.magicalcodewit.ch", event.id),
            timestamp: event.id.timestamp(),
            start: event.scheduled_start_time,
            end: event.scheduled_end_time,
            created: Some(event.id.timestamp()),
            summary: Some(event.name),
            description: event.description.or(discord_channel.as_ref().and_then(|c| c.topic.clone())),
            location: match event.entity_type {
                discord::GuildEventEntityType::External => event.entity_metadata.and_then(|m| m.location),
                discord::GuildEventEntityType::Voice | discord::GuildEventEntityType::Stage => discord_channel.and_then(|c| c.name).map(|c| format!("#{}", c))
            },
            organiser: Some(ical::Organiser {
                address: format!("https//discord.com/channels/{}", event.guild_id),
                common_name: event.creator.as_ref().map(|c| format!("{}#{}", c.username, c.discriminator)),
                sent_by: event.creator.as_ref().map(|c|format!("https//discord.com/channels/@me/{}", c.id))
            }),
            status: Some(match event.status {
                discord::GuildEventStatus::Cancelled => "CANCELLED",
                _ => "CONFIRMED"
            }.to_string()),
            images: event.image.map(|i| vec![
                ical::Image::Url(format!(
                    "https://cdn.discordapp.com/guild-events/{scheduled_event_id}/{scheduled_event_cover_image}.png",
                    scheduled_event_id = event.id, scheduled_event_cover_image = i
                ))
            ]).unwrap_or_else(|| vec![])
        })
    }

    let calendar = ical::Calendar {
        product: format!("Discord Events Export {}", env!("CARGO_PKG_VERSION")),
        version: "2.0".to_string(),
        scale: Some("GREGORIAN".to_string()),
        method: None,
        name: Some(format!("{} Events", discord_guild.name)),
        description: discord_guild.description,
        uid: Some(format!("{}@c.discord-events.magicalcodewit.ch", discord_guild.id)),
        url: Some(format!("{}{}", config.root_url, uri!(calendar(guild_id = guild_id)))),
        events,
    };

    Ok((rocket::http::ContentType::Calendar, calendar.to_string()))
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![calendar])
        .attach(rocket::fairing::AdHoc::config::<Config>())
        .attach(rocket::fairing::AdHoc::try_on_ignite("HTTP client", |rocket| async {
            let config = match rocket.state::<Config>() {
                Some(c) => c,
                None => {
                    println!("Unable to access config");
                    return Err(rocket)
                }
            };
            let mut headers = reqwest::header::HeaderMap::new();
            let mut auth_value = match reqwest::header::HeaderValue::from_str(&format!("Bot {}", config.discord_token)) {
                Ok(a) => a,
                Err(e) => {
                    println!("Unable to make auth header: {}", e);
                    return Err(rocket)
                }
            };
            auth_value.set_sensitive(true);
            headers.insert("Authorization", auth_value);
            let client = match reqwest::Client::builder()
                .https_only(true)
                .user_agent(format!("DiscordEventExport ({})", env!("CARGO_PKG_VERSION")))
                .default_headers(headers)
                .build() {
                Ok(a) => a,
                Err(e) => {
                    println!("Unable to build request client: {}", e);
                    return Err(rocket)
                }
            };
            Ok(rocket.manage(client))
        }))
}
