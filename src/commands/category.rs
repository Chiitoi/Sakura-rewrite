use crate::util::{context::Context, invite::extract_codes_from_category};
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
    desc = "Modifies the list of category channels to check",
    name = "category"
)]
pub enum CategoryCommand {
    #[command(name = "add")]
    Add(CategoryAdd),
    #[command(name = "remove")]
    Remove(CategoryRemove)
}

#[derive(CommandModel, CreateCommand)]
#[command(desc = "Adds a category channel to the list", name = "add")]
pub struct CategoryAdd {
    #[command(channel_types = "guild_category", desc = "The category to add")]
    category: InteractionChannel
}

#[derive(CommandModel, CreateCommand)]
#[command(desc = "Removes a category channel from the list", name = "remove")]
pub struct CategoryRemove {
    #[command(channel_types = "guild_category", desc = "The category to not check anymore")]
    category: InteractionChannel
}

impl CategoryCommand {
    pub async fn run(command: ApplicationCommand, context: Arc<Context>) {
        context
            .get_interaction_client()
            .interaction_callback(
                command.id,
                &command.token,
                &InteractionResponse::DeferredChannelMessageWithSource(CallbackDataBuilder::new().build())
            )
            .exec()
            .await
            .unwrap();

        let guild_id = command.guild_id.unwrap();
        let options = CategoryCommand::from_interaction(command.data.into()).unwrap();
        let category_channel_ids = context.database.read_category_channel_ids(guild_id).await;
        let mut embed = EmbedBuilder::new().color(0xF8F8FF);
    
        embed = match options {
            CategoryCommand::Add(CategoryAdd { category }) => {
                if category_channel_ids.contains(&category.id) {
                    embed.description("This category has already been added.")
                } else {
                    extract_codes_from_category(guild_id, category.id, context.clone()).await;
                    category_channel_ids.insert(category.id);
                    context.database.update_category_channel_ids(guild_id, category_channel_ids).await;
                    embed.description(format!("<#{}> will now be checked during invite checks.", category.id))
                }
            },
            CategoryCommand::Remove(CategoryRemove { category }) => {
                if !category_channel_ids.contains(&category.id) {
                    embed.description("This channel is not in the \"category\" list.")
                } else {
                    category_channel_ids.remove(&category.id);
                    context.database.update_category_channel_ids(guild_id, category_channel_ids).await;
                    embed.description(format!("<#{}> will no longer be checked during invite checks.", category.id))
                }
            },
        };

        context
            .get_interaction_client()
            .update_interaction_original(&command.token)
            .embeds(Some(&[embed.build().unwrap()]))
            .unwrap()
            .exec()
            .await
            .unwrap();
    }
}