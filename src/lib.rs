extern crate base64;

use reqwest::Url;
use serde::Deserialize;
use serde_json::Value;
use std::fmt;
use std::fmt::Display;
use std::time::Duration;

pub type ProfileUploadResult = core::result::Result<Url, String>;
pub type ProfileStatusResult = core::result::Result<ProfileStatus, String>;

pub struct Profiler {
	key: String,
	apk_path: String,
}

pub struct ProfileResult {
	status: ProfileStatus,
	pub profiles: Vec<Profile>,
}

impl Display for ProfileResult {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "status: {}\n", self.status);
		for p in &self.profiles {
			write!(f, "{}\n", p);
		}
		write!(f, "\n")
	}
}

pub enum ProfileStatus {
	Crawling,
	Pending,
	Complete,
	Failed,
	Error,
}

#[derive(Deserialize)]
pub struct Profile {
	pub scenario_name: String,
	pub time_in_ms: u64,
	pub profile_url: String,
}

#[allow(dead_code)]
impl<'a> Profile {
	pub fn get_scenario_name(&'a self) -> &'a String {
		&self.scenario_name
	}
	pub fn get_time_in_ms(&'a self) -> u64 {
		self.time_in_ms
	}

}

impl Display for Profile {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "scenario_name: {}\n", self.scenario_name);
		write!(f, "time_in_ms: {}\n", self.time_in_ms);
		write!(f, "profile_url: {}\n", self.profile_url)
	}
}
pub struct ProfileScenarios {
	scenarios: Vec<ProfileScenario>,
}

#[allow(dead_code)]
impl<'a> ProfileScenarios {
	pub fn get_scenarios(&'a self) -> &'a Vec<ProfileScenario> {
		&self.scenarios
	}
}

#[derive(Deserialize)]
pub struct ProfileScenario {
	name: String,
	time: u64,
	screenshots: Vec<String>,
	thumbnail_screenshots: Vec<String>,
}

#[allow(dead_code)]
impl<'a> ProfileScenario {
	pub fn get_name(&'a self) -> &'a String {
		&self.name
	}

	pub fn get_time(&self) -> Duration {
		Duration::from_millis(self.time)
	}
}

impl Display for ProfileScenario {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let mut result = write!(f, "name: {}\n", self.name);
		result = result.or(write!(f, "time: {}\n", self.time));

		result = result.or(write!(f, "Screenshots:\n"));
		for s in &self.screenshots {
			result = result.or(write!(f, "{}\n", s));
		}
		result = result.or(write!(f, "Thumbnail Screenshots:\n"));
		for s in &self.thumbnail_screenshots {
			result = result.or(write!(f, "{}\n", s));
		}
		result
	}
}
impl Display for ProfileStatus {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match *self {
			ProfileStatus::Crawling => write!(f, "Crawling"),
			ProfileStatus::Pending => write!(f, "Pending"),
			ProfileStatus::Complete => write!(f, "Complete"),
			ProfileStatus::Failed => write!(f, "Failed"),
			ProfileStatus::Error => write!(f, "Error"),
		}
	}
}
#[allow(non_snake_case)]
fn convert_Value_to_ProfileStatus(v: &Value) -> ProfileStatus {
	let status = &v["status"];
	if status == "Pending" {
		ProfileStatus::Pending
	} else if status == "Crawling" {
		ProfileStatus::Crawling
	} else if status == "Complete" {
		ProfileStatus::Complete
	} else if status == "Failed" {
		ProfileStatus::Failed
	} else {
		ProfileStatus::Error
	}
}

#[allow(non_snake_case)]
fn convert_Value_to_ProfileUploadResult(v: &Value) -> ProfileUploadResult {
	let apk_url = v["apk_url"].to_string().trim_matches('\"').to_string();
	match Url::parse(&apk_url) {
		Ok(apk_url) => ProfileUploadResult::Ok(apk_url),
		Err(e) => ProfileUploadResult::Err(e.to_string()),
	}
}

const ND_UPLOAD_URL: &str = "https://nimbledroid.com/api/v2/apks";

