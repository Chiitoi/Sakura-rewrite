pub mod setting;
pub mod invite;

use crate::constants::DATABASE_URL;
use deadpool_postgres::{Client, Manager, ManagerConfig, Pool, RecyclingMethod};
use std::str::FromStr;
use tokio_postgres::{Config, NoTls};

pub struct Database {
    pool: Pool
}

impl Database {
    pub fn new() -> Self {
        let pool = Pool::builder(Manager::from_config(
            Config::from_str(&DATABASE_URL).unwrap(),
            NoTls,
            ManagerConfig { recycling_method: RecyclingMethod::Fast }
        ))
            .max_size(16)
            .build()
            .unwrap();

        Self {
            pool
        }
    }

    async fn get_object(&self) -> Client {
        self.pool.get().await.unwrap()
    }

    pub async fn create_tables(&self) {
        let client = self.get_object().await;
        let query = "
            CREATE TABLE IF NOT EXISTS public.setting (
                guild_id INT8 NOT NULL,
                results_channel_id INT8,
                category_channel_ids INT8[] NOT NULL DEFAULT '{}',
                ignored_channel_ids INT8[] NOT NULL DEFAULT '{}',
                embed_color INT4 NOT NULL DEFAULT 16316671,
                last_check TIMESTAMP(3),
                in_check BOOLEAN NOT NULL DEFAULT FALSE,
                CONSTRAINT pk_setting PRIMARY KEY (guild_id)
            );
            CREATE UNIQUE INDEX IF NOT EXISTS idx_setting_guild_id ON public.setting USING btree (guild_id);
            CREATE TABLE IF NOT EXISTS public.invite (
                guild_id INT8 NOT NULL,
                code TEXT NOT NULL,
                expires_at TIMESTAMP(3),
                is_permanent BOOLEAN,
                is_valid BOOLEAN,
                is_checked BOOLEAN NOT NULL DEFAULT FALSE,
                created_at TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
                CONSTRAINT ck_invite PRIMARY KEY (guild_id, code)
            );
            CREATE UNIQUE INDEX IF NOT EXISTS idx_invite_guild_id_code ON public.invite USING btree (guild_id, code);
        ";
        
        client.batch_execute(query).await.unwrap();
    }
}