extern crate nimbledroidrs;

use nimbledroidrs::Profile;

fn main() {
	let profile = Profile::new(
		"APIKEY",
		"/home/hawkinsw/code/firefox/nimbledroid/test.apk",
	);

	match profile.upload() {
      Ok(apk_url) => {
          println!("Got an APK URL: {}", apk_url.to_string());
      }
      Err(e) => {
          println!("Upload failed: {}", e.to_string());
      }
  }
}
