use anyhow::Result;
use clap::{Parser, Subcommand};

use screencast::wait;

/// Screencast records the screen and sends it to a loopback video device.
#[derive(Parser, Debug)]
#[command(version)]
struct Cli {
    /// Video device. If not provided, detected as the first v4l2 device with a
    /// driver name containing 'loopback' and with video output capability
    #[arg(short, long)]
    device: Option<String>,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Wait for user input.
    WaitInput,

    /// Wait a fixed amount of time.
    WaitDelay {
        /// Delay, in seconds
        #[arg(short, long)]
        seconds: u64,
    },

    /// Starts the Android Emulator and wait for it to be closed.
    Android {
        emulator_path: Option<String>,
        avd: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let wait = get_waiter(cli.command);
    screencast::run(cli.device, wait).await?;

    println!("Bye.");
    Ok(())
}

fn get_waiter(command: Option<Command>) -> Box<dyn wait::Wait> {
    match command {
        Some(Command::WaitDelay { seconds }) => Box::new(wait::WaitDelay::from_secs(seconds)),
        Some(Command::Android { emulator_path, avd }) => {
            Box::new(wait::android::Emulator::new(emulator_path, avd))
        }
        _ => Box::new(wait::WaitInput),
    }
}

// adb shell am start -n com.whatsapp/.HomeActivity
// xdg-open
// adb kill-server
// https://developer.android.com/studio/run/emulator-commandline?hl=fr   - -port
