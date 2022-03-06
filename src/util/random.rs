use onig::Regex;
use twilight_model::id::{Id, marker::{GuildMarker, GenericMarker}};

pub fn add_commas(s: &str) -> String {
    let re = Regex::new(r"/(\d)(?=(\d{3})+(?!\d))/g").unwrap();
    
    re.replace_all(s, "$1,")
}

pub fn remove_leading_hashtag(mut s: &str) -> String {
    if s.starts_with("#") {
        let mut chars = s.chars();
        chars.next();
        s = chars.as_str();
    }

    s.to_string()
}

pub fn validate_hex_code(hex_code: &str) -> bool {
    let re = Regex::new("^(?i)([0-9A-F]{3}){1,2}$").unwrap();

    re.is_match(hex_code)
}

pub fn get_shard_id(guild_id: Id<GuildMarker>, total_shards: u64) -> u64 {
    (guild_id.get() >> 22) % total_shards
}

pub fn humanize(mut milliseconds: u64, show_ms: bool) -> String {
    let days = milliseconds / 86_400_000;
    milliseconds = milliseconds % 86_400_000;
    let hours = milliseconds / 3_600_000;
    milliseconds = milliseconds % 3_600_000;
    let minutes = milliseconds / 60_000;
    milliseconds = milliseconds % 60_000;
    let seconds = milliseconds / 1_000;
    milliseconds = milliseconds % 1_000;


    let parts = vec![(days, "d"), (hours, "h"), (minutes, "m"), (seconds, "s"), (milliseconds, "ms")];
    let duration: String = parts.iter().filter_map(|(value, unit)| match *unit {
           "ms" if *value > 0 && show_ms => Some(format!("{value}{unit}")),
           _ if *value > 0 => Some(format!("{value}{unit}")),
           _ => None
    }).collect::<Vec<String>>().join(" ");

    duration
}

pub fn snowflake_to_ms(snowflake: Id<GenericMarker>) -> u64 {
    (snowflake.get() >> 22) + 1420070400000
}