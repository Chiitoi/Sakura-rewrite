use crate::{constants::DISCORD_INVITE_REGEX, util::context::Context};
use std::{collections::HashSet, sync::Arc};
use twilight_model::{
    channel::{GuildChannel, Message},
    id::{
        Id,
        marker::{ChannelMarker, GuildMarker}
    }
};

pub async fn extract_codes_from_category(guild_id: Id<GuildMarker>, category_id: Id<ChannelMarker>, context: Arc<Context>) {
    if let Some(guild_channel_ids) = context.cache.guild_channels(guild_id) {
        let mut codes: HashSet<String> = HashSet::new();

        for guild_channel_id in guild_channel_ids.iter() {
            let guild_channel_reference = match context.cache.guild_channel(*guild_channel_id) {
                Some(reference) => reference,
                None => continue,
            };
            let guild_channel = guild_channel_reference.value().resource();
            let channel_id_to_search = match guild_channel {
                GuildChannel::Text(channel) => {
                    match (channel.last_message_id, channel.parent_id) {
                        (Some(_), Some(parent_id)) if parent_id == category_id => channel.id,
                        _ => continue
                    }
                }
                _ => continue
            };
            let mut request = context.client.channel_messages(channel_id_to_search).limit(15).unwrap().exec();
            let context_clone = context.clone();

            request.set_pre_flight(Box::new(move || {
                match context_clone.cache.channel_messages(channel_id_to_search) {
                    Some(message_ids) => message_ids.count() < 15,
                    None => true,
                }
            }));

            let messages = match request.await {
                Ok(response) => match response.models().await {
                    Ok(messages) => messages,
                    Err(_) => vec![],
                },
                Err(_) => continue,
            };

            for message in messages {
                let extracted = extract_codes_from_message(message);
                codes.extend(extracted);
            }
        }

        if codes.len() > 0 {
            context.database.create_invites(guild_id, codes).await;
        }
    }
}

pub fn extract_codes_from_message(message: Message) -> HashSet<String> {
    let mut codes = HashSet::new();

    for capture in DISCORD_INVITE_REGEX.captures_iter(&message.content) {
        match capture.at(1) {
            Some(code) => codes.insert(code.to_string()),
            None => continue
        };
    }

    codes
}