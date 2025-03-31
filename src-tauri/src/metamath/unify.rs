use crate::Error;

#[tauri::command]
pub async fn unify(text: &str, cursor_pos: u32) -> Result<String, Error> {
    let mut res = String::new();

    let mut statement = String::new();

    let mut line_num = 0;

    // Chain an extra dummy line to close off the last statement
    for line in text.lines().chain("dummy_line".lines()) {
        line_num += 1;
        if line.chars().next().is_none_or(|c| c.is_ascii_whitespace()) {
            statement.push_str(line);
            statement.push('\n');
        } else {
            if res.len() as u32 <= cursor_pos && cursor_pos < (res.len() + statement.len()) as u32 {
                println!("{}", &statement);
            }
            res.push_str(&statement);
            statement = String::new();
            statement.push_str(line);
            statement.push('\n');
        }
    }

    println!("{}", line_num);

    Ok(res)
}
