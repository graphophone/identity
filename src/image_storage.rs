use anyhow::Result;
use aws_config::{BehaviorVersion, Region};
use aws_sdk_s3::{Client, config::Credentials, primitives::ByteStream};
use uuid::Uuid;
use crate::config;

pub struct ImageStorage {
    avatar_bucket: String,
    banner_bucket: String,
    client: Client,
}

impl ImageStorage {
    pub async fn build(conf: &config::RustfsConfig) -> Result<Self> {
        let credentials = Credentials::new(
            &conf.access_key,
            &conf.secret_key,
            None,
            None,
            "rustfs",
        );

        let shared_config = aws_config::defaults(BehaviorVersion::latest())
            .region(Region::new(conf.region.clone()))
            .credentials_provider(credentials)
            .endpoint_url(&conf.endpoint_url)
            .load()
            .await;

        let s3_config = aws_sdk_s3::config::Builder::from(&shared_config)
            .force_path_style(true)
            .build();

        let rustfs_client = Client::from_conf(s3_config);
        Self::ensure_bucket(&rustfs_client, &conf.avatar_bucket).await?;
        Self::ensure_bucket(&rustfs_client, &conf.banner_bucket).await?;

        Ok(ImageStorage {
            client: rustfs_client,
            avatar_bucket: conf.avatar_bucket.clone(),
            banner_bucket: conf.banner_bucket.clone(),
        })
    }

    pub async fn upload_avatar(&self, image: &[u8]) -> Result<String> {
        let key = Uuid::new_v4().to_string();
        self.client
            .put_object()
            .bucket(&self.avatar_bucket)
            .key(&key)
            .body(ByteStream::from(image.to_vec()))
            .send()
            .await?;
        Ok(key)
    }

    pub async fn remove_avatar(&self, key: &str) -> Result<()> {
        self.client
            .delete_object()
            .bucket(&self.avatar_bucket)
            .key(key)
            .send()
            .await?;
        Ok(())
    }

    pub async fn upload_banner(&self, image: &[u8]) -> Result<String> {
        let key = Uuid::new_v4().to_string();
        self.client
            .put_object()
            .bucket(&self.banner_bucket)
            .key(&key)
            .body(ByteStream::from(image.to_vec()))
            .send()
            .await?;
        Ok(key)
    }

    pub async fn remove_banner(&self, key: &str) -> Result<()> {
        self.client
            .delete_object()
            .bucket(&self.banner_bucket)
            .key(key)
            .send()
            .await?;
        Ok(())
    }

    async fn ensure_bucket(rustfs_client: &Client, bucket_name: &str) -> Result<()> {
        let buckets = rustfs_client.list_buckets().send().await?;
        let bucket = buckets.buckets().iter()
            .find(|&b| b.name == Some(bucket_name.to_string()));
        match bucket {
            Some(_) => (),
            None => {
                rustfs_client.create_bucket()
                    .bucket(bucket_name)
                    .send()
                    .await?;
            },
        };
        return Ok(())
    }
}