use std::sync::{Arc, Mutex};

use crate::infra::pi::{storage::*, Sinkable, Sourceable};

// submodules
mod redis_row_sink;
use redis_row_sink::*;

// TODO(han): enable async redis client
#[derive(Debug, Clone)]
pub struct RedisConnector {
    client: Arc<Mutex<redis::Client>>,
}

pub fn get_connection_arc(
    client: &Arc<Mutex<redis::Client>>,
) -> redis::RedisResult<redis::Connection> {
    client.as_ref().lock().unwrap().get_connection()
}

impl RedisConnector {
    pub fn new(uri: impl redis::IntoConnectionInfo) -> Self {
        Self {
            client: Arc::new(Mutex::new(redis::Client::open(uri).unwrap())),
        }
    }

    pub fn get_connection(&self) -> redis::Connection {
        get_connection_arc(&self.client).unwrap()
    }

    pub fn get_client(&self) -> Arc<Mutex<redis::Client>> {
        self.client.clone()
    }
}

impl Sinkable for RedisConnector {
    fn get_supported_sources(&self) -> Vec<Storage> {
        vec![Storage::OfnilRow]
    }

    fn insert_rows(&self, type_info: Schema) -> Box<dyn Sink<Row>> {
        Box::new(RedisRowSink::new(self.get_client(), type_info))
    }
}

impl Sourceable for RedisConnector {
    fn get_supported_sinks(&self) -> Vec<Storage> {
        vec![Storage::OfnilRow]
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_redis() {
    use redis::{cmd, Commands, ConnectionLike};
    let redis_uri = std::env::var("REDIS_URI");
    println!("redis_uri={redis_uri:?}");
    if let Ok(uri) = redis_uri {
        let client = RedisConnector::new(uri);
        let mut conn = client.get_connection();
        let value = conn.req_command(&cmd("ping")).unwrap();
        assert_eq!(value, redis::Value::Status("PONG".to_string()));
        let _ = conn.set::<&str, &str, ()>("123", "456");
        let val: String = conn.get_del("123").unwrap();
        assert_eq!(val, "456");
    }
}
