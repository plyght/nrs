use crate::ai::{openai_keywords_blocking, openai_summarize_blocking};
use crate::notes::note_path;
use crate::tui::AppState;
use crate::MyError;
use futures::executor::block_on;
use std::fs;
use tokio::task;

/// Handle a command entered in the TUI.
pub fn handle_cmd(cmd: String, st: &mut AppState) -> Result<(), MyError> {
    let trimmed = cmd.trim_start_matches(':').trim();
    match trimmed {
        "summarize" => {
            if let Some(sn_ref) = st.selected_note() {
                let sn = sn_ref.clone();
                let content = fs::read_to_string(note_path(&sn))?;
                let handle = task::spawn_blocking(move || openai_summarize_blocking(content));
                let text = block_on(handle)??;
                st.last_ai_output = Some(text);
                st.status_message = Some(format!("AI Summarize done for '{}'", sn));
            }
        }
        "keywords" => {
            if let Some(sn_ref) = st.selected_note() {
                let sn = sn_ref.clone();
                let content = fs::read_to_string(note_path(&sn))?;
                let handle = task::spawn_blocking(move || openai_keywords_blocking(content));
                let text = block_on(handle)??;
                st.last_ai_output = Some(text);
                st.status_message = Some(format!("AI Keywords done for '{}'", sn));
            }
        }
        other => {
            st.last_ai_output = Some(format!("Unknown command: '{}'", other));
            st.status_message = Some(format!("Unknown command: '{}'", other));
        }
    }
    Ok(())
}
