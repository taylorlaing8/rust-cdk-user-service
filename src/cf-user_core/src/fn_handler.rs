use super::{dynamo, models::handler_response::HandleResponse};
use aws_sdk_dynamodb::Client;
use lambda_http::http::StatusCode;
use lambda_http::request::RequestContext;
use lambda_http::{Request, RequestExt, Response};
use serde_json::json;
use std::future::Future;
use tracing::info;

use super::models::permissions::Permission;

pub async fn handle_request<F, Fut>(
    event: Request,
    fn_handler: F,
    client: Option<Client>,
    permission: Permission,
) -> Result<Response<String>, Box<dyn std::error::Error>>
where
    F: FnOnce(Request, Client, String) -> Fut,
    Fut: Future<Output = Result<HandleResponse, Box<dyn std::error::Error>>>,
{
    let function_name = event.lambda_context().env_config.function_name;
    let app_stack = String::from(function_name.split("-app").collect::<Vec<&str>>()[0]) + "-app";
    let request_context = match event.request_context() {
        RequestContext::ApiGatewayV1(ctx) => ctx,
        _ => {
            panic!("Invalid Request Type")
        }
    };

    let req_data = json!({
        "domain_name": request_context.domain_name.unwrap_or("".to_string()),
        "function_name": function_name,
        "path": request_context.path.unwrap_or("".to_string()),
        "request_id": request_context.request_id.unwrap_or("".to_string()),
        "request_time": request_context.request_time.unwrap_or("".to_string()),
        "resource_path": request_context.resource_path.unwrap_or("".to_string()),
        "stage": request_context.stage.unwrap_or("".to_string()),
    })
    .to_string();

    info!("Request Data: {}", req_data);

    let authorizer_permissions = request_context
        .authorizer
        .clone()
        .get("permissions")
        .expect("Auth context missing permissions.")
        .to_owned();

    let user_permissions: Vec<&str> = serde_json::from_str(
        authorizer_permissions
            .as_str()
            .expect("User permissions are invalid type."),
    )
    .expect("Parsing user permissions failed");

    if !user_permissions.contains(&permission.value().as_str()) {
        return Ok(Response::builder()
            .status(StatusCode::FORBIDDEN)
            .header("Content-Type", "application/json")
            .body(
                json!({
                    "error": "User unauthorized to perform this action".to_string(),
                })
                .to_string(),
            )
            .map_err(Box::new)?);
    }

    let client = match client {
        Some(client) => client,
        None => match dynamo::get_client().await {
            Ok(client) => client,
            Err(_e) => {
                return Ok(Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .header("Content-Type", "application/json")
                    .body(
                        json!({
                            "error": "Error fetching DynamoDB Client".to_string(),
                        })
                        .to_string(),
                    )
                    .map_err(Box::new)?);
            }
        },
    };

    match fn_handler(event, client, app_stack).await {
        Ok(res) => {
            let status = match res.status_code.to_owned() {
                Some(status_code) => status_code,
                None => match res.err_message.to_owned() {
                    Some(_err) => StatusCode::BAD_REQUEST,
                    None => StatusCode::OK,
                },
            };

            if let 0..=299 = status.as_u16() {
                return match res.body.to_owned() {
                    Some(body) => Ok(Response::builder()
                        .status(status)
                        .header("Content-Type", "application/json")
                        .body(body.into())
                        .map_err(Box::new)?),
                    None => Ok(Response::builder()
                        .status(StatusCode::NO_CONTENT)
                        .header("Content-Type", "application/json")
                        .body("".into())
                        .map_err(Box::new)?),
                };
            } else {
                return match res.err_message.to_owned() {
                    Some(err) => Ok(Response::builder()
                        .status(status)
                        .header("Content-Type", "application/json")
                        .body(
                            json!({
                                "error": err.to_string(),
                            })
                            .to_string()
                            .into(),
                        )
                        .map_err(Box::new)?),
                    None => Ok(Response::builder()
                        .status(StatusCode::BAD_REQUEST)
                        .header("Content-Type", "application/json")
                        .body(
                            json!({
                                "error": "Unhandled Exception".to_string(),
                            })
                            .to_string()
                            .into(),
                        )
                        .map_err(Box::new)?),
                };
            }
        }
        Err(err) => {
            return Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .header("Content-Type", "application/json")
                .body(err.to_string().into())
                .map_err(Box::new)?);
        }
    }
}
