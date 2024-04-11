use std::path::Path;

use aws_config::BehaviorVersion;
use aws_sdk_s3::{
    config::SharedCredentialsProvider,
    operation::{
        delete_object::DeleteObjectOutput, get_object::GetObjectOutput,
        head_object::HeadObjectOutput, list_objects_v2::ListObjectsV2Output,
    },
    primitives::ByteStream,
    Client,
};
use secrecy::ExposeSecret;

use crate::{config::Config, error::Error, util};

pub struct S3 {
    client: Client,
    bucket: String,
}

impl S3 {
    pub async fn new(config: &Config) -> Self {
        let sdk_config = aws_config::defaults(BehaviorVersion::latest())
            .region(util::string_to_static_str(config.s3_region.clone()))
            .endpoint_url(config.s3_endpoint.clone().as_str())
            .credentials_provider(SharedCredentialsProvider::new(
                aws_credential_types::Credentials::from_keys(
                    config.s3_access_key.clone().as_str(),
                    config.s3_secret_key.clone().expose_secret().as_str(),
                    None,
                ),
            ))
            .load()
            .await;

        S3 {
            client: Client::new(&sdk_config),
            bucket: config.s3_bucket.clone(),
        }
    }

    pub async fn list(&self, prefix: String) -> Result<ListObjectsV2Output, Error> {
        self.client
            .list_objects_v2()
            .bucket(self.bucket.clone())
            .prefix(prefix)
            .send()
            .await
            .map_err(|_| {
                Error::InternalServerError("Something went wrong while listing files".into())
            })
    }

    pub async fn head(&self, path: String) -> Result<HeadObjectOutput, Error> {
        self.client
            .head_object()
            .bucket(self.bucket.clone())
            .key(path)
            .send()
            .await
            .map_err(|_| {
                Error::InternalServerError("Something went wrong while getting file head".into())
            })
    }

    pub async fn get(&self, path: String) -> Result<GetObjectOutput, Error> {
        self.client
            .get_object()
            .bucket(self.bucket.clone())
            .key(path)
            .send()
            .await
            .map_err(|_| {
                Error::InternalServerError("Something went wrong while getting a file".into())
            })
    }

    pub async fn upload(
        &self,
        file_name: String,
        content_type: String,
        file_path: &Path,
    ) -> Result<aws_sdk_s3::operation::put_object::PutObjectOutput, Error> {
        self.client
            .put_object()
            .bucket(self.bucket.clone())
            .key(file_name)
            .content_type(content_type)
            .body(ByteStream::from_path(file_path).await.unwrap())
            .send()
            .await
            .map_err(|_| {
                Error::InternalServerError("Something went wrong while uploading files".into())
            })
    }

    pub async fn delete(&self, path: String) -> Result<DeleteObjectOutput, Error> {
        self.client
            .delete_object()
            .bucket(self.bucket.clone())
            .key(path)
            .send()
            .await
            .map_err(|_| {
                Error::InternalServerError("Something went wrong while deleting a file".into())
            })
    }
}
