use crate::{
    constants::APPLICATION_ID,
    database::Database
};
use std::sync::Arc;
use twilight_cache_inmemory::{InMemoryCache, ResourceType};
use twilight_gateway::Cluster;
use twilight_http::client::{Client, InteractionClient};

pub struct Context {
    pub cache: InMemoryCache,
    pub client: Arc<Client>,
    pub cluster: Cluster,
    pub database: Database
}


impl Context {
    pub fn new(client: Arc<Client>, cluster: Cluster) -> Self {
        let resource_types = ResourceType::CHANNEL 
            | ResourceType::GUILD
            | ResourceType::MEMBER
            | ResourceType::MESSAGE
            | ResourceType::ROLE 
            | ResourceType::USER_CURRENT;
        
        Self {
            cache: InMemoryCache::builder()
                .message_cache_size(15)
                .resource_types(resource_types)
                .build(),
            client,
            cluster,
            database: Database::new()
        }
    }

    pub fn get_interaction_client(&self) -> InteractionClient {
        self.client.interaction(*APPLICATION_ID)
    }
}