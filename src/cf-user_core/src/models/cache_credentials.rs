use chrono::{NaiveDateTime, TimeZone, Utc};
use serde::{self, Deserialize, Deserializer, Serialize, Serializer};

#[derive(Clone, PartialEq, Deserialize, Serialize, Debug)]
pub struct CachedCredentials {
    #[serde(rename = "accessToken")]
    pub access_token: Option<String>,

    #[serde(rename = "clientId")]
    pub client_id: String,

    #[serde(rename = "clientSecret")]
    pub client_secret: String,

    #[serde(rename = "expiresAt", with = "option_date_format")]
    pub expires_at: Option<NaiveDateTime>,
}

mod option_date_format {
    use super::*;

    const FORMAT: &'static str = "%Y-%m-%dT%H:%M:%S%fZ";

    pub fn serialize<S>(date: &Option<NaiveDateTime>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match date {
            Some(date) => {
                let s = format!("{}", date.format(FORMAT));
                return serializer.serialize_str(&s);
            }
            _ => unreachable!(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<NaiveDateTime>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        // let d = Utc.datetime_from_str(&s, FORMAT);
        let d = NaiveDateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M:%SZ");

        return match d.map_err(serde::de::Error::custom) {
            Ok(date) => Ok(Some(date)),
            Err(err) => Err(err),
        };
    }
}
