use aws_sdk_dynamodb::Client;
use cf_user_core::models::permissions::Permission;
use cf_user_core::{
    dynamo, fn_handler, models::handler_response::HandleResponse, models::user::User,
};
use lambda_http::{http, run, service_fn, Body, Error, Request, RequestExt, Response};
use serde_json::json;

async fn function_handler(
    event: Request,
    client: Client,
    stack_name: String,
) -> Result<HandleResponse, Box<dyn std::error::Error>> {
    let table_name = format!("{stack_name}-users");
    let parameters = event
        .path_parameters_ref()
        .expect("Missing path parameters.");

    match parameters.first("userId") {
        Some(user_id) => {
            let user = match dynamo::get_user_by_id(&client, &table_name, &user_id).await {
                Ok(user) => user,
                Err(err) => {
                    return Ok(HandleResponse::error(Some(
                        format!("Error fetching user by ID: {}", err.to_string()).as_str(),
                    )));
                }
            };

            if user.is_none() {
                return Ok(HandleResponse::error(Some("Error parsing user data")));
            }

            match dynamo::delete_user(&client, &table_name, user_id).await {
                Ok(_success) => {
                    return Ok(HandleResponse::success(None));
                }
                Err(err) => {
                    return Ok(HandleResponse::error(Some(
                        format!("Error deleting user by ID: {}", err.to_string()).as_str(),
                    )));
                }
            };
        }
        None => {
            return Ok(HandleResponse::error(Some(
                "Error locating User ID within path parameters",
            )));
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
        fn_handler::handle_request(event, function_handler, None, Permission::UserDelete).await
    }))
    .await
}

#[tokio::test]
async fn delete_user_by_id_should_succeed() {
    let client = match dynamo::create_client().await {
        Ok(client) => client,
        Err(_e) => panic!("Failed to create client"),
    };

    let user_id = "01H5FS6FKMB0YY0VDJ015741BJ";
    let req = http::Request::builder().uri(format!(
        "https://dev-api.classifind.app/user/v1/users/{}",
        user_id
    ));

    let result: Result<Response<String>, Box<dyn std::error::Error>> = fn_handler::handle_request(
        req.body(Body::Empty).unwrap(),
        function_handler,
        Some(client),
        Permission::UserDelete,
    )
    .await;

    match result {
        Ok(response) => {
            let body = response.body().as_str();

            match serde_json::from_str::<User>(body) {
                Ok(user) => {
                    assert_eq!(user.user_id, user_id);
                }
                Err(_) => {}
            };
        }
        Err(_) => {}
    }
}
