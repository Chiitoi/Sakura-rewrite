use chrono::NaiveDateTime;
use super::Database;
use std::collections::{HashMap, HashSet};
use tokio_postgres::Row;
use twilight_model::{id::{Id, marker::GuildMarker}, datetime::Timestamp};

#[derive(Debug)]
pub struct Invite {
    pub guild_id: Id<GuildMarker>,
    pub code: String,
    pub expires_at: Option<NaiveDateTime>,
    pub is_permanent: Option<bool>,
    pub is_valid: Option<bool>,
    pub is_checked: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime
}

impl From<Row> for Invite {
    fn from(row: Row) -> Self {
        Self {
            guild_id: Id::new(row.get::<_, i64>(0) as u64), 
            code: row.get(1),
            expires_at: match row.try_get::<_, chrono::NaiveDateTime>(2) {
                Ok(ndt) => Some(ndt),
                Err(_) => None   
            },
            is_permanent: match row.try_get::<_, bool>(3) {
                Ok(value) => Some(value),
                Err(_) => None   
            },
            is_valid: match row.try_get::<_, bool>(4) {
                Ok(value) => Some(value),
                Err(_) => None   
            },
            is_checked: row.get(5),
            created_at: row.get(6),
            updated_at: row.get(7),  
        }
    }
}

#[derive(Eq, Hash, PartialEq)]
pub struct Code {
    pub guild_id: Id<GuildMarker>,
    pub code: String
}

impl From<Row> for Code {
    fn from(row: Row) -> Self {
        Self {
            guild_id: Id::new(row.get::<_, i64>(0) as u64),
            code: row.get(1)
        }
    }
}

impl Database {
    pub async fn create_invites(&self, guild_id: Id<GuildMarker>, codes: HashSet<String>) {
        let client = self.get_object().await;
        let values = codes.iter().map(|code| format!("({}, '{}')", guild_id, code)).collect::<Vec<String>>().join(", ");
        let query = format!("INSERT INTO invite(guild_id, code) VALUES {} ON CONFLICT DO NOTHING;", values);
        
        client.query(&query, &[]).await.unwrap();
    }

    pub async fn read_checked_codes(&self, amount: u16) -> Option<HashSet<Code>> {
        let client = self.get_object().await;
        let query = "SELECT guild_id, code FROM invite WHERE is_checked = TRUE and is_valid = TRUE ORDER BY updated_at LIMIT $1;";

        match client.query(query, &[&(amount as i64)]).await {
            Ok(rows) => {
                let mut codes = HashSet::new();

                for row in rows.into_iter() {
                    codes.insert(Code::from(row));
                }

                Some(codes)
            },
            Err(_) => None
        }
    }

    pub async fn read_guild_invites(&self, guild_id: Id<GuildMarker>) -> Option<HashMap<String, Invite>> {
        let client = self.get_object().await;
        let query = "SELECT * FROM invite WHERE guild_id = $1;";
        
        match client.query(query, &[&(guild_id.get() as i64)]).await {
            Ok(rows) => {
                let mut invites = HashMap::new();

                for row in rows.into_iter() {
                    invites.insert(row.get(1), Invite::from(row));
                }
                
                Some(invites)
            },
            Err(_) => None
        }
    }

    pub async fn read_unchecked_codes(&self, amount: u16) -> Option<HashSet<Code>> {
        let client = self.get_object().await;
        let query = "SELECT guild_id, code FROM invite WHERE is_checked = FALSE ORDER BY created_at LIMIT $1;";

        match client.query(query, &[&(amount as i64)]).await {
            Ok(rows) => {
                let mut codes = HashSet::new();

                for row in rows.into_iter() {
                    codes.insert(Code::from(row));
                }

                Some(codes)
            },
            Err(_) => None
        }
    }

    pub async fn update_code(&self, guild_id: Id<GuildMarker>, code: String, expires_at: Option<Timestamp>, is_permanent: bool, is_valid: bool) {
        let client = self.get_object().await;
        let query = "
            UPDATE invite
            SET 
                expires_at = $3,
                is_permanent = $4,
                is_valid = $5,
                is_checked = TRUE,
                updated_at = CURRENT_TIMESTAMP
            WHERE
                guild_id = $1
                AND code = $2;
        ";

        match expires_at {  
            Some(timestamp) => client.query(query, &[&(guild_id.get() as i64), &code, &NaiveDateTime::from_timestamp(timestamp.as_secs(), 0), &is_permanent, &is_valid]).await.unwrap(),
            None => client.query(query, &[&(guild_id.get() as i64), &code, &None::<NaiveDateTime>, &is_permanent, &is_valid]).await.unwrap(),
        };
    }

    pub async fn upsert_code(&self, guild_id: Id<GuildMarker>, code: String, expires_at: Option<Timestamp>, is_permanent: bool, is_valid: bool) {
        let client = self.get_object().await;
        let query = "
            INSERT INTO invite(guild_id, code, expires_at, is_permanent, is_valid, is_checked)
            VALUES($1, $2, $3, $4, $5, TRUE)
            ON CONFLICT (guild_id, code)
            DO 
            UPDATE SET
                expires_at = EXCLUDED.expires_at,
                is_permanent = EXCLUDED.is_permanent,
                is_valid = EXCLUDED.is_valid,
                is_checked = TRUE,
                updated_at = CURRENT_TIMESTAMP
        ";

        match expires_at {  
            Some(timestamp) => client.query(query, &[&(guild_id.get() as i64), &code, &NaiveDateTime::from_timestamp(timestamp.as_secs(), 0), &is_permanent, &is_valid]).await.unwrap(),
            None => client.query(query, &[&(guild_id.get() as i64), &code, &None::<NaiveDateTime>, &is_permanent, &is_valid]).await.unwrap(),
        };
    }
}