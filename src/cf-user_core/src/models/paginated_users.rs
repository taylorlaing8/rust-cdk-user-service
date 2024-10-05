use crate::models::{paginated_result::PaginatedResult, user::User};
use serde_json::{Map, Value};

impl From<PaginatedResult<User>> for Option<String> {
    fn from(result: PaginatedResult<User>) -> Option<String> {
        let mut response = Map::new();

        response.insert(
            "data".to_string(),
            match serde_json::to_string(&result.to_owned().data) {
                Ok(data) => Value::String(data),
                Err(_err) => Value::Null,
            },
        );

        response.insert(
            "token".to_string(),
            match serde_json::to_string(&result.to_owned().token) {
                Ok(token) => Value::String(token),
                Err(_err) => Value::Null,
            },
        );

        return match serde_json::to_string(&response) {
            Ok(value) => Some(value),
            Err(_err) => None,
        };
    }
}
