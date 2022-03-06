use crate::util::context::Context;
use std::sync::Arc;
use twilight_embed_builder::{EmbedBuilder, EmbedFieldBuilder};
use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::application::{callback::InteractionResponse, interaction::ApplicationCommand};
use twilight_util::builder::CallbackDataBuilder;


#[derive(CommandModel, CreateCommand)]
#[command(
    desc = "Displays a guild\'s settings",
    name = "settings"
)]
pub struct SettingsCommand;

impl SettingsCommand {
    pub async fn run(command: ApplicationCommand, context: Arc<Context>) {
        let guild_id = command.guild_id.unwrap();
        let mut embed = EmbedBuilder::new().color(0xF8F8FF);
        
        embed = match context.database.read_setting(guild_id).await {
            Some(setting) => {
                let categories_text = if setting.category_channel_ids.len() == 0 {
                    "No categories added".to_string()
                } else {
                    setting.category_channel_ids.into_iter().map(|channel_id| {
                        match context.cache.guild_channel(channel_id){
                            Some(_) => format!("<#{}>", channel_id),
                            None => format!("{} **(no longer exists)**", channel_id),
                        }
                    }).collect::<Vec<String>>().join("\n")
                };
                let color_text = format!("#{:06X}", setting.embed_color);
                let ignored_text = if setting.ignored_channel_ids.len() == 0 {
                    "No channels ignored".to_string()
                } else {
                    setting.ignored_channel_ids.into_iter().map(|channel_id| {
                        match context.cache.guild_channel(channel_id){
                            Some(_) => format!("<#{}>", channel_id),
                            None => format!("{} **(no longer exists)**", channel_id),
                        }
                    }).collect::<Vec<String>>().join("\n")
                };
                let result_text = match setting.results_channel_id {
                    Some(channel_id) => format!("<#{}>", channel_id),
                    None => "No results channel set".to_string()
                };

                embed
                    .field(EmbedFieldBuilder::new("Categories", categories_text).build())
                    .field(EmbedFieldBuilder::new("Embed color", color_text).build())
                    .field(EmbedFieldBuilder::new("Ignored", ignored_text).build())
                    .field(EmbedFieldBuilder::new("Results channel", result_text).build())              
            },
            None => {
                embed.description("No settings found. Please kick and reinvite Sakura.")
            }
        };

        context
            .get_interaction_client()
            .interaction_callback(
                command.id,
                &command.token,
                &InteractionResponse::ChannelMessageWithSource(
                    CallbackDataBuilder::new().embeds(embed.build()).build()
                )
            )
            .exec()
            .await
            .unwrap();
    }
}