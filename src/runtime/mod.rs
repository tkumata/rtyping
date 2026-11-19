mod input;
mod session;

type GenerationJob = std::sync::mpsc::Receiver<Result<String, String>>;

pub use session::run_app;
