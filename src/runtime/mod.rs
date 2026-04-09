mod input;
mod session;
mod timer;

use rodio::MixerDeviceSink;
use std::sync::{Arc, Mutex, mpsc};

pub(crate) enum TimerCommand {
    Start(i32),
    Stop,
    Shutdown,
}

struct GenerationJobResult {
    request_id: u64,
    result: Result<String, String>,
}

struct RuntimeContext<'a> {
    timer: &'a Arc<Mutex<i32>>,
    generation_tx: &'a mpsc::Sender<GenerationJobResult>,
    next_request_id: &'a mut u64,
    active_request_id: &'a mut Option<u64>,
    audio_sink: &'a MixerDeviceSink,
    timer_command_tx: &'a mpsc::Sender<TimerCommand>,
}

pub use session::run_app;
pub use timer::spawn_timer_thread;
