#![cfg(not(debug_assertions))]

use std::env;

use std::error::Error;
use std::fs::File;
use std::io::Write;

use reqwest::get;
use semver::Version;
use serde_json::Value;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

/// Checks and performs updates, returns true if an update was performed
pub async fn update_check() -> bool {
    // Get the latest release on GitHub
    let latest_version = Version::parse(
        get_latest_version()
            .await
            .unwrap_or_else(|_| {
                eprintln!("Failed to fetch latest version from GitHub!");
                String::from("0.0.0")
            })
            .as_str(),
    )
    .unwrap();
    let current_version = Version::parse(env!("CARGO_PKG_VERSION")).unwrap();

    // Compare our version string
    if latest_version > current_version && perform_update().await.is_ok() {
        true
    } else {
        false
    }
}

/// Gets the latest version number from GitHub
async fn get_latest_version() -> Result<String> {
    let response =
        get("https://api.github.com/repos/purduehackers/door-opener/releases/latest").await?;

    let json: Value = response.json().await?;

    let tag_name = json
        .get("tag_name")
        .ok_or("missing tag_name")?
        .as_str()
        .ok_or("invalid type for tag_name")?
        .to_string();

    Ok(tag_name)
}

async fn perform_update() -> Result<()> {
    // Where are we?
    let current_executable_path = env::current_exe().unwrap();

    // Grab artifact from latest release
    let response =
        get("https://github.com/purduehackers/door-opener/releases/latest/download/door-opener")
            .await?;
    let artifact = response.bytes().await?.to_vec();

    // Replace the current executable with the downloaded artifact
    let mut file = File::create(&current_executable_path)?;
    file.write_all(&artifact)?;

    Ok(())
}
