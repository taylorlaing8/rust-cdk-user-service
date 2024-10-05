use aws_sdk_dynamodb::Client;
use cf_user_core::models::permissions::Permission;
use cf_user_core::{
    args::update_user_args::UpdateUserArgs, dynamo, fn_handler,
    models::handler_response::HandleResponse, models::user::User,
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

    let body = event.body();
    let s = std::str::from_utf8(body).expect("invalid utf-8 sequence");

    let item = match serde_json::from_str::<UpdateUserArgs>(s) {
        Ok(item) => item,
        Err(_err) => {
            return Ok(HandleResponse::error(Some(
                "Error parsing incoming request object",
            )));
        }
    };

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

            match dynamo::update_user(&client, &table_name, user_id, item.clone()).await {
                Ok(_success) => {
                    return Ok(HandleResponse::success(None));
                }
                Err(err) => {
                    return Ok(HandleResponse::error(Some(
                        format!("Error updating user by ID: {}", err.to_string()).as_str(),
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
        fn_handler::handle_request(event, function_handler, None, Permission::UserUpdate).await
    }))
    .await
}

#[tokio::test]
async fn update_user_by_id_should_succeed() {
    let client = match dynamo::create_client().await {
        Ok(client) => client,
        Err(_e) => panic!("Failed to create client"),
    };

    let user_request = json!({
        "Username": "taylorlaing8",
        "FirstName": "Taylor",
        "LastName": "Laing",
        "Email": "taylorlaing8@gmail.com",
        "PhoneNumber": "8013911705",
        "ProfilePhoto": "https://laingdevelopment.net/wp-content/uploads/2020/12/AAEAAQAAAAAAAAhrAAAAJDU0YzUxYzFlLTFjYzUtNDM4NC1iOWUzLTE4ZTk4NGU5ODk0YQ-1.jpg",
        "Summary": "UPDATE: Bruh this is gonna take forever..."
    });

    let user_id = "01H2PFJW211F4A7D98DS551H2M";
    let req = http::Request::builder().uri(format!(
        "https://dev-api.classifind.app/user/v1/users/{}",
        user_id
    ));

    let result: Result<Response<String>, Box<dyn std::error::Error>> = fn_handler::handle_request(
        req.body(user_request.to_string().into()).unwrap(),
        function_handler,
        Some(client),
        Permission::UserUpdate,
    )
    .await;

    match result {
        Ok(response) => {
            let body = response.body().as_str();

            match serde_json::from_str::<User>(body) {
                Ok(user) => {
                    assert_eq!(user.email, "taylorlaing8@gmail.com");
                }
                Err(_) => {}
            };
        }
        Err(_) => {}
    }
}
