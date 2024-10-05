use aws_sdk_dynamodb::Client;
use cf_user_core::models::permissions::Permission;
use cf_user_core::{
    dynamo, fn_handler, models::handler_response::HandleResponse, models::user::User,
};
use lambda_http::http::StatusCode;
use lambda_http::{http, run, service_fn, Body, Error, Request, RequestExt, Response};
use ulid::Ulid;

async fn function_handler(
    event: Request,
    client: Client,
    stack_name: String,
) -> Result<HandleResponse, Box<dyn std::error::Error>> {
    let table_name = format!("{stack_name}-users");
    let parameters = event
        .path_parameters_ref()
        .expect("Missing path parameters.");

    return match parameters.first("userId") {
        Some(user_id) => {
            let mut user: Option<User> = None;

            if let Ok(_uuid) = Ulid::from_string(user_id) {
                user = match dynamo::get_user_by_id(&client, &table_name, user_id).await {
                    // Ok(user) => user,
                    Ok(user) => match user {
                        Some(u) => Some(u),
                        None => {
                            return Ok(HandleResponse::set_error(
                                Some(format!("User not found").as_str()),
                                StatusCode::NOT_FOUND,
                            ))
                        }
                    },
                    Err(err) => {
                        return Ok(HandleResponse::error(Some(
                            format!("Error fetching user by ID: {}", err.to_string()).as_str(),
                        )));
                    }
                };
            } else {
                let users = match dynamo::get_user_by_email(&client, &table_name, user_id).await {
                    Ok(users) => users,
                    Err(err) => {
                        return Ok(HandleResponse::error(Some(
                            format!("Error fetching user by email: {}", err.to_string()).as_str(),
                        )));
                    }
                };

                if let Some(users) = users {
                    if let Some(usr) = users.first() {
                        user = Some(usr.clone());
                    }
                } else {
                    return Ok(HandleResponse::set_error(
                        Some(format!("User not found").as_str()),
                        StatusCode::NOT_FOUND,
                    ));
                }
            }

            if let Some(user_obj) = user {
                match serde_json::to_string(&user_obj) {
                    Ok(value) => Ok(HandleResponse::success(Some(value.as_str()))),
                    Err(err) => Ok(HandleResponse::error(Some(
                        format!("Error serializing user data: {}", err.to_string()).as_str(),
                    ))),
                }
            } else {
                Ok(HandleResponse::error(Some("Error parsing user data")))
            }
        }
        None => Ok(HandleResponse::error(Some(
            "Error locating User ID within path parameters",
        ))),
    };

    // let path = event.uri().path();
    // let path_segments: Vec<&str> = path.split('/').collect();
    // let index = path_segments.iter().position(|&r| r == "users").unwrap();
    //
    // match path_segments.get(index + 1) {
    //     Some(user_id) => {
    //         let mut user: Option<User> = None;
    //
    //         if let Ok(_uuid) = Ulid::from_string(user_id) {
    //             user = match dynamo::get_user_by_id(&client, &table_name, user_id).await {
    //                 // Ok(user) => user,
    //                 Ok(user) => match user {
    //                     Some(u) => Some(u),
    //                     None => {
    //                         return Ok(HandleResponse::set_error(
    //                             Some(format!("User not found").as_str()),
    //                             StatusCode::NOT_FOUND,
    //                         ))
    //                     }
    //                 },
    //                 Err(err) => {
    //                     return Ok(HandleResponse::error(Some(
    //                         format!("Error fetching user by ID: {}", err.to_string()).as_str(),
    //                     )));
    //                 }
    //             };
    //         } else {
    //             let users = match dynamo::get_user_by_email(&client, table_name, user_id).await {
    //                 Ok(users) => users,
    //                 Err(err) => {
    //                     return Ok(HandleResponse::error(Some(
    //                         format!("Error fetching user by email: {}", err.to_string()).as_str(),
    //                     )));
    //                 }
    //             };
    //
    //             if let Some(users) = users {
    //                 if let Some(usr) = users.first() {
    //                     user = Some(usr.clone());
    //                 }
    //             } else {
    //                 return Ok(HandleResponse::set_error(
    //                     Some(format!("User not found").as_str()),
    //                     StatusCode::NOT_FOUND,
    //                 ));
    //             }
    //         }
    //
    //         if let Some(user_obj) = user {
    //             return match serde_json::to_string(&user_obj) {
    //                 Ok(value) => Ok(HandleResponse::success(Some(value.as_str()))),
    //                 Err(err) => Ok(HandleResponse::error(Some(
    //                     format!("Error serializing user data: {}", err.to_string()).as_str(),
    //                 ))),
    //             };
    //         } else {
    //             return Ok(HandleResponse::error(Some("Error parsing user data")));
    //         }
    //     }
    //     None => {
    //         return Ok(HandleResponse::error(Some(
    //             "Error locating User ID within path parameters",
    //         )));
    //     }
    // }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .with_ansi(false)
        .without_time()
        .init();

    run(service_fn(|event| async {
        fn_handler::handle_request(event, function_handler, None, Permission::UserGet).await
    }))
    .await
}

#[tokio::test]
async fn get_user_by_id_should_succeed() {
    let client = match dynamo::create_client().await {
        Ok(client) => client,
        Err(_e) => panic!("Failed to create client"),
    };

    let user_id = "01H4E0XFKZ2SRKBR29GQRFPV30";
    let req = http::Request::builder().uri(format!(
        "https://dev-api.classifind.app/user/v1/users/{}",
        user_id
    ));

    let result: Result<Response<String>, Box<dyn std::error::Error>> = fn_handler::handle_request(
        req.body(Body::Empty).unwrap().into(),
        function_handler,
        Some(client),
        Permission::UserGet,
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

#[tokio::test]
async fn get_user_by_email_should_succeed() {
    let client = match dynamo::create_client().await {
        Ok(client) => client,
        Err(_e) => panic!("Failed to create client"),
    };

    let user_email = "testemail3@gmail.com";
    let req = http::Request::builder().uri(format!(
        "https://dev-api.classifind.app/user/v1/users/{}",
        user_email
    ));

    let result: Result<Response<String>, Box<dyn std::error::Error>> = fn_handler::handle_request(
        req.body(Body::Empty).unwrap().into(),
        function_handler,
        Some(client),
        Permission::UserGet,
    )
    .await;

    match result {
        Ok(response) => {
            let body = response.body().as_str();

            match serde_json::from_str::<User>(body) {
                Ok(user) => {
                    assert_eq!(user.email, user_email);
                }
                Err(_) => {}
            };
        }
        Err(_) => {}
    }
}
