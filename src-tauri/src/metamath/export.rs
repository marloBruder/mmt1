use std::{
    collections::{HashMap, HashSet},
    fs,
    path::PathBuf,
};

use tauri::async_runtime::Mutex;

use crate::{
    model::{
        DatabaseElement, Header, HeaderRepresentation, MetamathData, OptimizedMetamathData,
        SymbolNumberMapping,
    },
    util::{self, earley_parser_optimized::Grammar},
    AppState, Error,
};

#[tauri::command]
pub async fn new_database(
    state: tauri::State<'_, Mutex<AppState>>,
    file_path: &str,
) -> Result<(HeaderRepresentation, u32), Error> {
    let mut app_state = state.lock().await;

    fs::write(file_path, "").map_err(|_| Error::FileWriteError)?;

    let path_buf = PathBuf::from(file_path);
    let file_name = path_buf.file_name().ok_or(Error::InternalLogicError)?;
    let file_name_string = file_name
        .to_str()
        .ok_or(Error::InternalLogicError)?
        .to_string();

    let metamath_data = MetamathData {
        alt_variable_colors: Vec::new(),
        database_id: app_state.id_manager.get_next_id(),
        database_hash: util::str_to_hash_string(""),
        database_header: Header {
            title: file_name_string,
            description: String::new(),
            content: Vec::new(),
            subheaders: Vec::new(),
        },
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

    let header_rep = metamath_data.database_header.to_representation();
    let database_id = metamath_data.database_id;

    app_state.metamath_data = Some(metamath_data);

    Ok((header_rep, database_id))
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

pub fn write_text_wrapped_maintain_paragraphs(target: &mut String, text: &str, line_prefix: &str) {
    let max_line_length = 80;
    let mut curr_line_length = last_line_length(&target);

    let mut i: usize = 0;

    while text.as_bytes().get(i).is_some() {
        let whitespace_end_i = whitespace_end_starting_at(text, i);
        let word_end_i = word_end_starting_at(text, whitespace_end_i);

        let whitespace = &text[i..whitespace_end_i];
        let word = &text[whitespace_end_i..word_end_i];

        if util::new_lines_in_str(whitespace) >= 2 && word != "" {
            target.push_str("\n\n");
            target.push_str(line_prefix);
            target.push_str(word);
            curr_line_length = line_prefix.len() + word.len();
        } else if curr_line_length + 1 + word.len() < max_line_length {
            target.push(' ');
            target.push_str(word);
            curr_line_length += 1 + word.len();
        } else {
            target.push('\n');
            target.push_str(line_prefix);
            target.push_str(word);
            curr_line_length = line_prefix.len() + word.len();
        }

        i = word_end_i;
    }
}

fn whitespace_end_starting_at(text: &str, start: usize) -> usize {
    let mut i = start;

    while text
        .as_bytes()
        .get(i)
        .is_some_and(|b| b.is_ascii_whitespace())
    {
        i += 1;
    }

    i
}

fn word_end_starting_at(text: &str, start: usize) -> usize {
    let mut i = start;

    while text
        .as_bytes()
        .get(i)
        .is_some_and(|b| !b.is_ascii_whitespace())
    {
        i += 1;
    }

    i
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
