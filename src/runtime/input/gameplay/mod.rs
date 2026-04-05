mod generation;
mod loading;
mod result;
mod typing;

pub(super) use generation::{apply_generation_result, spawn_generation_job};
pub(super) use loading::handle_loading_input;
pub(super) use result::handle_result_input;
pub(super) use typing::handle_typing_input;
