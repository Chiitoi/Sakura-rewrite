use crate::util::{context::Context, random::{remove_leading_hashtag, validate_hex_code}};
use std::{iter, sync::Arc};
use twilight_embed_builder::EmbedBuilder; 
use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::application::{
    callback::InteractionResponse,
    interaction::{ApplicationCommand, application_command::InteractionChannel}
};
use twilight_util::builder::CallbackDataBuilder;

#[derive(CommandModel, CreateCommand, Debug)]
#[command(
    desc = "Modifies various server settings",
    name = "set"
)]
pub enum SetCommand {
    #[command(name = "results-channel")]
    ResultsChannel(SetResultsChannel),
    #[command(name = "embed-color")]
    EmbedColor(SetEmbedColor)
}

#[derive(CommandModel, CreateCommand, Debug)]
#[command(desc = "Sets the channel to send invite check results to", name = "results-channel")]
pub struct SetResultsChannel {
    #[command(channel_types = "guild_news guild_text", desc = "The results channel")]
    channel: Option<InteractionChannel>
}

#[derive(CommandModel, CreateCommand, Debug)]
#[command(desc = "Sets the color for invite check result embeds", name = "embed-color")]
pub struct SetEmbedColor {
    #[command(desc = "The (hex) color code")]
    color: String
}

impl SetCommand {
    pub async fn run(command: ApplicationCommand, context: Arc<Context>) {
        let guild_id = command.guild_id.unwrap();
        let options = SetCommand::from_interaction(command.data.into()).unwrap();
        let mut embed = EmbedBuilder::new().color(0xF8F8FF);
        
        embed = match options {
            SetCommand::ResultsChannel(option) => {
                match option.channel {
                    Some(channel) => {
                        context.database.update_results_channel_id(guild_id, Some(channel.id)).await;
                        embed.description(format!("Invite check results will now be sent in <#{}>.", channel.id))
                    },
                    None => {
                        context.database.update_results_channel_id(guild_id, None).await;
                        embed.description("This server no longer has a results channel.")
                    }
                }
            },
            SetCommand::EmbedColor(option) => {
                let hashtag_free_color = remove_leading_hashtag(&option.color).to_uppercase();
                let mut formatted_hashtag_free_color = String::new();
                
                if hashtag_free_color.len() == 3 {
                    formatted_hashtag_free_color.extend(hashtag_free_color.chars().flat_map(|c| iter::repeat(c).take(2)));
                } else {
                    formatted_hashtag_free_color = hashtag_free_color;
                }

                if !validate_hex_code(&formatted_hashtag_free_color) {
                    embed.description("No valid color provided.")
                } else {
                    let color = u32::from_str_radix(&formatted_hashtag_free_color, 16).unwrap();
                    context.database.update_embed_color(guild_id, color).await;
                    embed.description(format!("The embed color for invite check embeds is now **#{:06X}**.", color))
                }
            },
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