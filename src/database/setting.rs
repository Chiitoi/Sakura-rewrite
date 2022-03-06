use chrono::NaiveDateTime;
use dashmap::DashSet;
use super::Database;
use tokio_postgres::Row;
use twilight_model::id::{
    Id,
    marker::{ChannelMarker, GuildMarker}
};

#[derive(Debug)]
pub struct Setting {
    pub guild_id: Id<GuildMarker>,
    pub results_channel_id: Option<Id<ChannelMarker>>,
    pub category_channel_ids: DashSet<Id<ChannelMarker>>,
    pub ignored_channel_ids: DashSet<Id<ChannelMarker>>,
    pub embed_color: u32,
    pub last_check: Option<NaiveDateTime>,
    pub in_check: bool
}

impl From<Row> for Setting {
    fn from(row: Row) -> Self {
        Self {
            guild_id: Id::new(row.get::<_, i64>(0) as u64), 
            results_channel_id: match row.try_get::<_, i64>(1) {
                Ok(channel_id) => Some(Id::new(channel_id as u64)),
                Err(_) => None
            },
            category_channel_ids: row.get::<_, Vec<i64>>(2).into_iter().map(|id| Id::new(id as u64)).collect(),
            ignored_channel_ids: row.get::<_, Vec<i64>>(3).into_iter().map(|id| Id::new(id as u64)).collect(),
            embed_color: row.get::<_, i32>(4) as u32,
            last_check: match row.try_get::<_, chrono::NaiveDateTime>(5) {
                Ok(ndt) => Some(ndt),
                Err(_) => None   
            },
            in_check: row.get(6)  
        }
    }
}

impl Database {
    pub async fn create_setting(&self, guild_id: Id<GuildMarker>) {
        let client = self.get_object().await;
        let query = "INSERT INTO setting(guild_id) VALUES($1) ON CONFLICT DO NOTHING;";

        client.query(query, &[&(guild_id.get() as i64)]).await.unwrap();
    }

    pub async fn delete_channel(&self, guild_id: Id<GuildMarker>, channel_id: Id<ChannelMarker>) {
        if let Some(setting) = self.read_setting(guild_id).await {
            let client = self.get_object().await;
            let query = "UPDATE setting SET category_channel_ids = $1, ignored_channel_ids = $2, results_channel_id = $3 WHERE guild_id = $4;";
            let results_channel_id = match setting.results_channel_id {
                Some(results_channel_id) => if results_channel_id == channel_id { None::<i64> } else { Some(results_channel_id.get() as i64) },
                None => Some(channel_id.get() as i64),
            };

            client.query(
                query,
                &[
                    &setting.category_channel_ids.remove(&channel_id).into_iter().map(|id| id.get() as i64).collect::<Vec<i64>>(),
                    &setting.ignored_channel_ids.remove(&channel_id).into_iter().map(|id| id.get() as i64).collect::<Vec<i64>>(),
                    &results_channel_id,
                    &(guild_id.get() as i64)
                ]
            ).await.unwrap();
        }
    }

    pub async fn read_category_channel_ids(&self, guild_id: Id<GuildMarker>) -> DashSet<Id<ChannelMarker>> {
        let client = self.get_object().await;
        let query = "SELECT category_channel_ids FROM setting WHERE guild_id = $1;";
        let row = client.query_one(query, &[&(guild_id.get() as i64)]).await.unwrap();
        let category_channel_ids = row.get::<_, Vec<i64>>(0).into_iter().map(|id| Id::new(id as u64)).collect();

        category_channel_ids
    }

    pub async fn read_ignored_channel_ids(&self, guild_id: Id<GuildMarker>) -> DashSet<Id<ChannelMarker>> {
        let client = self.get_object().await;
        let query = "SELECT ignored_channel_ids FROM setting WHERE guild_id = $1;";
        let row = client.query_one(query, &[&(guild_id.get() as i64)]).await.unwrap();
        let ignored_channel_ids = row.get::<_, Vec<i64>>(0).into_iter().map(|id| Id::new(id as u64)).collect();

        ignored_channel_ids
    }

    pub async fn read_setting(&self, guild_id: Id<GuildMarker>) -> Option<Setting> {
        let client = self.get_object().await;
        let query = "SELECT * FROM setting WHERE guild_id = $1;";

        match client.query_one(query, &[&(guild_id.get() as i64)]).await {
            Ok(row) => Some(row.into()),
            Err(_) => None
        }
    }

    pub async fn update_results_channel_id(&self, guild_id: Id<GuildMarker>, channel_id: Option<Id<ChannelMarker>>) {
        let client = self.get_object().await;
        let query = "UPDATE setting SET results_channel_id = $1 WHERE guild_id = $2;".to_string();
        
        match channel_id {
            Some(channel_id) => client.query(&query, &[&(channel_id.get() as i64), &(guild_id.get() as i64)]).await.unwrap(),
            None => client.query(&query, &[&None::<&[i64]>, &(guild_id.get() as i64)]).await.unwrap()
        };
    }

    pub async fn update_category_channel_ids(&self, guild_id: Id<GuildMarker>, channel_ids: DashSet<Id<ChannelMarker>>) {
        let client = self.get_object().await;
        let query = "UPDATE setting SET category_channel_ids = $1 WHERE guild_id = $2;".to_string();

        client.query(
            &query,
            &[
                &channel_ids.into_iter().map(|id| id.get() as i64).collect::<Vec<i64>>(),
                &(guild_id.get() as i64)
            ]
        ).await.unwrap();
    }

    pub async fn update_ignored_channel_ids(&self, guild_id: Id<GuildMarker>, channel_ids: DashSet<Id<ChannelMarker>>) {
        let client = self.get_object().await;
        let query = "UPDATE setting SET ignored_channel_ids = $1 WHERE guild_id = $2;".to_string();

        client.query(
            &query,
            &[
                &channel_ids.into_iter().map(|id| id.get() as i64).collect::<Vec<i64>>(),
                &(guild_id.get() as i64)
            ]
        ).await.unwrap();
    }

    pub async fn update_embed_color(&self, guild_id: Id<GuildMarker>, color: u32) {
        let client = self.get_object().await;
        let query = "UPDATE setting SET embed_color = $1 WHERE guild_id = $2;".to_string();
        
        client.query(&query, &[&(color as i32), &(guild_id.get() as i64)]).await.unwrap();
    }

    pub async fn update_last_check(&self, guild_id: Id<GuildMarker>) {
        let client = self.get_object().await;
        let query = "UPDATE setting SET last_check = NOW()::TIMESTAMP WHERE guild_id = $1;".to_string();

        client.query(&query, &[&(guild_id.get() as i64)]).await.unwrap();
    }

    pub async fn update_in_check(&self, guild_id: Id<GuildMarker>, in_check: bool) {
        let client = self.get_object().await;
        let query = "UPDATE setting SET in_check = $1 WHERE guild_id = $2;".to_string();

        client.query(&query, &[&in_check, &(guild_id.get() as i64)]).await.unwrap();
    }

    pub async fn delete_setting(&self, guild_id: Id<GuildMarker>) {
        let client = self.get_object().await;
        let query = "DELETE FROM setting WHERE guild_id = $1;";

        client.query(query, &[&(guild_id.get() as i64)]).await.unwrap();
    }
}