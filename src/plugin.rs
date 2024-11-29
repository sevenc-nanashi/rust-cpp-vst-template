use crate::ui::UiNotification;
use anyhow::Result;
use base64::{engine::general_purpose::STANDARD as base64, Engine as _};
use serde::{Deserialize, Serialize};
use std::{
    io::Write as _,
    sync::{Arc, Once},
};
use tokio::sync::{mpsc::UnboundedSender, Mutex, RwLock};

pub struct PluginImpl {
    pub notification_sender: Option<UnboundedSender<UiNotification>>,

    pub params: Arc<RwLock<PluginParams>>,

    prev_position: i64,
    prev_is_playing: bool,

    pub current_position: f32,
    pub current_position_updated: bool,
}
impl std::fmt::Debug for PluginImpl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PluginImpl").finish_non_exhaustive()
    }
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct PluginParams {
    pub dummy: String,
}

static INIT: Once = Once::new();

impl PluginImpl {
    pub fn new(params: PluginParams) -> Self {
        INIT.call_once(|| {
            if option_env!("RUST_VST_LOG").map_or(false, |v| v.len() > 0) {
                let dest = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR").to_string())
                    .join("logs")
                    .join(format!(
                        "{}.log",
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs()
                    ));

                let Ok(writer) = std::fs::OpenOptions::new()
                    .create(true)
                    .write(true)
                    .truncate(true)
                    .open(&dest)
                else {
                    return;
                };

                let default_panic_hook = std::panic::take_hook();

                std::panic::set_hook(Box::new(move |info| {
                    let mut panic_writer =
                        std::fs::File::create(dest.with_extension("panic")).unwrap();
                    let backtrace = std::backtrace::Backtrace::force_capture();
                    let _ = writeln!(panic_writer, "{}\n{}", info, backtrace);

                    default_panic_hook(info);
                }));

                let _ = tracing_subscriber::fmt()
                    .with_writer(writer)
                    .with_ansi(false)
                    .try_init();
            }
        });
        PluginImpl {
            notification_sender: None,
            params: Arc::new(RwLock::new(params)),

            prev_position: 0,
            prev_is_playing: false,

            current_position: 0.0,
            current_position_updated: false,
        }
    }

    // NOTE: DPF cannot handle binary data, so it's required to encode the state in base64
    pub fn set_state(&self, state_base64: &str) -> Result<()> {
        if state_base64.is_empty() {
            return Ok(());
        }
        let mut params = self.params.blocking_write();
        let state = base64.decode(state_base64)?;
        let loaded_params = bincode::deserialize(&state)?;
        *params = loaded_params;

        Ok(())
    }

    pub fn get_state(&self) -> String {
        let params = { self.params.blocking_read().clone() };
        let state = bincode::serialize(&params).unwrap();
        base64.encode(state.as_slice())
    }

    pub fn run(
        this_ref: Arc<Mutex<PluginImpl>>,
        inputs: &[&[f32]],
        outputs: &mut [&mut [f32]],
        sample_rate: f32,
        is_playing: bool,
        current_sample: i64,
    ) {
        for output in outputs.iter_mut() {
            for sample in output.iter_mut() {
                *sample = 0.0;
            }
        }
        if let Ok(mut this) = this_ref.try_lock() {
            // ...

            if this.prev_position != current_sample {
                this.prev_position = current_sample;
                this.current_position = (current_sample as f32 / sample_rate).max(0.0);
                this.current_position_updated = true;
            }
            if this.prev_is_playing != is_playing {
                this.prev_is_playing = is_playing;
                if let Some(sender) = &this.notification_sender {
                    if sender
                        .send(UiNotification::UpdatePlayingState(is_playing))
                        .is_err()
                    {
                        this.notification_sender = None;
                    }
                }
            }
        }
    }
}
