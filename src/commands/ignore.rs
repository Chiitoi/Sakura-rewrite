use crate::util::context::Context;
use std::sync::Arc;
use twilight_embed_builder::EmbedBuilder; 
use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::application::{
    callback::InteractionResponse,
    interaction::{ApplicationCommand, application_command::InteractionChannel}
};
use twilight_util::builder::CallbackDataBuilder;

#[derive(CommandModel, CreateCommand)]
#[command(
    desc = "Modifies the list of ignored channels",
    name = "ignore"
)]
pub enum IgnoreCommand {
    #[command(name = "add")]
    Add(IgnoreAdd),
    #[command(name = "remove")]
    Remove(IgnoreRemove)
}

#[derive(CommandModel, CreateCommand)]
#[command(desc = "Adds a channel to the ignore list", name = "add")]
pub struct IgnoreAdd {
    #[command(channel_types = "guild_text", desc = "The channel to ignore")]
    channel: InteractionChannel
}

#[derive(CommandModel, CreateCommand)]
#[command(desc = "Removes a channel from the ignore list", name = "remove")]
pub struct IgnoreRemove {
    #[command(channel_types = "guild_text", desc = "The channel to not ignore anymore")]
    channel: InteractionChannel
}

impl IgnoreCommand {
    pub async fn run(command: ApplicationCommand, context: Arc<Context>) {
        let guild_id = command.guild_id.unwrap();
        let options = IgnoreCommand::from_interaction(command.data.into()).unwrap();
        let ignored_channel_ids = context.database.read_ignored_channel_ids(guild_id).await;
        let mut embed = EmbedBuilder::new().color(0xF8F8FF);
    
        embed = match options {
            IgnoreCommand::Add(IgnoreAdd { channel }) => {
                if ignored_channel_ids.contains(&channel.id) {
                    embed.description("This channel is already ignored.")
                } else {
                    ignored_channel_ids.insert(channel.id);
                    context.database.update_ignored_channel_ids(guild_id, ignored_channel_ids).await;
                    embed.description(format!("<#{}> will now be ignored during invite checks.", channel.id))
                }
            },
            IgnoreCommand::Remove(IgnoreRemove { channel }) => {
                if !ignored_channel_ids.contains(&channel.id) {
                    embed.description("This channel is not in the \"ignored\" list.")
                } else {
                    ignored_channel_ids.remove(&channel.id);
                    context.database.update_ignored_channel_ids(guild_id, ignored_channel_ids).await;
                    embed.description(format!("<#{}> will no longer be ignored during invite checks.", channel.id))
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