use crate::error::Error;
use aws_config::{AppName, SdkConfig};
use aws_sdk_sso::config::Region;
use chrono;
use chrono::Utc;
use serde_json::Value;
use std::fs;
use std::fs::File;
use std::io::Read;

use super::models::cache_credentials::CachedCredentials;

pub async fn create_mock_config(
    account_id: &str,
    region: &str,
    role_name: &str,
) -> Result<SdkConfig, Error> {
    let mut cached_credentials: Option<CachedCredentials> = None;

    let mut home_directory = dirs::home_dir().expect("Error accessing home directory");
    home_directory.push(".aws/sso/cache/");

    let directory = fs::read_dir(home_directory.as_path())
        .expect("Error locating aws credentials cache directory");

    for file_name in directory {
        let file_path = file_name.unwrap().path();

        match file_path.extension() {
            Some(ext) => {
                if ext != "json" {
                    continue;
                }
            }
            None => {
                continue;
            }
        }

        let mut file = File::open(file_path).expect("Error opening file");
        let mut data = String::new();
        file.read_to_string(&mut data).unwrap();

        let file_value: Value = serde_json::from_str(&data).expect("Error parsing file");
        let expiration_exists = file_value.get("accessToken");

        if let Some(_) = expiration_exists {
            let file_credentials: CachedCredentials =
                serde_json::from_str(&data).expect("Error parsing caches credentials");

            if let Some(current_credentials) = cached_credentials {
                if current_credentials.expires_at < file_credentials.expires_at {
                    cached_credentials = Some(file_credentials.to_owned());
                }
            }

            cached_credentials =
                Some(serde_json::from_str(&data).expect("Error parsing caches credentials"));
        }
    }

    if let Some(credentials) = cached_credentials {
        let exp_date = credentials.expires_at.expect("Missing expiration date");
        let today = Utc::now();

        if today > exp_date.and_utc() {
            panic!("Unable to use expired token");
        }

        let sso_config = aws_config::from_env()
            .region(Region::new(region.to_owned()))
            .load()
            .await;

        let sso_client = aws_sdk_sso::Client::new(&sso_config);

        let token = credentials
            .access_token
            .expect("Error accessing Access Token");

        let role_credentials_request = sso_client
            .get_role_credentials()
            .account_id(account_id)
            .role_name(role_name)
            .access_token(token)
            .send()
            .await;

        if let Ok(role_credentials) = role_credentials_request {
            let client_credentials = role_credentials
                .role_credentials()
                .expect("Error accessing role credentials");

            let sdk_config = aws_config::from_env()
                .credentials_provider(aws_sdk_dynamodb::config::Credentials::new(
                    client_credentials
                        .access_key_id()
                        .expect("Error accessing Account Key ID"),
                    client_credentials
                        .secret_access_key()
                        .expect("Error accessing Secret Access Key"),
                    Some(
                        client_credentials
                            .session_token()
                            .expect("Error accessing Session Token")
                            .to_string(),
                    ),
                    Some(exp_date.and_utc().into()),
                    "sso",
                ))
                .app_name(AppName::new("cf-dev-test").expect("valid app name"))
                .region(Region::new(region.to_owned()))
                .load()
                .await;

            return Ok(sdk_config);
        } else {
            panic!("Error retrieving role credentials")
        }
    } else {
        panic!("Error accessing parsed cached credentials");
    }
}
