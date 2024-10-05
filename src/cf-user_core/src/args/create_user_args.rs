use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct CreateUserArgs {
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
}