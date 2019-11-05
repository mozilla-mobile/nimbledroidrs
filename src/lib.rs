extern crate base64;

use reqwest::Url;
use serde_json::Value;

type ProfileResult = core::result::Result<Url, String>;

pub struct Profile {
	key: String,
	apk_path: String,
}

fn parse_response(response: String) -> Option<Url> {
	println!("response: {}", response);
	if let Ok(parsed_response) = serde_json::from_str::<Value>(&response) {
		println!("parsed_response: {}", parsed_response.to_string());
		let apk_url = parsed_response["apk_url"]
			.to_string()
			.trim_matches('\"')
			.to_string();
		match Url::parse(&apk_url) {
			Ok(apk_url) => {
				println!("url: {}", apk_url.to_string());
				Some(apk_url)
			}
			Err(e) => {
				println!("Url::parse error: {}", e.to_string());
				None
			}
		}
	} else {
		None
	}
}

impl Profile {
	pub fn new(key: &str, apk_path: &str) -> Self {
		Self {
			key: key.to_string(),
			apk_path: apk_path.to_string(),
		}
	}

	pub fn upload(&self) -> ProfileResult {
		/*
		 * Auth username is the api key. Don't forget to base64 encode.
		 * The body is a multipart/form-data
		 * The path is api/v2/apks
		 * The filedata is in the apk key
		 *
		 * The result seems to have some json fields and the important
		 * one is the apk_url.
		 */

		let form = reqwest::multipart::Form::new();
		let apk_part = reqwest::multipart::Part::file(self.apk_path.clone());
		if let Err(e) = apk_part {
			return Err(e.to_string());
		}
		let apk_part = apk_part.unwrap();
		let form = form.part("apk", apk_part);

		let upload_client = reqwest::Client::builder()
			.timeout(None)
			.gzip(false)
			.build()
			.unwrap();
		let upload_result = upload_client
			.post("https://nimbledroid.com/api/v2/apks")
			.header(
				reqwest::header::AUTHORIZATION,
				format!("Basic {}", base64::encode(&format!("{}:", self.key))),
			)
			.header(reqwest::header::HOST, "nimbledroid.com".to_string())
			.header(
				reqwest::header::USER_AGENT,
				"nimbledroidrs/0.0.1".to_string(),
			)
			.multipart(form)
			.send();

		match upload_result {
			Ok(mut o) => {
				if let Some(apk_url) = parse_response(o.text().unwrap()) {
					Ok(apk_url)
				} else {
					Err("Could not parse response.".to_string())
				}
			}
			Err(e) => Err(format!("Request to API failed: {}", e.to_string())),
		}
	}
}
