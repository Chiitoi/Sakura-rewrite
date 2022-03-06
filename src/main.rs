extern crate onig;

mod commands;
mod constants;
mod database;
mod events;
mod tasks;
mod util;

use commands::*;
use constants::*;
use dotenv::dotenv;
use futures_util::stream::StreamExt;
use std::{error::Error, sync::Arc};
use twilight_gateway::cluster::{ClusterBuilder, ShardScheme};
use twilight_http::client::ClientBuilder;
use twilight_interactions::command::CreateCommand;
use util::context::Context;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    dotenv().ok();


    let client = ClientBuilder::new()
        .token(TOKEN.to_string())
        .build();
    let client = Arc::new(client);
    let client_clone = client.clone();
    let gateway_info = client
        .gateway()
        .authed()
        .exec()
        .await
        .unwrap()
        .model()
        .await;
    let shard_scheme = match gateway_info {
        Ok(info) => ShardScheme::Range {
            from: 0,
            to: info.shards - 1,
            total: info.shards
        },
        Err(_) => ShardScheme::Auto,
    };
    let (cluster, mut events) = ClusterBuilder::new(TOKEN.to_string(), *INTENTS)
        // .event_types(EventTypeFlags::SHARD_PAYLOAD)
        .http_client(client_clone)
        .shard_scheme(shard_scheme)
        .build()
        .await?;
    let context = Arc::new(Context::new(client, cluster));
    let context_clone = context.clone();
  
    tokio::spawn(async move {
        context_clone.cluster.up().await;
        context_clone.database.create_tables().await;
    });
    tokio::spawn(tasks::start(context.clone()));



    context
        .get_interaction_client()
        .set_guild_commands(
            *TEST_GUILD_ID,
            &[
                CategoryCommand::create_command().into(),
                CheckCommand::create_command().into(),
                IgnoreCommand::create_command().into(),
                PingCommand::create_command().into(),
                SetCommand::create_command().into(),
                SettingsCommand::create_command().into(),
                StatsCommand::create_command().into()
            ]
        )
        .exec()
        .await?;

    while let Some((_, event)) = events.next().await {
        tokio::spawn(events::handle(event, context.clone()));
    }

    Ok(())
}