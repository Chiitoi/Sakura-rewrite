use chrono::{DateTime, Utc};
use crate::{
    constants::CLIENT_ID,
    database::setting::Setting,
    util::{
        context::Context,
        invite::extract_codes_from_message,
        random::{add_commas, humanize}
    }
};
use std::{
    collections::{HashMap, HashSet},
    cmp,
    fmt,
    error::Error,
    sync::Arc
};
use twilight_embed_builder::{
    EmbedBuilder,
    EmbedFieldBuilder,
    EmbedFooterBuilder
};
use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::{
    application::{callback::InteractionResponse, interaction::ApplicationCommand},
    channel::{embed::Embed, GuildChannel, message::MessageFlags},
    datetime::Timestamp,
    id::{Id, marker::{ChannelMarker, MessageMarker}},
    guild::Permissions, 
};
use twilight_util::builder::CallbackDataBuilder;

#[derive(CommandModel, CreateCommand)]
#[command(
    desc = "Runs an invite check",
    name = "check"
)]
pub struct CheckCommand;

pub struct ChannelResult {
    bad: u32,
    channel_id: Id<ChannelMarker>,
    good: u32
}

impl ChannelResult {
    fn new(channel_id: Id<ChannelMarker>) -> Self {
        Self {
            bad: 0,
            channel_id,
            good: 0
        }
    }
}

impl fmt::Display for ChannelResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (emoji, description) = if self.bad > 0 { ('🔴', format!(" (**{}** bad)", self.bad)) } else { ('🟢', "".to_string()) };
        let total = self.bad + self.good;

        write!(f, "{emoji} <#{}> - **{total}** total{description}", self.channel_id)
    }
}

pub struct CategoryResult {
    channel_results: Vec<ChannelResult>,
    issues: u32,
    manual: Vec<Id<ChannelMarker>>,
    name: String
}

impl CategoryResult {
    fn new(name: String) -> Self {
        Self {
            channel_results: vec![],
            issues: 0,
            manual: vec![],
            name
        }
    }

    fn embed(&self, color: u32) -> Embed {        
        let (description, footer) = if self.channel_results.len() > 0 {
            (
                self.channel_results.iter().map(|channel_result| format!("{}", channel_result)).collect::<Vec<String>>().join("\n"),
                EmbedFooterBuilder::new("Checked 15 messages")
            )
        } else {
            (
                "No channels to check in this category.".to_string(),
                EmbedFooterBuilder::new("Checked 0 messages")
            )
        };
        let mut embed = EmbedBuilder::new()
            .color(color)
            .description(description)
            .footer(footer)
            .timestamp(Timestamp::from_secs(Utc::now().timestamp()).unwrap())
            .title(format!("The \'{}\" category", self.name));

        if self.issues > 0 {
            embed = embed.field(EmbedFieldBuilder::new(
                "Issues",
                format!("- {} channel(s) could not be checked", self.channel_results.len())
            ).build());
        }
        if self.manual.len() > 0 {
            embed = embed.field(EmbedFieldBuilder::new(
                "Manual check(s) required",
                self.manual.iter().map(|channel_id| format!("- <#{}>", channel_id)).collect::<Vec<String>>().join("\n")
                
            ).build());
        }
        
        embed.build().unwrap()
    }
}


pub struct InviteCheck {
    category_results: Vec<CategoryResult>,
    start_time: DateTime<Utc>
}

impl InviteCheck {
    fn new() -> Self {
        Self {
            category_results: vec![],
            start_time: Utc::now()
        }
    }

    fn end_and_show_results(&self, color: u32) -> Embed {
        let end_time = Utc::now();
        let elapsed_time = humanize((end_time.timestamp_millis() - self.start_time.timestamp_millis()) as u64, true);
        let mut total_channels = 0;
        let mut total_bad = 0;
        let mut total_good = 0;

        for CategoryResult { channel_results, issues, manual, .. } in &self.category_results {
            total_channels += channel_results.len() as u32 + issues + manual.len() as u32;

            if channel_results.len() == 0 {
                continue
            }

            for ChannelResult { bad, good, .. } in channel_results {
                total_bad += bad;
                total_good += good;
            }
        }

        let total_invites = cmp::max(total_bad + total_good, 1);
        let stats = vec![
            format!("- **{}** channel(s) checked", add_commas(&total_channels.to_string())),
            format!("- **{}** invite(s) checked", add_commas(&total_invites.to_string())),
            format!("- **{total_bad}** ({:.2}%) invalid invite(s)", (total_bad * 100) as f32 / total_invites as f32),
            format!("- **{total_good}** ({:.2}%) valid invite(s)", (total_good * 100) as f32 / total_invites as f32)
        ].join("\n");
        
        EmbedBuilder::new()
            .color(color)
            .field(EmbedFieldBuilder::new("Elapsed time", elapsed_time).build())
            .field(EmbedFieldBuilder::new("Stats", stats).build())
            .timestamp(Timestamp::from_secs(Utc::now().timestamp()).unwrap())
            .title("Invite check results")
            .build()
            .unwrap()
    }
}

