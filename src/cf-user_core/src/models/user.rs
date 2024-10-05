use super::super::error::Error;
use super::super::ext::AttributeValuesExt;
use aws_sdk_dynamodb::types::AttributeValue;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct User {
    #[serde(rename = "PK", skip)]
    pub pk: String,
    #[serde(rename = "SK", skip)]
    pub sk: String,
    #[serde(rename = "UserId")]
    pub user_id: String,
    #[serde(rename = "Username")]
    pub username: String,
    #[serde(rename = "FirstName")]
    pub first_name: Option<String>,
    #[serde(rename = "LastName")]
    pub last_name: Option<String>,
    #[serde(rename = "Email")]
    pub email: String,
    #[serde(rename = "ProfilePhoto")]
    pub profile_photo: Option<String>,
    #[serde(rename = "Summary")]
    pub summary: Option<String>,
    #[serde(rename = "PhoneNumber")]
    pub phone_number: Option<String>,
    #[serde(rename = "GSI1PK", skip)]
    pub gsi1pk: String,
    #[serde(rename = "GSI1SK", skip)]
    pub gsi1sk: String,
    #[serde(rename = "CreatedDate")]
    pub created_date: String,
    #[serde(rename = "UpdatedDate")]
    pub updated_date: String,
}

impl From<&User> for HashMap<String, AttributeValue> {
    fn from(user: &User) -> HashMap<String, AttributeValue> {
        let mut val = HashMap::new();
        val.insert("PK".to_owned(), AttributeValue::S(user.pk.clone()));
        val.insert("SK".to_owned(), AttributeValue::S(user.sk.clone()));
        val.insert("UserId".to_owned(), AttributeValue::S(user.user_id.clone()));
        val.insert(
            "Username".to_owned(),
            AttributeValue::S(user.username.clone()),
        );
        val.insert(
            "FirstName".to_owned(),
            match user.first_name.clone() {
                Some(first_name) => AttributeValue::S(first_name),
                None => AttributeValue::Null(true),
            },
        );
        val.insert(
            "LastName".to_owned(),
            match user.last_name.clone() {
                Some(last_name) => AttributeValue::S(last_name),
                None => AttributeValue::Null(true),
            },
        );
        val.insert("Email".to_owned(), AttributeValue::S(user.email.clone()));
        val.insert(
            "ProfilePhoto".to_owned(),
            match user.profile_photo.clone() {
                Some(profile_photo) => AttributeValue::S(profile_photo),
                None => AttributeValue::Null(true),
            },
        );
        val.insert(
            "Summary".to_owned(),
            match user.summary.clone() {
                Some(summary) => AttributeValue::S(summary),
                None => AttributeValue::Null(true),
            },
        );
        val.insert(
            "PhoneNumber".to_owned(),
            match user.phone_number.clone() {
                Some(phone_number) => AttributeValue::S(phone_number),
                None => AttributeValue::Null(true),
            },
        );
        val.insert("GSI1PK".to_owned(), AttributeValue::S(user.gsi1pk.clone()));
        val.insert("GSI1SK".to_owned(), AttributeValue::S(user.gsi1sk.clone()));
        val.insert(
            "CreatedDate".to_owned(),
            AttributeValue::S(user.created_date.clone()),
        );
        val.insert(
            "UpdatedDate".to_owned(),
            AttributeValue::S(user.updated_date.clone()),
        );

        val
    }
}

impl TryFrom<HashMap<String, AttributeValue>> for User {
    type Error = Error;

    fn try_from(value: HashMap<String, AttributeValue>) -> Result<Self, Self::Error> {
        Ok(User {
            pk: value
                .get_opt_s("PK")
                .ok_or(Error::InternalError("Missing PK"))?,
            sk: value
                .get_opt_s("SK")
                .ok_or(Error::InternalError("Missing SK"))?,
            user_id: value
                .get_opt_s("UserId")
                .ok_or(Error::InternalError("Missing User ID"))?,
            username: value.get_s("Username"),
            first_name: Some(value.get_s("FirstName")),
            last_name: Some(value.get_s("LastName")),
            email: value
                .get_opt_s("Email")
                .ok_or(Error::InternalError("Missing Email"))?,
            profile_photo: Some(value.get_s("ProfilePhoto")),
            summary: Some(value.get_s("Summary")),
            phone_number: Some(value.get_s("PhoneNumber")),
            gsi1pk: value
                .get_opt_s("GSI1PK")
                .ok_or(Error::InternalError("Missing GSI1PK"))?,
            gsi1sk: value
                .get_opt_s("GSI1SK")
                .ok_or(Error::InternalError("Missing GSI1SK"))?,
            created_date: value
                .get_opt_s("CreatedDate")
                .ok_or(Error::InternalError("Missing Created Date"))?,
            updated_date: value
                .get_opt_s("UpdatedDate")
                .ok_or(Error::InternalError("Missing Updated Date"))?,
        })
    }
}
