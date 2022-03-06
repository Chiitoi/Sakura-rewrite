use lazy_static::lazy_static;
use std::env;
use onig::*;
use twilight_gateway::Intents;
use twilight_model::id::{Id, marker::{ApplicationMarker, GuildMarker, UserMarker}};

lazy_static! {
    pub static ref APPLICATION_ID: Id<ApplicationMarker> = Id::new(env::var("APPLICATION_ID").unwrap().parse::<u64>().unwrap());
    pub static ref CLIENT_ID: Id<UserMarker> = Id::new(env::var("APPLICATION_ID").unwrap().parse::<u64>().unwrap());
    pub static ref DATABASE_URL: String = env::var("DATABASE_URL").unwrap();
    pub static ref DISCORD_INVITE_REGEX: Regex = Regex::new(r"(?i)(?:https?:\/\/)?(?:\w+\.)?discord(?:(?:app)?\.com\/invite|\.gg)\/(?<code>[a-z0-9-]+)").unwrap();
    pub static ref INVITE_CHECK_COOLDOWN: u64 = env::var("INVITE_CHECK_COOLDOWN").unwrap().parse::<u64>().unwrap();
    pub static ref TOKEN: String = env::var("BOT_TOKEN").unwrap();
    pub static ref INTENTS: Intents = Intents::GUILDS | Intents::GUILD_MESSAGES;
    pub static ref TEST_GUILD_ID: Id<GuildMarker> = Id::new(env::var("TEST_GUILD_ID").unwrap().parse::<u64>().unwrap());
}