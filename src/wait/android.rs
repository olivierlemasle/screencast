//! Android crate

use std::process::Command;

use anyhow::{anyhow, Context, Result};
use regex::Regex;

use super::Wait;

pub struct Emulator {
    emulator_path: String,
    avd: String,
}

impl Emulator {
    pub fn new(emulator_path: Option<String>, avd: Option<String>) -> Self {
        Emulator {
            emulator_path: emulator_path
                .unwrap_or("/home/olivier/Android/Sdk/emulator/emulator".to_string()),
            avd: avd.unwrap_or("Pixel_4_API_29".to_string()),
        }
    }

    fn get_webcam(&self, device: &str) -> Result<String> {
        let output = Command::new(&self.emulator_path)
            .arg("-webcam-list")
            .output()
            .with_context(|| format!("Unable to execute {}", &self.emulator_path))?;

        anyhow::ensure!(
            output.status.success(),
            "Android emulator could not successfully list webcams"
        );

        let re = Regex::new(r"Camera '(?<name>\w+)' ").unwrap();

        let stdout = String::from_utf8_lossy(&output.stdout);
        let webcam_name = stdout
            .lines()
            .find(|&line| line.contains(device))
            .and_then(|line| re.captures(line))
            .and_then(|c| c.name("name"))
            .map(|m| m.as_str())
            .ok_or(anyhow!("no webcam found"))?;

        println!("Android Emulator camera '{webcam_name}' is connected to device '{device}'.");
        Ok(webcam_name.to_owned())
    }

    pub fn launch(&self, device: &str) -> Result<()> {
        let webcam = self.get_webcam(device)?;

        let mut child = Command::new(&self.emulator_path)
            .arg("-avd")
            .arg(&self.avd)
            .arg("-camera-back")
            .arg(webcam)
            .spawn()?;

        child.wait()?;
        Ok(())
    }
}

impl Wait for Emulator {
    fn wait(&self, device: &str) -> Result<()> {
        self.launch(device)
    }
}
