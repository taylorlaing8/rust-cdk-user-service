use crate::models::paginated_result::{EncodedToken, PaginationToken};

use super::{
    args::{create_user_args::CreateUserArgs, update_user_args::UpdateUserArgs},
    aws_config_loader::create_mock_config,
    models::paginated_result::PaginatedResult,
    models::user::User,
};

use aws_sdk_dynamodb::types::{AttributeAction, AttributeValue, AttributeValueUpdate};
use aws_sdk_dynamodb::{Client, Error};
use chrono;
use std::collections::HashMap;
use ulid::Ulid;

static GSI_1_INDEX: &'static str = "GSI1";

pub async fn create_client() -> Result<Client, Error> {
    let account_id = "584620395262";
    let region = "us-west-2";
    let role_name = "DeveloperAccess";

    let sdk_config = create_mock_config(account_id, region, role_name).await;

    if let Ok(config) = sdk_config {
        let client = Client::new(&config);
        return Ok(client);
    } else {
        panic!("Error accessing mock aws config")
    }
}

pub async fn get_client() -> Result<Client, Error> {
    let config = aws_config::load_from_env().await;
    let client = Client::new(&config);

    return Ok(client);
}

pub async fn get_user_by_id(
    client: &Client,
    table: &str,
    user_id: &str,
) -> Result<Option<User>, Box<dyn std::error::Error>> {
    let pk: AttributeValue = AttributeValue::S(String::from("USER#") + user_id);
    let sk: AttributeValue = AttributeValue::S(String::from("USER#") + user_id);

    let request = client
        .get_item()
        .table_name(table)
        .key("PK", pk)
        .key("SK", sk);

    let resp = request.send().await?;

    if let Some(item) = resp.item {
        let user: User = User::try_from(item.clone())?;

        return Ok(Some(user));
    }

    return Ok(None);
}

pub async fn get_user_by_email(
    client: &Client,
    table: &str,
    email: &str,
) -> Result<Option<Vec<User>>, Box<dyn std::error::Error>> {
    let gsi1pk: AttributeValue = AttributeValue::S(String::from("EMAIL#") + email);
    let gsi1sk: AttributeValue = AttributeValue::S(String::from("USERNAME#"));

    let mut expression_values: HashMap<String, AttributeValue> = HashMap::new();
    expression_values.insert(":gsi1pk".to_string(), gsi1pk);
    expression_values.insert(":gsi1sk".to_string(), gsi1sk);

    let request = client
        .query()
        .index_name(GSI_1_INDEX.to_string())
        .table_name(table)
        .set_key_condition_expression(Some(
            "GSI1PK = :gsi1pk and begins_with(GSI1SK, :gsi1sk)".to_string(),
        ))
        .set_expression_attribute_values(Some(expression_values));

    let resp = request.send().await?;

    if resp.count <= 0 {
        return Ok(None);
    }

    let items = resp.items;
    if let Some(items) = items {
        let mut users: Vec<User> = Vec::new();

        for item in items.iter() {
            let user: User = User::try_from(item.clone())?;
            users.push(user);
        }

        return Ok(Some(users));
    }

    return Ok(None);
}

