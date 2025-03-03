use crate::{AppState, Error};
use tauri::async_runtime::Mutex;

#[tauri::command]
pub async fn add_to_database(
    state: tauri::State<'_, Mutex<AppState>>,
    text: &str,
) -> Result<(), Error> {
    let lines = text_to_lines(text);

    for line in lines {
        for token in line {
            print!("{} ", token);
        }
        println!("");
    }

    let app_state = state.lock().await;

    Ok(())
}

fn text_to_lines(text: &str) -> Vec<Vec<&str>> {
    let mut lines_vec = Vec::new();

    let mut line_iter = text.lines().peekable();

    while let Some(line) = line_iter.next() {
        let mut line_vec = Vec::new();

        let mut token_iter = line.split_ascii_whitespace().peekable();

        while let Some(token) = token_iter.next() {
            line_vec.push(token);

            // If token_iter is empty, extend token_iter by the tokens on the next nonempty line,
            // that starts with whitespace (if it exists).
            while token_iter.peek() == None
                && line_iter.peek().is_some_and(|l| {
                    l.chars()
                        .next()
                        .map(|c| c.is_ascii_whitespace())
                        .unwrap_or(true)
                })
            {
                token_iter = line_iter
                    .next()
                    .unwrap()
                    .split_ascii_whitespace()
                    .peekable();
            }
        }

        lines_vec.push(line_vec);
    }

    lines_vec
}