impl CheckCommand {
    pub async fn run(command: ApplicationCommand, context: Arc<Context>) -> Result<(), Box<dyn Error + Send + Sync>> {
        let guild_id = command.guild_id.unwrap();
        let setting = context.database.read_setting(guild_id).await;
        let error_embed = EmbedBuilder::new().color(0xF8F8FF);
        let now = Utc::now();
        let setting_error_description = match &setting {
            Some(setting) => {
                let remaining_seconds = match setting.last_check {
                    Some(ndt) => (((ndt.timestamp_millis() - now.timestamp_millis() + 86_400_000) as f64) / 1000f64).floor() as i64,
                    None => 0
                };

                if remaining_seconds > 0 {
                    let next_check_s = now.timestamp() + remaining_seconds;
                    format!("You may run an invite check at <t:{}> (<t:{}:R>)", next_check_s, next_check_s)
                } else if setting.results_channel_id.is_none() {
                    "No results channel has been set for this guild. Please set one before running an invite check.".to_string()
                } else if context.cache.guild_channel(setting.results_channel_id.unwrap()).is_none() {
                    "Your current results channel may have been deleted. Please set a new one.".to_string()
                } else if setting.results_channel_id.unwrap() != command.channel_id {
                    format!("This command can only be run in <#{}>.", setting.results_channel_id.unwrap())
                } else if setting.category_channel_ids.len() <= 0 {
                    "There are no categories to check. Please add some before running an invite check.".to_string()
                } else if setting.in_check {
                    "Sakura is still checking categories for this guild. Please try again at a later time.".to_string()
                } else {
                    String::new()
                }
            },
            None => "No settings found. Please kick and reinvite Sakura.".to_string(),
        };

        if !setting_error_description.is_empty() {
            context
                .get_interaction_client()
                .interaction_callback(
                    command.id,
                    &command.token,
                    &InteractionResponse::ChannelMessageWithSource(
                        CallbackDataBuilder::new()
                            .embeds(error_embed.description(setting_error_description).build())
                            .flags(MessageFlags::EPHEMERAL)
                            .build()
                    )
                )
                .exec()
                .await?;

            return Ok(())
        }

        let setting = setting.unwrap();
        let known_codes = context.database.read_guild_invites(guild_id).await;
        let codes_error_description = match &known_codes {
            Some(known_codes) => match setting.last_check {
                Some(last_check) if known_codes.values().into_iter().any(|code| code.is_valid.is_some() && code.updated_at < last_check) => {
                    "All invites have not been updated since your last invite check. Please try again at a later time.".to_string()
                },
                _ => String::new(),
            },
            None => "There are no codes to check.".to_string(),
        };

        if !codes_error_description.is_empty() {
            context
                .get_interaction_client()
                .interaction_callback(
                    command.id,
                    &command.token,
                    &InteractionResponse::ChannelMessageWithSource(
                        CallbackDataBuilder::new()
                            .embeds(error_embed.description(codes_error_description).build())
                            .flags(MessageFlags::EPHEMERAL)
                            .build()
                    )
                )
                .exec()
                .await?;

            return Ok(())
        }

        context.database.update_in_check(guild_id, true).await;
        context
            .get_interaction_client()
            .interaction_callback(
                command.id,
                &command.token,
                &InteractionResponse::ChannelMessageWithSource(
                    CallbackDataBuilder::new()
                        .embeds(EmbedBuilder::new().color(setting.embed_color).description("Sakura is checking your invites now!").build())
                        .build()
                )
            )
            .exec()
            .await?;

        let known_codes = known_codes.unwrap();
        let Setting { category_channel_ids, ignored_channel_ids, ..} = setting;
        let results_channel_id = setting.results_channel_id.unwrap();
        let guild_channel_ids = context.cache.guild_channels(guild_id).unwrap();
        let mut ids: HashMap<Id<ChannelMarker>, HashSet<(Id<ChannelMarker>, Option<Id<MessageMarker>>, i64)>> = HashMap::new();
        let mut invite_check = InviteCheck::new();
        let minimum_client_permissions = Permissions::READ_MESSAGE_HISTORY | Permissions::VIEW_CHANNEL;
       
        for guild_channel_id in guild_channel_ids.value() {
            match context.cache.guild_channel(*guild_channel_id)  {
                Some(channel) => match channel.value().resource() {
                    GuildChannel::Text(text) if text.parent_id.is_some() => {
                        match text.parent_id {
                            Some(parent_id) if category_channel_ids.contains(&parent_id) && !ignored_channel_ids.contains(&text.id) => {
                                ids.entry(parent_id).or_default().insert((text.id, text.last_message_id, text.position));
                            },
                            _ => continue,
                        };
                    },
                    _ => continue
                },
                _ => continue
            }
        }

        let mut category_ids = guild_channel_ids.value().clone();
        category_ids.retain(|channel_id| category_channel_ids.contains(channel_id));
        let mut sorted_categories: Vec<(Id<ChannelMarker>, String , i64)> = category_ids
            .iter()
            .filter_map(|channel_id| match context.cache.guild_channel(*channel_id) {
                Some(channel) => match channel.value().resource() {
                    GuildChannel::Category(category) => Some((category.id, category.name.clone(), category.position)),
                    _ => None
                },
                None => None,
            })
            .collect();
        sorted_categories.sort_by(|a , b| a.2.cmp(&b.2));

        for sorted_category in sorted_categories {
            let mut category_result = CategoryResult::new(sorted_category.1);
            let children = ids.get(&sorted_category.0);

            if children.is_none() {
                context
                    .client
                    .create_message(results_channel_id)
                    .embeds(&[category_result.embed(setting.embed_color)])?
                    .exec()
                    .await?;
                invite_check.category_results.push(category_result);
                continue
            }

            let mut sorted_children: Vec<(Id<ChannelMarker>, Option<Id<MessageMarker>>, i64)> = Vec::from_iter(children.unwrap().to_owned());
            sorted_children.sort_by(|a, b| a.2.cmp(&b.2));

            for (channel_id, last_message_id, ..) in sorted_children {
                let mut channel_result = ChannelResult::new(channel_id);
                let channel_reference = match context.cache.guild_channel(channel_id) {
                    Some(channel) => channel,
                    None => {
                        category_result.issues += 1;
                        continue
                    }
                };
                let channel = channel_reference.value().resource();
                
                match context.cache.permissions().in_channel(CLIENT_ID.clone(), channel_id) {
                    Ok(permissions) if permissions.contains(minimum_client_permissions) => {},
                    _ => {
                        category_result.manual.push(channel.id());
                        continue
                    }
                };

                if last_message_id.is_none() {
                    category_result.channel_results.push(channel_result);
                    continue
                }

                let mut request = context.client.channel_messages(channel_id).limit(15).unwrap().exec();
                let context_clone = context.clone();

                request.set_pre_flight(Box::new(move || {
                    match context_clone.cache.channel_messages(channel_id) {
                        Some(message_ids) => message_ids.count() < 15,
                        None => true,
                    }
                }));

                let messages = match request.await?.models().await {
                    Ok(messages) => messages,
                    _ => {
                        category_result.manual.push(channel.id());
                        continue
                    }
                };
                let mut codes: HashSet<String> = HashSet::new();

                for message in messages {
                    let extracted = extract_codes_from_message(message);
                    codes.extend(extracted);
                }

                for code in codes {
                    match known_codes.get(&code) {
                        Some(known_code) if known_code.is_checked => {
                            let is_expired_code = match known_code.expires_at {
                                Some(ndt) => ndt.timestamp_millis() <= now.timestamp_millis(),
                                None => false,
                            };

                            if known_code.is_valid.unwrap() && (known_code.is_permanent.unwrap() || !is_expired_code) {
                                channel_result.good += 1;
                            } else {
                                channel_result.bad += 1;
                            }
        
                        },
                        _ => {
                            let (expires_at, is_permanent, is_valid) = match context.client.invite(&code).with_expiration().exec().await {
                                Ok(response) => {
                                    let invite = response.model().await.unwrap();
    
                                    channel_result.good += 1;
                                    (invite.expires_at, invite.expires_at.is_none() && invite.max_age.is_none() && invite.max_uses.is_none(), true)
                                },
                                Err(_) => {
                                    channel_result.bad += 1;
                                    (None, false, false)
                                },
                            };

                            context.database.upsert_code(guild_id, code, expires_at, is_permanent, is_valid).await;
                        },
                    }
                }

                category_result.channel_results.push(channel_result);
            }
            
            context
                .client
                .create_message(results_channel_id)
                .embeds(&[category_result.embed(setting.embed_color)])?
                .exec()
                .await?;
            invite_check.category_results.push(category_result);
        }

        context
            .client
            .create_message(results_channel_id)
            .embeds(&[invite_check.end_and_show_results(setting.embed_color)])?
            .exec()
            .await?;

        context.database.update_last_check(guild_id).await;
        context.database.update_in_check(guild_id, false).await;

        Ok(())
    }
}