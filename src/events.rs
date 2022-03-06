use crate::{
    commands::*,
    constants::CLIENT_ID,
    util::{context::Context, invite::extract_codes_from_message}
};
use std::sync::Arc;
use twilight_model::{
    application::interaction::Interaction,
    channel::Channel,
    gateway::event::Event,
    guild::Permissions
};

pub async fn handle(event: Event, context: Arc<Context>) {
    context.cache.update(&event);

    match event {
        Event::ChannelDelete(channel) => {
            if let Channel::Guild(guild_channel) = channel.0 {
                if let Some(guild_id) = guild_channel.guild_id() {
                    context.database.delete_channel(guild_id, guild_channel.id()).await
                }
            }
        },
        Event::GuildCreate(guild) => context.database.create_setting(guild.id).await,
        Event::GuildDelete(guild) => context.database.delete_setting(guild.id).await,
        Event::InteractionCreate(interaction) => {
            if let Some(guild_id) = interaction.guild_id() {
                if let Interaction::ApplicationCommand(command) = interaction.0 {
                    let user_id = command.clone().member.unwrap().user.unwrap().id;
                    let is_user_admin = match context.cache.permissions().root(user_id, guild_id) {
                        Ok(permissions) => permissions.contains(Permissions::ADMINISTRATOR),
                        Err(_) => false
                    };
                    let minimum_client_permissions = Permissions::EMBED_LINKS | Permissions::READ_MESSAGE_HISTORY | Permissions::SEND_MESSAGES | Permissions::USE_SLASH_COMMANDS | Permissions::VIEW_CHANNEL;
                    let client_can_see_channel = match context.cache.permissions().in_channel(CLIENT_ID.clone(), command.channel_id) {
                        Ok(permissions) => permissions.contains(minimum_client_permissions),
                        Err(_) => false
                    };

                    if is_user_admin & client_can_see_channel {
                        match command.data.name.as_str() {
                            "category" => CategoryCommand::run(*command, context).await,
                            "check" => CheckCommand::run(*command, context).await.unwrap(),
                            "ignore" => IgnoreCommand::run(*command, context).await,
                            "ping" => PingCommand::run(*command, context).await,
                            "set" => SetCommand::run(*command, context).await,
                            "settings" => SettingsCommand::run(*command, context).await,
                            "stats" => StatsCommand::run(*command, context).await,
                            _ => {}
                        }                        
                    }
                }
            }
        },
        Event::MessageCreate(message) => {
            if let Some(guild_id) = message.guild_id {
                let codes = extract_codes_from_message(message.0);
                
                if codes.len() > 0 {
                    context.database.create_invites(guild_id, codes).await;
                }
            }
        },
        Event::Ready(ready) => println!("{}#{} is online!", ready.user.name, ready.user.discriminator),
        _ => {}
    }
}