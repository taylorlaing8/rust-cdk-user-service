use std::collections::HashMap;

use aws_sdk_dynamodb::Client;
use cf_user_core::models::permissions::Permission;
use cf_user_core::{
    dynamo, fn_handler,
    models::{handler_response::HandleResponse, paginated_result::PaginatedResult, user::User},
};
use lambda_http::{http, run, service_fn, Body, Error, Request, RequestExt, Response};

async fn function_handler(
    event: Request,
    client: Client,
    stack_name: String,
) -> Result<HandleResponse, Box<dyn std::error::Error>> {
    let table_name = format!("{stack_name}-users");
    let mut limit: i32 = 25;
    let mut pagination_token: Option<&str> = None;

    let query_parameters = event.query_string_parameters_ref();

    if let Some(qp) = query_parameters {
        limit = qp
            .first("limit")
            .unwrap_or("25")
            .parse::<i32>()
            .expect("Error parsing limit");

        pagination_token = qp.first("paginationToken");
    }

    let paginated_users: PaginatedResult<User> =
        match dynamo::list_users(&client, &table_name, &limit, pagination_token).await {
            Ok(users) => users,
            Err(err) => {
                return Ok(HandleResponse::error(Some(
                    format!("Error serializing users: {}", err.to_string()).as_str(),
                )));
            }
        };

    return match serde_json::to_string(&paginated_users) {
        Ok(users) => Ok(HandleResponse::success(Some(users.as_str()))),
        Err(err) => Ok(HandleResponse::error(Some(
            format!("Error serializing user list: {}", err.to_string()).as_str(),
        ))),
    };
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
        fn_handler::handle_request(event, function_handler, None, Permission::UserList).await
    }))
    .await
}

#[tokio::test]
async fn list_users_should_succeed() {
    // let client = match dynamo::create_client().await {
    //     Ok(client) => client,
    //     Err(_e) => panic!("Failed to create client"),
    // };

    // let req = http::Request::builder().uri(format!(
    //     "https://dev-api.classifind.app/user/v1/users?limit=1"
    // ));

    // let result: Result<Response<String>, Box<dyn std::error::Error>> = fn_handler::handle_request(
    //     req.body(Body::Empty).unwrap(),
    //     function_handler,
    //     Some(client),
    // )
    // .await;

    // match result {
    //     Ok(response) => {
    //         let body = response.body().as_str();

    //         match serde_json::from_str::<PaginatedResult<User>>(body) {
    //             Ok(user) => {
    //                 // assert_eq!(user.user_id, user_id);
    //             }
    //             Err(_) => {}
    //         };
    //     }
    //     Err(_) => {}
    // }
}
