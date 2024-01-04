use anyhow::{anyhow, Context, Result};
use ashpd::{
    desktop::screencast::{CursorMode, PersistMode, Screencast, SourceType},
    WindowIdentifier,
};
use gstreamer::{prelude::*, ClockTime, StateChangeSuccess};
use v4l::{capability, context as v4lcontext, Device};

pub mod wait;

fn default_video_dev() -> Result<String> {
    v4lcontext::enum_devices()
        .iter()
        .find_map(|node| {
            let path = node.path();
            let dev = Device::with_path(path).ok()?;
            let caps = dev.query_caps().ok()?;
            if caps.driver.to_lowercase().contains("loopback")
                && caps.capabilities.contains(capability::Flags::VIDEO_OUTPUT)
            {
                Some(path.to_string_lossy().into_owned())
            } else {
                None
            }
        })
        .ok_or(anyhow!(
            "no v4l device with driver name containing 'loopback' and with Video output capability"
        ))
}

fn check_device_capabilities(path: &str) -> Result<()> {
    let caps = Device::with_path(path)?.query_caps()?.capabilities;

    anyhow::ensure!(
        caps.contains(capability::Flags::VIDEO_CAPTURE),
        "device {} does not have Video capture capability",
        path
    );
    Ok(())
}

async fn start_screencast() -> Result<u32> {
    let proxy = Screencast::new().await?;
    let session = proxy.create_session().await?;
    proxy
        .select_sources(
            &session,
            CursorMode::Hidden,
            SourceType::Monitor | SourceType::Window,
            false,
            None,
            PersistMode::DoNot,
        )
        .await?;

    let response = proxy
        .start(&session, &WindowIdentifier::default())
        .await?
        .response()?;

    let stream = response.streams().first().ok_or(anyhow!("no stream"))?;
    println!("Screencast stream: {:?}", stream);

    Ok(stream.pipe_wire_node_id())
}

pub async fn run(dev_path: Option<String>, waiter: Box<dyn wait::Wait>) -> Result<()> {
    let dev_path = match dev_path {
        Some(path) => path,
        None => default_video_dev()?,
    };

    println!("Video device: {dev_path}");

    let node_id = start_screencast()
        .await
        .with_context(|| "cannot get video stream from Screencast portal")?;

    gstreamer::init()?;

    let pipeline_cmd = format!(
        "pipewiresrc path={} ! videoconvert ! videoscale ! video/x-raw,format=YUY2,width=850,height=480 ! tee ! v4l2sink device={}",
        node_id, dev_path
    );
    println!("GStreamer pipeline: {}", &pipeline_cmd);
    let pipeline = gstreamer::parse_launch(&pipeline_cmd)?;
    let result = pipeline
        .set_state(gstreamer::State::Playing)
        .with_context(|| "cannot start gstreamer pipeline")?;
    if result == StateChangeSuccess::Async {
        let (state_result, _, _) = pipeline.state(ClockTime::from_seconds(10));
        let state = state_result?;

        println!("GStreamer pipeline start: {:?}", state);
    }

    check_device_capabilities(&dev_path)?;

    waiter.wait(&dev_path)?;

    let result = pipeline
        .set_state(gstreamer::State::Null)
        .with_context(|| "cannot stop gstreamer pipeline")?;
    println!("GStreamer pipeline stop: {:?}", result);

    Ok(())
}
