use std::{
    collections::{HashMap, HashSet},
    fs,
};

use tauri::async_runtime::Mutex;

use crate::{
    model::{DatabaseElement, Header, MetamathData, OptimizedMetamathData, SymbolNumberMapping},
    util::{self, earley_parser_optimized::Grammar},
    AppState, Error,
};

#[tauri::command]
pub async fn new_database(
    state: tauri::State<'_, Mutex<AppState>>,
    file_path: &str,
) -> Result<(), Error> {
    let mut app_state = state.lock().await;

    fs::write(file_path, "").map_err(|_| Error::FileWriteError)?;

    let metamath_data = MetamathData {
        alt_variable_colors: Vec::new(),
        database_id: app_state.id_manager.get_next_id(),
        database_hash: util::str_to_hash_string(""),
        database_header: Header::default(),
        html_representations: Vec::new(),
        optimized_data: OptimizedMetamathData {
            floating_hypotheses: Vec::new(),
            variables: HashSet::new(),
            theorem_amount: 0,
            theorem_data: HashMap::new(),
            header_data: HashMap::new(),
            symbol_number_mapping: SymbolNumberMapping::default(),
            grammar: Grammar::default(),
        },
        database_path: file_path.to_string(),
        grammar_calculations_done: true,
        syntax_typecodes: Vec::new(),
        logical_typecodes: Vec::new(),
        variable_colors: Vec::new(),
    };

    app_state.metamath_data = Some(metamath_data);

    Ok(())
}

#[tauri::command]
pub async fn save_database(state: tauri::State<'_, Mutex<AppState>>) -> Result<(), Error> {
    let app_state = state.lock().await;
    let mm_data = app_state.metamath_data.as_ref().ok_or(Error::NoMmDbError)?;

    let database_string = calc_database_string(mm_data);

    fs::write(&mm_data.database_path, database_string).or(Err(Error::FileWriteError))?;

    Ok(())
}

#[tauri::command]
pub async fn export_database(
    state: tauri::State<'_, Mutex<AppState>>,
    file_path: &str,
) -> Result<(), Error> {
    let app_state = state.lock().await;
    let mm_data = app_state.metamath_data.as_ref().ok_or(Error::NoMmDbError)?;

    let database_string = calc_database_string(mm_data);

    fs::write(file_path, database_string).or(Err(Error::FileWriteError))?;

    Ok(())
}

fn calc_database_string(metamath_data: &MetamathData) -> String {
    let mut res = String::new();

    for element in metamath_data.database_header.iter() {
        match element {
            DatabaseElement::Header(header, depth) => {
                res.push_str("$(\n");
                res.push_str(header_line(depth));
                res.push_str("\n  ");
                res.push_str(&header.title);
                res.push('\n');
                res.push_str(header_line(depth));
                res.push('\n');
                res.push_str("$)\n\n");
            }
            DatabaseElement::Statement(statement) => {
                statement.write_mm_string(&mut res);
                res.push_str("\n\n");
            }
        }
    }

    res
}

pub fn write_text_wrapped(target: &mut String, text: &str, line_prefix: &str) {
    let max_line_length = 80;
    let mut curr_line_length = last_line_length(&target);

    for token in text.split_ascii_whitespace() {
        if curr_line_length + 1 + token.len() < max_line_length {
            target.push(' ');
            target.push_str(token);
            curr_line_length += 1 + token.len();
        } else {
            target.push('\n');
            target.push_str(line_prefix);
            target.push_str(token);
            curr_line_length = line_prefix.len() + token.len();
        }
    }
}

pub fn write_text_wrapped_no_whitespace(target: &mut String, text: &str, line_prefix: &str) {
    let max_line_length = 80;
    let mut curr_line_length = last_line_length(&target);

    for char in text.chars() {
        if !char.is_ascii_whitespace() {
            if curr_line_length >= max_line_length - 1 {
                target.push('\n');
                target.push_str(line_prefix);
                curr_line_length = line_prefix.len();
            }
            target.push(char);
            curr_line_length += 1;
        }
    }
}

fn last_line_length(text: &str) -> usize {
    let mut len = 0;
    let mut index = text.len() - 1;

    while index > 0 && text.as_bytes()[index] != b'\n' {
        len += 1;
        index -= 1;
    }

    len
}

fn header_line(depth: u32) -> &'static str {
    match depth {
        1 => "###############################################################################",
        2 => "#*#*#*#*#*#*#*#*#*#*#*#*#*#*#*#*#*#*#*#*#*#*#*#*#*#*#*#*#*#*#*#*#*#*#*#*#*#*#*#",
        3 => "=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=",
        4 => "-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-",
        _ => "",
    }
}