pub async fn create_user(
    client: &Client,
    table: &str,
    input: CreateUserArgs,
) -> Result<String, Box<dyn std::error::Error>> {
    let user_id = Ulid::new().to_string();
    let current_date = chrono::offset::Utc::now().to_string();

    let mut insert_map: HashMap<String, AttributeValue> = HashMap::new();

    insert_map.insert(
        "PK".to_owned(),
        AttributeValue::S(String::from("USER#") + user_id.to_owned().as_str()),
    );
    insert_map.insert(
        "SK".to_owned(),
        AttributeValue::S(String::from("USER#") + user_id.to_owned().as_str()),
    );
    insert_map.insert("UserId".to_owned(), AttributeValue::S(user_id.to_owned()));
    insert_map.insert(
        "Username".to_owned(),
        AttributeValue::S(input.username.to_owned()),
    );
    insert_map.insert(
        "Email".to_owned(),
        AttributeValue::S(input.email.to_owned()),
    );
    insert_map.insert(
        "CreatedDate".to_owned(),
        AttributeValue::S(current_date.clone()),
    );
    insert_map.insert(
        "UpdatedDate".to_owned(),
        AttributeValue::S(current_date.clone()),
    );
    insert_map.insert(
        "GSI1PK".to_owned(),
        AttributeValue::S(String::from("EMAIL#") + input.email.to_owned().as_str()),
    );
    insert_map.insert(
        "GSI1SK".to_owned(),
        AttributeValue::S(String::from("USERNAME#") + input.username.to_owned().as_str()),
    );

    if let Some(first_name) = input.first_name {
        insert_map.insert(
            "FirstName".to_owned(),
            AttributeValue::S(first_name.to_owned()),
        );
    }
    if let Some(last_name) = input.last_name {
        insert_map.insert(
            "LastName".to_owned(),
            AttributeValue::S(last_name.to_owned()),
        );
    }
    if let Some(profile_photo) = input.profile_photo {
        insert_map.insert(
            "ProfilePhoto".to_owned(),
            AttributeValue::S(profile_photo.to_owned()),
        );
    }
    if let Some(summary) = input.summary {
        insert_map.insert("Summary".to_owned(), AttributeValue::S(summary.to_owned()));
    }
    if let Some(phone_number) = input.phone_number {
        insert_map.insert(
            "PhoneNumber".to_owned(),
            AttributeValue::S(phone_number.to_owned()),
        );
    }

    let request = client
        .put_item()
        .table_name(table)
        .set_item(Some(insert_map));

    let _resp = request.send().await?;

    Ok(user_id)
}

pub async fn update_user(
    client: &Client,
    table: &str,
    user_id: &str,
    input: UpdateUserArgs,
) -> Result<bool, Box<dyn std::error::Error>> {
    let current_date = chrono::offset::Utc::now().to_string();

    let pk: AttributeValue = AttributeValue::S(String::from("USER#") + user_id);
    let sk: AttributeValue = AttributeValue::S(String::from("USER#") + user_id);

    let mut update_map: HashMap<String, AttributeValueUpdate> = HashMap::new();

    update_map.insert(
        "Username".to_owned(),
        AttributeValueUpdate::builder()
            .action(AttributeAction::Put)
            .value(AttributeValue::S(input.username.to_owned()))
            .build(),
    );
    update_map.insert(
        "FirstName".to_owned(),
        match input.first_name {
            Some(first_name) => AttributeValueUpdate::builder()
                .action(AttributeAction::Put)
                .value(AttributeValue::S(first_name))
                .build(),
            None => AttributeValueUpdate::builder()
                .action(AttributeAction::Delete)
                .build(),
        },
    );
    update_map.insert(
        "LastName".to_owned(),
        match input.last_name {
            Some(last_name) => AttributeValueUpdate::builder()
                .action(AttributeAction::Put)
                .value(AttributeValue::S(last_name))
                .build(),
            None => AttributeValueUpdate::builder()
                .action(AttributeAction::Delete)
                .build(),
        },
    );
    update_map.insert(
        "Email".to_owned(),
        AttributeValueUpdate::builder()
            .action(AttributeAction::Put)
            .value(AttributeValue::S(input.email.to_owned()))
            .build(),
    );
    update_map.insert(
        "ProfilePhoto".to_owned(),
        match input.profile_photo {
            Some(profile_photo) => AttributeValueUpdate::builder()
                .action(AttributeAction::Put)
                .value(AttributeValue::S(profile_photo))
                .build(),
            None => AttributeValueUpdate::builder()
                .action(AttributeAction::Delete)
                .build(),
        },
    );
    update_map.insert(
        "Summary".to_owned(),
        match input.summary {
            Some(summary) => AttributeValueUpdate::builder()
                .action(AttributeAction::Put)
                .value(AttributeValue::S(summary))
                .build(),
            None => AttributeValueUpdate::builder()
                .action(AttributeAction::Delete)
                .build(),
        },
    );
    update_map.insert(
        "PhoneNumber".to_owned(),
        match input.phone_number {
            Some(phone_number) => AttributeValueUpdate::builder()
                .action(AttributeAction::Put)
                .value(AttributeValue::S(phone_number))
                .build(),
            None => AttributeValueUpdate::builder()
                .action(AttributeAction::Delete)
                .build(),
        },
    );
    update_map.insert(
        "UpdatedDate".to_owned(),
        AttributeValueUpdate::builder()
            .action(AttributeAction::Put)
            .value(AttributeValue::S(current_date.clone()))
            .build(),
    );
    update_map.insert(
        "GSI1PK".to_owned(),
        AttributeValueUpdate::builder()
            .action(AttributeAction::Put)
            .value(AttributeValue::S(
                String::from("EMAIL#") + input.email.to_owned().as_str(),
            ))
            .build(),
    );
    update_map.insert(
        "GSI1SK".to_owned(),
        AttributeValueUpdate::builder()
            .action(AttributeAction::Put)
            .value(AttributeValue::S(
                String::from("USERNAME#") + input.username.to_owned().as_str(),
            ))
            .build(),
    );

    let request = client
        .update_item()
        .key("PK", pk)
        .key("SK", sk)
        .table_name(table)
        .set_attribute_updates(Some(update_map));

    let _resp = request.send().await?;

    return Ok(true);
}

