use chrono::prelude::*;

pub struct Snowflake(pub u64);

impl Snowflake {
    pub fn timestamp(&self) -> DateTime<Utc> {
        let v = (self.0 >> 22) + 1420070400000;
        Utc.timestamp_millis(v as i64)
    }

    pub fn worker_id(&self) -> u8 {
        ((self.0 & 0x3E0000) >> 17) as u8
    }

    pub fn process_id(&self) -> u8 {
        ((self.0 & 0x1F000) >> 12) as u8
    }

    pub fn counter(&self) -> u16 {
        (self.0 & 0xFFF) as u16
    }
}

impl std::fmt::Debug for Snowflake {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Snowflake({}, ts={}, w={}, p={}, c={})", self.0, self.timestamp(), self.worker_id(), self.process_id(), self.counter())
    }
}

impl std::fmt::Display for Snowflake {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<'de> serde::Deserialize<'de> for Snowflake {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Snowflake, D::Error> {
        let v = String::deserialize(deserializer)?;
        let n = v.parse::<u64>()
            .map_err(|e| serde::de::Error::custom(e.to_string()))?;
        Ok(Snowflake(n))
    }
}

#[derive(Deserialize, Debug)]
pub struct User {
    pub id: Snowflake,
    pub username: String,
    pub discriminator: String,
    pub avatar: Option<String>
}

#[derive(Deserialize, Debug)]
pub struct Guild {
    pub id: Snowflake,
    pub name: String,
    pub icon: Option<String>,
    pub splash: Option<String>,
    pub discovery_splash: Option<String>,
    pub owner_id: Snowflake,
    pub description: Option<String>
}

#[derive(Deserialize, Debug)]
pub struct Channel {
    pub id: Snowflake,
    #[serde(default)]
    pub guild_id: Option<Snowflake>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub topic: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct GuildEvent {
    pub id: Snowflake,
    pub guild_id: Snowflake,
    pub channel_id: Option<Snowflake>,
    #[serde(default)]
    pub creator_id: Option<Snowflake>,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub image: Option<String>,
    pub scheduled_start_time: DateTime<Utc>,
    pub scheduled_end_time: Option<DateTime<Utc>>,
    pub privacy_level: GuildEventPrivacyLevel,
    pub status: GuildEventStatus,
    pub entity_type: GuildEventEntityType,
    pub entity_id: Option<String>,
    pub entity_metadata: Option<GuildEventEntityMetadata>,
    #[serde(default)]
    pub creator: Option<User>
}

#[derive(Deserialize, Debug)]
pub struct GuildEventEntityMetadata {
    pub location: Option<String>
}

#[derive(Debug, serde_repr::Deserialize_repr)]
#[repr(i32)]
pub enum GuildEventPrivacyLevel {
    GuildOnly = 2
}

#[derive(Debug, serde_repr::Deserialize_repr)]
#[repr(i32)]
pub enum GuildEventStatus {
    Scheduled = 1,
    Active = 2,
    Completed = 3,
    Cancelled = 4
}

#[derive(Debug, serde_repr::Deserialize_repr)]
#[repr(i32)]
pub enum GuildEventEntityType {
    Stage = 1,
    Voice = 2,
    External = 3
}