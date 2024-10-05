use aws_sdk_dynamodb::types::AttributeValue;
use std::collections::HashMap;

pub trait AttributeValuesExt {
    fn get_s(&self, key: &str) -> String;
    fn get_opt_s(&self, key: &str) -> Option<String>;
    fn get_n(&self, key: &str) -> Option<f64>;
}

impl AttributeValuesExt for HashMap<String, AttributeValue> {
	fn get_s(&self, key: &str) -> String {		
		match self.get_opt_s(key) {
			None => {
				return "".to_string();
			}
			Some(v) => {
				return v;
			}
		}
	}
    fn get_opt_s(&self, key: &str) -> Option<String> {
        Some(self.get(key)?.as_s().ok()?.to_owned())
    }
    fn get_n(&self, key: &str) -> Option<f64> {
        self.get(key)?.as_n().ok()?.parse::<f64>().ok()
    }
}
