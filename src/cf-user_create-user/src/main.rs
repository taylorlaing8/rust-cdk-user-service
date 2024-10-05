use aws_sdk_dynamodb::Client;
use cf_user_core::models::permissions::Permission;
use cf_user_core::{
    args::create_user_args::CreateUserArgs, dynamo, fn_handler,
    models::handler_response::HandleResponse, models::user::User,
};
use lambda_http::{http, http::StatusCode, run, service_fn, Body, Error, Request, Response};
use serde_json::json;
use tracing::info;

async fn function_handler(
    event: Request,
    client: Client,
    stack_name: String,
) -> Result<HandleResponse, Box<dyn std::error::Error>> {
    let table_name = format!("{stack_name}-users");

    let body = event.body();
    let s = std::str::from_utf8(body).expect("invalid utf-8 sequence");

    info!("Create New User: {}", s);

    let item = match serde_json::from_str::<CreateUserArgs>(s) {
        Ok(item) => item,
        Err(_err) => {
            return Ok(HandleResponse::error(Some(
                "Error parsing incoming request object",
            )));
        }
    };

    if let Ok(result) = dynamo::get_user_by_email(&client, &table_name, &item.email).await {
        if let Some(users) = result {
            if users.len() > 0 {
                match users.first() {
                    Some(user) => {
                        return Ok(HandleResponse::set_error(
                            Some(
                                format!(
                                "User record exists with matching email address {{ UserID: {} }}",
                                user.user_id
                            )
                                .as_str(),
                            ),
                            StatusCode::CONFLICT,
                        ));
                    }
                    None => {
                        return Ok(HandleResponse::set_error(
                            Some("User record exists with matching email address"),
                            StatusCode::CONFLICT,
                        ));
                    }
                }
            }
        }
    }

    let user_id = match dynamo::create_user(&client, &table_name, item.clone()).await {
        Ok(user_id) => user_id,
        Err(err) => {
            return Ok(HandleResponse::error(Some(
                format!("Error creating user: {}", err.to_string()).as_str(),
            )));
        }
    };

    let user = match dynamo::get_user_by_id(&client, &table_name, &user_id).await {
        Ok(user) => user,
        Err(err) => {
            return Ok(HandleResponse::error(Some(
                format!("Error fetching user by ID: {}", err.to_string()).as_str(),
            )));
        }
    };

    match user {
        Some(user) => {
            let user_string = match serde_json::to_string(&user) {
                Ok(user_string) => user_string,
                Err(err) => {
                    return Ok(HandleResponse::error(Some(
                        format!("Error serializing user data: {}", err.to_string()).as_str(),
                    )));
                }
            };

            return Ok(HandleResponse::success(Some(user_string.as_str())));
        }
        None => {
            return Ok(HandleResponse::error(Some("Error parsing user data")));
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .with_ansi(false)
        .without_time()
        .init();

    run(service_fn(|event: Request| async {
        fn_handler::handle_request(event, function_handler, None, Permission::UserCreate).await
    }))
    .await
}

#[tokio::test]
async fn create_new_user_should_succeed() {
    let client = match dynamo::create_client().await {
        Ok(client) => client,
        Err(_e) => panic!("Failed to create client"),
    };

    let user_request = json!({
        "Username": "taylorlaing121234",
        "FirstName": "Taylor",
        "LastName": "Laing",
        "Email": "taylorlaing121234@gmail.com",
        "PhoneNumber": "8013911705",
        "ProfilePhoto": "https://laingdevelopment.net/wp-content/uploads/2020/12/AAEAAQAAAAAAAAhrAAAAJDU0YzUxYzFlLTFjYzUtNDM4NC1iOWUzLTE4ZTk4NGU5ODk0YQ-1.jpg",
        "Summary": "Bruh this is gonna take forever..."
    });

    let req = http::Request::builder().uri("https://dev-api.classifind.app/user/v1/users");

    let result: Result<Response<String>, Box<dyn std::error::Error>> = fn_handler::handle_request(
        req.body(user_request.to_string().into()).unwrap(),
        function_handler,
        Some(client),
        Permission::UserCreate,
    )
    .await;

    match result {
        Ok(response) => {
            let body = response.body().as_str();

            match serde_json::from_str::<User>(body) {
                Ok(user) => {
                    assert_eq!(user.email, "taylorlaing121234@gmail.com");
                }
                Err(_) => {}
            };
        }
        Err(_) => {}
    }
}