pub async fn delete_user(
    client: &Client,
    table: &str,
    user_id: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    let pk: AttributeValue = AttributeValue::S(String::from("USER#") + user_id);
    let sk: AttributeValue = AttributeValue::S(String::from("USER#") + user_id);

    let request = client
        .delete_item()
        .key("PK", pk)
        .key("SK", sk)
        .table_name(table);

    let _resp = request.send().await?;

    return Ok(true);
}

// NOTE: WILL NOT WORK. No secondary indexes on users to list them
pub async fn list_users(
    client: &Client,
    table: &str,
    limit: &i32,
    pagination_token: Option<&str>,
) -> Result<PaginatedResult<User>, Box<dyn std::error::Error>> {
    let pk: AttributeValue = AttributeValue::S(String::from("USER#"));
    let sk: AttributeValue = AttributeValue::S(String::from("USER#"));

    let mut expression_values: HashMap<String, AttributeValue> = HashMap::new();
    expression_values.insert(":pk".to_string(), pk);
    expression_values.insert(":sk".to_string(), sk);

    let start_key: Option<HashMap<String, AttributeValue>> = match pagination_token {
        Some(token) => Some(PaginationToken::decode_token(token.to_owned().clone()).into()),
        None => None,
    };

    let request = client
        .query()
        .table_name(table)
        .limit(limit.to_owned())
        .set_exclusive_start_key(start_key)
        .set_key_condition_expression(Some("PK = :pk and begins_with(SK, :sk)".to_string()))
        .set_expression_attribute_values(Some(expression_values));

    let resp = request.send().await?;
    let items = resp.items;

    if let Some(items) = items {
        let mut users: Vec<User> = Vec::new();
        let last_key: Option<HashMap<String, AttributeValue>> = resp.last_evaluated_key;

        for item in items.iter() {
            let user: User = User::try_from(item.clone())?;
            users.push(user);
        }

        let paginated_result: PaginatedResult<User> = PaginatedResult {
            data: users,
            token: match last_key {
                Some(key) => match PaginationToken::try_from(key) {
                    Ok(token) => Some(token.into()),
                    Err(err) => return Err(Box::new(err)),
                },
                None => None,
            },
        };

        return Ok(paginated_result);
    } else {
        panic!("Error parsing returned users");
    }
}

#[tokio::test]
async fn should_get_user_by_id() {
    let client = create_client().await.expect("Error retrieivng client.");
    let table_name = "cf-user-dev-app-users";
    let user_id = "01H4E0XFKZ2SRKBR29GQRFPV30";

    let response: Option<User> = get_user_by_id(&client, table_name, user_id)
        .await
        .expect("Unable to retrieve user by ID");

    let user = response.unwrap();
    assert_eq!(user.user_id, user_id);
}

#[tokio::test]
async fn should_list_users() {
    // BELOW WORKS! TOKEN SENT BACK, PARSED TO STRING, SENT TO FUNCTION, AND CORRECTLY PAGINATED THROUGH THE WHILE LOOP :)))))
    let client = create_client().await.expect("Error retrieivng client.");
    let table_name = "cf-user-dev-app-users";
    let limit = 1;

    let response: PaginatedResult<User> = list_users(&client, table_name, &limit, None)
        .await
        .expect("Unable to retrieve user by ID");

    let data = response;
    let test = "";

    let mut token = data.token;
    while let Some(t) = token.to_owned() {
        // let test = match serde_json::to_string(&t) {
        //     Ok(token_string) => Some(token_string),
        //     Err(_err) => None,
        // };

        let response: PaginatedResult<User> =
            list_users(&client, table_name, &limit, Some(t.as_str()))
                .await
                .expect("Unable to retrieve user by ID");

        let data = response;
        let test = "";

        token = data.token;
    }
    // assert_eq!(data.token, user_id);
}
