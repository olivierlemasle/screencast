use std::{io, thread, time};

use anyhow::Result;

pub trait Wait {
    fn wait(&self, device: &str) -> Result<()>;
}

pub struct WaitInput;

impl Wait for WaitInput {
    fn wait(&self, _device: &str) -> Result<()> {
        io::stdin().read_line(&mut String::new())?;
        Ok(())
    }
}

pub struct WaitDelay {
    duration: time::Duration,
}

impl WaitDelay {
    pub fn new(duration: time::Duration) -> Self {
        Self { duration }
    }

    pub fn from_secs(secs: u64) -> Self {
        Self {
            duration: time::Duration::from_secs(secs),
        }
    }
}

impl Wait for WaitDelay {
    fn wait(&self, _device: &str) -> Result<()> {
        thread::sleep(self.duration);
        Ok(())
    }
}

pub mod android;
