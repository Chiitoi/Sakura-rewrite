use crate::{database::invite::Code, util::context::Context};
use futures_util::stream::{self, StreamExt};
use std::sync::Arc;

pub async fn unchecked_codes(context: Arc<Context>, amount: u16) {  
    let context_clone = context.clone();
    if let Some(unchecked_codes) = context.database.read_unchecked_codes(amount).await {
        stream::iter(unchecked_codes)
            .for_each_concurrent(
                2,
                move |Code { guild_id, code }| {
                    let context_clone = context_clone.clone();

                    async move {
                        let (expires_at, is_permanent, is_valid, ) = match context_clone.client.invite(&code).with_expiration().exec().await {
                            Ok(response) => {
                                let invite = response.model().await.unwrap();

                                (invite.expires_at, invite.expires_at.is_none() && invite.max_age.is_none() && invite.max_uses.is_none(), true)
                            },
                            Err(_) => (None, false, false),
                        };

                        context_clone.database.upsert_code(guild_id, code, expires_at, is_permanent, is_valid).await;
                    }
                }
            ).await;
    }
}