extern crate nimbledroidrs;

use nimbledroidrs::Profiler;
use reqwest::Url;
use std::time::Duration;

fn main() {
	let profile = Profile::new(
		"APIKEY",
		"/home/hawkinsw/code/firefox/nimbledroid/test.apk",
	);
	let upload_url: Url;
	match profiler.upload() {
		Ok(apk_url) => upload_url = apk_url,
		Err(e) => {
			println!("Error occurred uploading APK: {}", e.to_string());
			return;
		}
	}

	println!(
		"APK profile results are available at {}",
		upload_url.to_string()
	);

	println!("Starting to wait for ND to finish profiling the application.");
	if profiler
		.wait_for_profile(&upload_url, Duration::from_secs(1200))
		.is_err()
	{
		println!("Timed out waiting for ND to finish profiling the application.");
		return;
	}
	println!("Done waiting for ND to finish profiling the application.");

	if let Some(profile_result) = profiler.get_profile_result(&upload_url) {
		for p in profile_result.profiles {
			println!("Profile: {}", p);
			if let Some(scenarios) =
				profiler.get_profile_scenarios(&Url::parse(&p.profile_url).unwrap())
			{
				for scenario in scenarios.get_scenarios() {
					println!("{}", scenario);
				}
			}
		}
	}
}
