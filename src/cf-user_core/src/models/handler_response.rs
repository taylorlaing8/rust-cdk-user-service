use lambda_http::http::StatusCode;

#[derive(Clone, Debug)]
pub struct HandleResponse {
	pub body: Option<String>,
	pub err_message: Option<String>,
	pub status_code: Option<StatusCode>,
}

impl HandleResponse {
	pub fn success(body: Option<&str>) -> Self {
		Self {
			body: match body {
				Some(body) => Some(body.to_string()),
				None => None	
			},
			err_message: None,
			status_code: None,
		}
	}

	pub fn set_success(body: Option<&str>, status_code: StatusCode) -> Self {
		Self {
			body: match body {
				Some(body) => Some(body.to_string()),
				None => None	
			},
			err_message: None,
			status_code: Some(status_code),
		}
	}

	pub fn error(err_message: Option<&str>) -> Self {
		Self {
			body: None,
			err_message:  match err_message {
				Some(err_message) => Some(err_message.to_string()),
				None => None	
			},
			status_code: None,
		}
	}

	pub fn set_error(err_message: Option<&str>, status_code: StatusCode) -> Self {
		Self {
			body: None,
			err_message:  match err_message {
				Some(err_message) => Some(err_message.to_string()),
				None => None	
			},
			status_code: Some(status_code),
		}
	}
}