use crate::util::{context::Context, random::{add_commas, humanize}};
use std::sync::Arc;
use sysinfo::{
    ProcessExt,
    ProcessRefreshKind,
    RefreshKind,
    System,
    SystemExt, 
};
use twilight_embed_builder::EmbedBuilder;
use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::application::{
    callback::InteractionResponse,
    interaction::ApplicationCommand
};
use twilight_util::builder::CallbackDataBuilder;


#[derive(CommandModel, CreateCommand)]
#[command(
    desc = "Displays random metrics of interest",
    name = "stats"
)]
pub struct StatsCommand;

impl StatsCommand {
    pub async fn run(command: ApplicationCommand, context: Arc<Context>) {
        let mut system = System::new_with_specifics(RefreshKind::new().with_processes(ProcessRefreshKind::everything()));

        system.refresh_all();

        let guild_count = add_commas(&context.cache.stats().guilds().to_string());
        let process = system.process(sysinfo::get_current_pid().unwrap()).unwrap();
        let memory = add_commas(&((f64::trunc((process.memory() as f64 / 1024_f64)  * 100.0) / 100.0).to_string()));
        let uptime = humanize(process.run_time() * 1000, false);
        let embed = EmbedBuilder::new()
            .color(0xF8F8FF)
            .description(format!("**Guild(s):** {}\n**Memory used:** {} MB\n**Uptime:** {}", guild_count, memory, uptime))
            .build();

        context
            .get_interaction_client()
            .interaction_callback(
                command.id,
                &command.token,
                &InteractionResponse::ChannelMessageWithSource(
                    CallbackDataBuilder::new().embeds(embed).build()
                )
            )
            .exec()
            .await
            .unwrap();
    }
}