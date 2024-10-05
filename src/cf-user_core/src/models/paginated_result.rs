use super::super::error::Error;
use super::super::ext::AttributeValuesExt;
use aws_sdk_dynamodb::types::AttributeValue;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub trait EncodedToken {
    fn encode_token(&mut self) -> String;
    fn decode_token(token: String) -> PaginationToken;
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct PaginationToken {
    #[serde(rename = "PK")]
    pub pk: String,

    #[serde(rename = "SK")]
    pub sk: String,
}

impl EncodedToken for PaginationToken {
    fn encode_token(&mut self) -> String {
        let hex_pk = hex::encode(self.pk.to_owned());
        let hex_sk = hex::encode(self.sk.to_owned());

        return String::from(format!("{}.{}", hex_pk, hex_sk));
    }

    fn decode_token(token: String) -> PaginationToken {
        let key_parts = token.split(".").collect::<Vec<&str>>();

        let pk = String::from_utf8(hex::decode(key_parts[0]).expect("Error decoding primary key"))
            .expect("Error parsing token from decoded bytes");

        let sk =
            String::from_utf8(hex::decode(key_parts[1]).expect("Error decoding secondary key"))
                .expect("Error parsing token from decoded bytes");

        return PaginationToken { pk: pk, sk: sk };
    }
}

impl From<PaginationToken> for HashMap<String, AttributeValue> {
    fn from(token: PaginationToken) -> HashMap<String, AttributeValue> {
        let mut val = HashMap::new();
        val.insert(String::from("PK"), AttributeValue::S(token.pk.clone()));
        val.insert(String::from("SK"), AttributeValue::S(token.sk.clone()));

        val
    }
}

impl TryFrom<HashMap<String, AttributeValue>> for PaginationToken {
    type Error = Error;

    fn try_from(value: HashMap<String, AttributeValue>) -> Result<Self, Self::Error> {
        Ok(PaginationToken {
            pk: value
                .get_opt_s("PK")
                .ok_or(Error::InternalError("Missing PK"))?,

            sk: value
                .get_opt_s("SK")
                .ok_or(Error::InternalError("Missing SK"))?,
        })
    }
}

impl From<PaginationToken> for String {
    fn from(token: PaginationToken) -> String {
        return token.clone().encode_token();
    }
}

impl TryFrom<String> for PaginationToken {
    type Error = Error;

    fn try_from(token: String) -> Result<Self, Self::Error> {
        Ok(PaginationToken::decode_token(token.clone()))
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct PaginatedResult<T> {
    pub data: Vec<T>,
    pub token: Option<String>,
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn should_encode_token_using_same_pk_and_sk() {
        let token = PaginationToken {
            pk: "USER#123".to_string(),
            sk: "USER#123".to_string(),
        };

        let encoded_token = token.clone().encode_token();

        let encoded_token_parts = encoded_token.split('.').collect::<Vec<&str>>();
        assert_eq!(encoded_token_parts.len(), 2);
        assert_eq!(encoded_token_parts.get(0), encoded_token_parts.get(1));
    }

    #[test]
    fn should_encode_token_using_different_pk_and_sk() {
        let token = PaginationToken {
            pk: "USER#abc123".to_string(),
            sk: "USER#def456".to_string(),
        };

        let encoded_token = token.clone().encode_token();

        let encoded_token_parts = encoded_token.split('.').collect::<Vec<&str>>();
        assert_eq!(encoded_token_parts.len(), 2);
        assert_ne!(encoded_token_parts.get(0), encoded_token_parts.get(1));
    }

    #[test]
    fn should_decode_token_to_original_value() {
        let token = PaginationToken {
            pk: "USER#asu47thaskdf".to_string(),
            sk: "USER#badkehf847au".to_string(),
        };

        let encoded_token = token.clone().encode_token();

        let decoded_token = PaginationToken::decode_token(encoded_token.clone());
        assert_eq!(token.pk, decoded_token.pk);
        assert_eq!(token.sk, decoded_token.sk);
    }

    #[test]
    fn should_parse_encoded_token_from_valid_string() {
        let token = PaginationToken {
            pk: "USER#asu47thaskdf".to_string(),
            sk: "USER#badkehf847au".to_string(),
        };
        let encoded_token = token.clone().encode_token();

        let decoded_token_result = PaginationToken::try_from(encoded_token);
        let decoded_token = decoded_token_result.expect("Failed to parse token from string.");

        assert_eq!(token.pk, decoded_token.pk);
        assert_eq!(token.sk, decoded_token.sk);
    }

    #[test]
    fn should_return_encoded_string_from_pagination_token() {
        let token = PaginationToken {
            pk: "USER#asu47thaskdf".to_string(),
            sk: "USER#badkehf847au".to_string(),
        };

        let manually_encoded_token = token.clone().encode_token();

        let encoded_token_result = String::try_from(token);
        let encoded_token =
            encoded_token_result.expect("Failed to encode token through string overload.");

        assert_eq!(manually_encoded_token, encoded_token);
    }
}