impl Profiler {
	pub fn new(key: &str, apk_path: &str) -> Self {
		Self {
			key: key.to_string(),
			apk_path: apk_path.to_string(),
		}
	}

	pub fn get_profile_result(&self, upload_url: &Url) -> Option<ProfileResult> {
		let get_profile_client = reqwest::Client::builder()
			.timeout(None)
			.gzip(false)
			.build()
			.unwrap();
		let mut profile_result: reqwest::Response;

		match get_profile_client
			.get(&upload_url.to_string())
			.header(
				reqwest::header::AUTHORIZATION,
				format!("Basic {}", base64::encode(&format!("{}:", self.key))),
			)
			.header(reqwest::header::HOST, "nimbledroid.com".to_string())
			.header(
				reqwest::header::USER_AGENT,
				"nimbledroidrs/0.0.1".to_string(),
			)
			.send()
		{
			Ok(response) => profile_result = response,
			Err(e) => return None,
		}
		let profile_result = profile_result.text().unwrap();

		if let Ok(profile_result) = serde_json::from_str::<Value>(&profile_result) {
			let profile_result_status = convert_Value_to_ProfileStatus(&profile_result);
			match profile_result_status {
				ProfileStatus::Error => Some(ProfileResult {
					status: ProfileStatus::Error,
					profiles: Vec::<Profile>::new(),
				}),
				s @ ProfileStatus::Failed | s @ ProfileStatus::Complete => {
					let mut result_profiles: Vec<Profile> = Vec::<Profile>::new();
					if let Ok(profiles) =
						serde_json::from_value::<Vec<Profile>>(profile_result["profiles"].clone())
					{
						result_profiles = profiles;
					}
					Some(ProfileResult {
						status: s,
						profiles: result_profiles,
					})
				}
				_ => Some(ProfileResult {
					status: ProfileStatus::Error,
					profiles: Vec::<Profile>::new(),
				}),
			}
		} else {
			None
		}
	}

	pub fn get_profile_scenarios(&self, upload_url: &Url) -> Option<ProfileScenarios> {
		let get_profile_client = reqwest::Client::builder()
			.timeout(None)
			.gzip(false)
			.build()
			.unwrap();
		let mut profile_result: reqwest::Response;

		match get_profile_client
			.get(&upload_url.to_string())
			.header(
				reqwest::header::AUTHORIZATION,
				format!("Basic {}", base64::encode(&format!("{}:", self.key))),
			)
			.header(reqwest::header::HOST, "nimbledroid.com".to_string())
			.header(
				reqwest::header::USER_AGENT,
				"nimbledroidrs/0.0.1".to_string(),
			)
			.send()
		{
			Ok(mut o) => {
				let profile_status_result = o.text().unwrap();
				if let Ok(parsed_response) = serde_json::from_str::<Value>(&profile_status_result) {
					if let Ok(scenarios) = serde_json::from_value::<Vec<ProfileScenario>>(
						parsed_response["scenarios"].clone(),
					) {
						Some(ProfileScenarios { scenarios })
					} else {
						None
					}
				} else {
					None
				}
			}
			_ => None,
		}
	}

	pub fn get_profile_status(&self, upload_url: &Url) -> ProfileStatusResult {
		match self.get_profile_result(upload_url) {
			Some(o) => Ok(o.status),
			None => Err("Failed to get profile status:".to_string()),
		}
	}
	pub fn wait_for_profile(&self, upload_url: &Url) {
		while {
			match self.get_profile_status(upload_url) {
				Ok(ProfileStatus::Complete) => false,
				Ok(ProfileStatus::Failed) => false,
				_ => true,
			}
		} {}
	}

	pub fn upload(&self) -> ProfileUploadResult {
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
			.post(ND_UPLOAD_URL)
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
				let upload_result = o.text().unwrap();
				if let Ok(parsed_response) = serde_json::from_str::<Value>(&upload_result) {
					convert_Value_to_ProfileUploadResult(&parsed_response)
				} else {
					Err("Could not parse response.".to_string())
				}
			}
			Err(e) => Err(format!("Request to API failed: {}", e.to_string())),
		}
	}
}
