use crate::util::{context::Context, random::{get_shard_id, snowflake_to_ms}};
use std::sync::Arc;
use twilight_embed_builder::{EmbedBuilder, EmbedFooterBuilder};
use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::application::{
    callback::InteractionResponse,
    interaction::ApplicationCommand
};
use twilight_util::builder::CallbackDataBuilder;

#[derive(CommandModel, CreateCommand)]
#[command(
    desc = "Checks Discord API latency",
    name = "ping"
)]
pub struct PingCommand;

impl PingCommand {
    pub async fn run(command: ApplicationCommand, context: Arc<Context>) {
        let interaction_client = context.get_interaction_client();

        interaction_client
            .interaction_callback(
                command.id,
                &command.token,
                &InteractionResponse::DeferredChannelMessageWithSource(CallbackDataBuilder::new().build()))
            .exec()
            .await
            .unwrap();
        let deferred_message = interaction_client
            .get_interaction_original(&command.token)
            .exec()
            .await
            .unwrap()
            .model()
            .await
            .unwrap();
        let rtt = snowflake_to_ms(deferred_message.id.cast()) - snowflake_to_ms(command.id.cast());
        let shard_id = get_shard_id(command.guild_id.unwrap(), context.cluster.shards().len().try_into().unwrap());
        let description = if let Ok(info) = context.cluster.shard(shard_id).unwrap().info() {
            if info.latency().heartbeats() > 0 {
                format!("🏓 **Latency**: {} ms\n🔂 **RTT**: {} ms", info.latency().average().unwrap().as_millis(), rtt)
            } else {
                format!("🔂 **RTT**: {} ms", rtt)
            }
        } else {
            "No data returned.".to_string()
        };
        let embed = EmbedBuilder::new()
            .color(0xF8F8FF)
            .description(description)
            .footer(EmbedFooterBuilder::new(format!("Shard {} stats", shard_id)))
            .build()
            .unwrap();

        interaction_client
            .update_followup_message(&command.token, deferred_message.id)
            .embeds(Some(&[embed]))
            .unwrap()
            .exec()
            .await
            .unwrap();
    }
}