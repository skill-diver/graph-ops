use etcd_rs::{Client, ClientConfig, Endpoint, KeyValueOp};
use std::error::Error;

pub(super) struct EtcdStorage {
    client: Client,
}

impl EtcdStorage {
    pub(super) async fn new(endpoints: Vec<impl Into<String>>) -> Result<Self, Box<dyn Error>> {
        let endpoints: Vec<Endpoint> = endpoints.into_iter().map(Endpoint::new).collect();
        let client = Client::connect(ClientConfig::new(endpoints)).await?;
        Ok(Self { client })
    }

    pub(super) async fn put(&self, key: &str, value: &str) -> Result<(), Box<dyn Error>> {
        let _resp = self.client.put((key, value)).await?;
        Ok(())
    }

    pub(super) async fn get(&self, key: &str) -> Result<String, Box<dyn Error>> {
        let resp = self.client.get(key).await?;
        let value = resp.kvs.first();
        if let Some(kv) = value {
            Ok(kv.value_str().to_string())
        } else {
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("No entry found for key {key}"),
            )))
        }
    }

    pub(super) async fn get_all(&self, key: &str) -> Result<Vec<String>, Box<dyn Error>> {
        let resp = self.client.get_by_prefix(key).await?;
        Ok(resp.kvs.iter().map(|e| e.value_str().to_string()).collect())
    }
}
