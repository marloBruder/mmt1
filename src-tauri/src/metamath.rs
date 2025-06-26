use crate::{
    local_state::{
        // constant::set_constants_local,
        floating_hypothesis::{
            get_floating_hypothesis_by_label, /*set_floating_hypotheses_local*/
        },
        // html_representation::set_html_representations_local,
        // in_progress_theorem::delete_in_progress_theorem_local,
        theorem::{add_theorem_local, get_theorem_insert_position},
        // variable::set_variables_local,
    },
    model::{Hypothesis, MetamathData, ProofLine, Theorem, TheoremPageData, TheoremPath},
    util::last_curr_next_iterator::IntoLastCurrNextIterator,
    AppState, Error,
};
use std::collections::HashMap;
use tauri::async_runtime::Mutex;

pub mod export;
pub mod mm_parser;
pub mod mmp_parser;

#[tauri::command]
pub async fn turn_into_theorem(
    state: tauri::State<'_, Mutex<AppState>>,
    text: &str,
    position_name: &str,
) -> Result<TheoremPath, Error> {
    if !text.is_ascii() {
        return Err(Error::InvalidCharactersError);
    }

    let mut last_token: Option<&str> = None;

    let mut name: Option<String> = None;
    let mut description = String::from("");
    let mut disjoints: Vec<String> = Vec::new();
    let mut hypotheses: Vec<Hypothesis> = Vec::new();
    let mut assertion: Option<String> = None;
    let mut proof: Option<String> = None;

    let mut token_iter = text.split_whitespace();
    while let Some(token) = token_iter.next() {
        match token {
            "$(" => description = get_next_as_string_until(&mut token_iter, "$)"),
            "$d" => {
                let disjoint_cond = get_next_as_string_until(&mut token_iter, "$.");
                disjoints.push(disjoint_cond);
            }
            "$e" => {
                let label = last_token.ok_or(Error::InvalidFormatError)?.to_string();
                let expression = get_next_as_string_until(&mut token_iter, "$.");
                hypotheses.push(Hypothesis { label, expression })
            }
            "$a" => {
                name = last_token.map(|s| s.to_string());
                assertion = Some(get_next_as_string_until(&mut token_iter, "$."));
            }
            "$p" => {
                name = last_token.map(|s| s.to_string());
                assertion = Some(get_next_as_string_until(&mut token_iter, "$="));
                proof = Some(get_next_as_string_until(&mut token_iter, "$."));
            }
            _ => {
                last_token = Some(token);
            }
        }
    }

    let name = name.ok_or(Error::InvalidFormatError)?;
    let assertion = assertion.ok_or(Error::InvalidFormatError)?;

    let mut app_state = state.lock().await;
    let metamath_data = app_state.metamath_data.as_mut().ok_or(Error::NoMmDbError)?;

    let insert_path = get_theorem_insert_position(metamath_data, position_name)?;

    add_theorem_local(
        metamath_data,
        &name,
        &description,
        &disjoints,
        &hypotheses,
        &assertion,
        proof.as_deref(),
        &insert_path,
    )?;

    //TODO: Add to database

    Ok(insert_path)
}

fn get_next_as_string_until(iter: &mut std::str::SplitWhitespace, until: &str) -> String {
    let mut result = String::new();
    while let Some(token) = iter.next() {
        if token == until {
            break;
        } else {
            result.push_str(token);
            result.push(' ');
        }
    }
    result.pop();
    result
}

// #[tauri::command]
// pub async fn text_to_constants(
//     state: tauri::State<'_, Mutex<AppState>>,
//     text: &str,
// ) -> Result<Vec<Constant>, Error> {
//     if !text.is_ascii() {
//         return Err(Error::InvalidCharactersError);
//     }

//     let symbols = text_to_constant_or_variable_symbols(text, true)?;

//     let mut app_state = state.lock().await;
//     let db_state = app_state.db_state.as_mut().ok_or(Error::NoDatabaseError)?;

//     set_constants_database(&mut db_state.db_conn, &symbols).await?;

//     set_constants_local(&mut db_state.metamath_data, &symbols);

//     let mut constants = Vec::new();

//     for symbol in symbols {
//         constants.push(Constant {
//             symbol: symbol.to_string(),
//         })
//     }

//     Ok(constants)
// }

// #[tauri::command]
// pub async fn text_to_variables(
//     state: tauri::State<'_, Mutex<AppState>>,
//     text: &str,
// ) -> Result<Vec<Variable>, Error> {
//     if !text.is_ascii() {
//         return Err(Error::InvalidCharactersError);
//     }

//     let symbols = text_to_constant_or_variable_symbols(text, false)?;

//     let mut app_state = state.lock().await;
//     let db_state = app_state.db_state.as_mut().ok_or(Error::NoDatabaseError)?;

//     set_variables_database(&mut db_state.db_conn, &symbols).await?;

//     set_variables_local(&mut db_state.metamath_data, &symbols);

//     let mut variables = Vec::new();

//     for symbol in symbols {
//         variables.push(Variable {
//             symbol: symbol.to_string(),
//         })
//     }

//     Ok(variables)
// }

// // Takes a text and returns references to the symbols between
// // "$c" and "$.", if constant is true,
// // "$v" and "$.", if constant is false
// // If there is a string not between these, the function returns an Error
// fn text_to_constant_or_variable_symbols(text: &str, constant: bool) -> Result<Vec<&str>, Error> {
//     let mut symbols = Vec::new();

//     // True if token is after "$c", but before the next "$."
//     let mut within_statement = false;

//     for token in text.split_whitespace() {
//         if !within_statement {
//             match token {
//                 "$c" if constant => within_statement = true,
//                 "$v" if !constant => within_statement = true,
//                 _ => return Err(Error::InvalidFormatError),
//             }
//         } else {
//             match token {
//                 "$." => within_statement = false,
//                 s => symbols.push(s),
//             }
//         }
//     }

//     Ok(symbols)
// }

// #[tauri::command]
// pub async fn text_to_floating_hypotheses(
//     state: State<'_, Mutex<AppState>>,
//     text: &str,
// ) -> Result<Vec<FloatingHypohesis>, Error> {
//     if !text.is_ascii() {
//         return Err(Error::InvalidCharactersError);
//     }

//     let mut floating_hypotheses = Vec::new();

//     let mut token_iter = text.split_whitespace();

//     let mut next_label: Option<&str> = None;

//     while let Some(token) = token_iter.next() {
//         match (token, next_label) {
//             ("$f", Some(label)) => {
//                 let typecode = token_iter.next();
//                 let variable = token_iter.next();

//                 if token_iter.next() == Some("$.") {
//                     floating_hypotheses.push(FloatingHypohesis {
//                         label: label.to_string(),
//                         // Safe unwraps, because the if branch requires a later call of next to have returned Some
//                         typecode: typecode.unwrap().to_string(),
//                         variable: variable.unwrap().to_string(),
//                     });
//                     next_label = None;
//                 } else {
//                     return Err(Error::InvalidFormatError);
//                 }
//             }
//             (label, None) => next_label = Some(label),
//             (_, _) => return Err(Error::InvalidFormatError),
//         }
//     }

//     let mut app_state = state.lock().await;
//     let db_state = app_state.db_state.as_mut().ok_or(Error::NoDatabaseError)?;

//     set_floating_hypotheses_database(&mut db_state.db_conn, &floating_hypotheses).await?;

//     set_floating_hypotheses_local(&mut db_state.metamath_data, &floating_hypotheses);

//     Ok(floating_hypotheses)
// }

// #[tauri::command]
// pub async fn text_to_html_representations(
//     state: State<'_, Mutex<AppState>>,
//     text: &str,
// ) -> Result<Vec<HtmlRepresentation>, Error> {
//     if !text.is_ascii() {
//         return Err(Error::InvalidCharactersError);
//     }

//     let tokens = tokenize_typesetting_text(text)?;
//     let mut token_iter = tokens.iter();

//     let mut html_representations = Vec::new();

//     loop {
//         let mut statement_tokens: Vec<&str> = Vec::new();
//         while let Some(&token) = token_iter.next() {
//             if !token.starts_with("/*") {
//                 if token != ";" {
//                     statement_tokens.push(token);
//                 } else {
//                     break;
//                 }
//             }
//         }

//         if statement_tokens.len() == 0 {
//             break;
//         }

//         if statement_tokens.len() < 4
//             || statement_tokens.len() % 2 != 0
//             || statement_tokens[0] != "htmldef"
//             || statement_tokens[2] != "as"
//         {
//             return Err(Error::InvalidFormatError);
//         }

//         let mut html: String =
//             get_str_in_quotes(statement_tokens[3]).ok_or(Error::InvalidFormatError)?;

//         let mut next_html_index = 5;

//         while next_html_index < statement_tokens.len() {
//             if statement_tokens[next_html_index - 1] != "+" {
//                 return Err(Error::InvalidFormatError);
//             }
//             html.push_str(
//                 &get_str_in_quotes(statement_tokens[next_html_index])
//                     .ok_or(Error::InvalidFormatError)?,
//             );

//             next_html_index += 2;
//         }

//         html_representations.push(HtmlRepresentation {
//             symbol: get_str_in_quotes(statement_tokens[1])
//                 .ok_or(Error::InvalidFormatError)?
//                 .to_string(),
//             html,
//         })
//     }

//     let mut app_state = state.lock().await;
//     let db_state = app_state.db_state.as_mut().ok_or(Error::NoDatabaseError)?;

//     set_html_representations_database(&mut db_state.db_conn, &html_representations).await?;

//     set_html_representations_local(&mut db_state.metamath_data, &html_representations);

//     Ok(html_representations)
// }

fn tokenize_typesetting_text(text: &str) -> Result<Vec<&str>, Error> {
    let mut tokens = Vec::new();

    let text_bytes = text.as_bytes();

    let mut index: usize = 0;

    while index < text.len() {
        let first = text_bytes[index];
        let second = if index + 1 < text.len() {
            Some(text_bytes[index + 1])
        } else {
            None
        };

        match (first, second) {
            (b';', _) => {
                tokens.push(&text[index..(index + 1)]);
                index += 1;
            }
            (b'/', Some(b'*')) => {
                let mut end_index = index + 2;

                loop {
                    end_index += 1;
                    if end_index >= text.len() {
                        // println!("Unclosed comment");
                        return Err(Error::TypesettingFormatError);
                    }
                    if text_bytes[end_index - 1] == b'*' && text_bytes[end_index] == b'/' {
                        break;
                    }
                }
                tokens.push(&text[index..(end_index + 1)]);
                index = end_index + 1;
            }
            (quote_type @ (b'\"' | b'\''), _) => {
                let mut end_index = index;

                loop {
                    end_index += 1;
                    if end_index >= text.len() {
                        // println!("Unclosed Quote");
                        return Err(Error::TypesettingFormatError);
                    }
                    if text_bytes[end_index] == quote_type {
                        if end_index + 1 < text.len() && text_bytes[end_index + 1] == quote_type {
                            end_index += 1;
                        } else {
                            break;
                        }
                    }
                }
                tokens.push(&text[index..(end_index + 1)]);
                index = end_index + 1;

                if index < text.len()
                    && !text_bytes[index].is_ascii_whitespace()
                    && text_bytes[index] != b';'
                {
                    // println!("Something after quote");
                    return Err(Error::TypesettingFormatError);
                }
            }
            (c, _) if c.is_ascii_whitespace() => index += 1,
            (_, _) => {
                let mut end_index = index + 1;
                while end_index <= text.len()
                    && !text_bytes[end_index].is_ascii_whitespace()
                    && text_bytes[index] != b';'
                {
                    end_index += 1;
                }
                tokens.push(&text[index..end_index]);
                index = end_index;
            }
        }
    }

    Ok(tokens)
}

// fn advance_until_whitespace(str: &str, mut index: usize) -> usize {
//     while index < str.len() {
//         if str.as_bytes()[index].is_ascii_whitespace() {
//             break;
//         }
//         index += 1;
//     }
//     index
// }

// fn advance_until_non_whitespace(str: &str, mut index: usize) -> usize {
//     while index < str.len() {
//         if !str.as_bytes()[index].is_ascii_whitespace() {
//             break;
//         }
//         index += 1;
//     }
//     index
// }

// fn advance_until_quotes(str: &str, mut index: usize) -> usize {
//     while index < str.len() {
//         if str.as_bytes()[index] == b'\"' {
//             break;
//         }
//         index += 1;
//     }
//     index
// }

fn get_str_in_quotes(str: &str) -> Option<String> {
    let chars: Vec<char> = str.chars().collect();

    if chars.len() < 3
        || !((*chars.first().unwrap() == '\"' && *chars.last().unwrap() == '\"')
            || (*chars.first().unwrap() == '\'' && *chars.last().unwrap() == '\''))
    {
        return None;
    }

    let (replace, replace_with) = if *chars.first().unwrap() == '\"' {
        ("\"\"", "\"")
    } else {
        ("\'\'", "\'")
    };

    Some(str[1..(str.len() - 1)].replace(replace, replace_with))
}

#[derive(Debug)]
struct ProofStep {
    pub label: String,
    pub hypotheses: Vec<String>,
    pub statement: String,
    // dispaly_step_number is -1, if the proof step was not saved,
    // else the display_step_num of the last stack_line when step was saved
    pub display_step_number: i32,
}

#[derive(Debug)]
struct StackLine {
    pub statement: String,
    pub display_step_number: i32,
}

pub fn calc_theorem_page_data(
    label: &str,
    metamath_data: &MetamathData,
) -> Result<TheoremPageData, Error> {
    let (theorem_i, (last_theorem, theorem, next_theorem)) = metamath_data
        .database_header
        .theorem_iter()
        .last_curr_next()
        .enumerate()
        .find(|(_, (_, curr_t, _))| curr_t.label == label)
        .ok_or(Error::NotFoundError)?;

    let last_theorem_label = last_theorem.map(|t| t.label.clone());
    let next_theorem_label = next_theorem.map(|t| t.label.clone());

    let theorem_number = (theorem_i + 1) as u32;

    if theorem.proof == None {
        return Ok(TheoremPageData {
            theorem: theorem.clone(),
            theorem_number,
            proof_lines: Vec::new(),
            preview_errors: None,
            preview_confirmations: None,
            preview_confirmations_recursive: None,
            last_theorem_label,
            next_theorem_label,
        });
    }
    let mut proof_lines = Vec::new();

    let mut proof_steps: Vec<ProofStep>;
    // step numbers begin at 1
    let step_numbers: Vec<(u32, bool)>;

    if theorem.proof.as_ref().unwrap().starts_with("( ") {
        proof_steps = calc_proof_steps_compressed(theorem, metamath_data)?;
        step_numbers = calc_proof_step_numbers_compressed(theorem)?;
    } else {
        (proof_steps, step_numbers) =
            calc_proof_steps_and_numbers_uncompressed(theorem, metamath_data)?;
    }

    // for (i, step) in proof_steps.iter().enumerate() {
    //     println!("Step {}:\n{:?}", i + 1, step);
    // }
    // println!("\nNumbers:\n{:?}\n", step_numbers);

    let mut stack: Vec<StackLine> = Vec::new();

    let mut next_hypotheses_num = 1;

    let step_numbers_len = step_numbers.len();

    for (i, (step_num, save)) in step_numbers.into_iter().enumerate() {
        let step = proof_steps
            .get((step_num - 1) as usize)
            .ok_or(Error::InvalidProofError)?;
        let mut hypotheses_nums: Vec<i32> = Vec::new();

        if step.hypotheses.len() == 0 {
            stack.push(StackLine {
                statement: step.statement.clone(),
                display_step_number: -1,
            });
        } else {
            let (next_step, display_hypotheses_num) =
                calc_step_application(step, &stack, metamath_data)?;
            for i in 0..step.hypotheses.len() {
                if i < display_hypotheses_num {
                    hypotheses_nums.push(
                        stack
                            .last()
                            .ok_or(Error::InvalidProofError)?
                            .display_step_number,
                    );
                }
                stack.pop();
            }
            stack.push(StackLine {
                statement: next_step,
                display_step_number: -1,
            });
        }

        if stack.last().unwrap().statement.split_whitespace().next() == Some("|-")
            || i == step_numbers_len - 1
        {
            if step.display_step_number == -1 {
                hypotheses_nums.reverse();
                proof_lines.push(ProofLine {
                    step_name: (proof_lines.len() + 1).to_string(),
                    hypotheses: hypotheses_nums.iter().map(|&i| i.to_string()).collect(),
                    reference: step.label.clone(),
                    indention: 1,
                    assertion: stack[stack.len() - 1].statement.clone(),
                });
                stack.last_mut().unwrap().display_step_number = next_hypotheses_num;
                next_hypotheses_num += 1;
            } else {
                stack.last_mut().unwrap().display_step_number = step.display_step_number;
            }
        }

        if save {
            proof_steps.push(ProofStep {
                label: String::new(),
                hypotheses: Vec::new(),
                statement: stack[stack.len() - 1].statement.clone(),
                display_step_number: stack.last().unwrap().display_step_number,
            });
        }

        // println!("\nStack:");
        // for stack_line in &stack {
        //     println!(
        //         "{}: {}",
        //         stack_line.display_step_number, stack_line.statement
        //     )
        // }
    }

    calc_indention(&mut proof_lines)?;

    Ok(TheoremPageData {
        theorem: theorem.clone(),
        theorem_number,
        proof_lines,
        preview_errors: None,
        preview_confirmations: None,
        preview_confirmations_recursive: None,
        last_theorem_label,
        next_theorem_label,
    })
}

#[derive(Debug)]
struct Tree {
    pub label: i32,
    pub nodes: Vec<Tree>,
}

fn calc_indention(proof_lines: &mut Vec<ProofLine>) -> Result<(), Error> {
    // calc tree rep
    let mut trees: Vec<Tree> = Vec::new();
    for (i, proof_line) in proof_lines.iter().enumerate() {
        let mut nodes: Vec<Tree> = Vec::new();
        for hypothesis in &proof_line.hypotheses {
            for tree_i in 0..trees.len() {
                if trees[tree_i].label
                    == hypothesis
                        .parse::<i32>()
                        .or(Err(Error::InternalLogicError))?
                {
                    nodes.push(trees.remove(tree_i));
                    break;
                }
            }
        }

        trees.push(Tree {
            label: (i + 1) as i32,
            nodes,
        })
    }

    // apply indention based on tree
    if trees.len() != 1 {
        println!("{:?}", trees);
        return Err(Error::InternalLogicError);
    }

    let mut indention = 1;
    let mut next_level: Vec<&Tree> = vec![trees.first().unwrap()];
    let mut current_level: Vec<&Tree>;

    while next_level.len() != 0 {
        current_level = next_level;
        next_level = Vec::new();

        for tree in current_level {
            proof_lines[(tree.label - 1) as usize].indention = indention;
            next_level.extend(tree.nodes.iter());
        }

        indention += 1;
    }

    Ok(())
}

fn calc_step_application<'a>(
    step: &'a ProofStep,
    stack: &Vec<StackLine>,
    metamath_data: &MetamathData,
) -> Result<(String, usize), Error> {
    if stack.len() < step.hypotheses.len() {
        return Err(Error::InvalidProofError);
    }
    let mut var_map: HashMap<&str, String> = HashMap::new();
    let mut display_hypotheses_num = 0;

    for (index, hypothesis) in (&step.hypotheses).iter().map(|s| s.as_str()).enumerate() {
        let tokens: Vec<&str> = hypothesis.split_whitespace().collect();
        let stack_str = stack[stack.len() - step.hypotheses.len() + index]
            .statement
            .as_str();

        if tokens.len() == 2 && tokens[0] != "|-" && metamath_data.is_variable(tokens[1]) {
            if tokens[0] != stack_str.split_whitespace().next().unwrap() {
                return Err(Error::InvalidProofError);
            }

            let mapped = statement_as_string_without_typecode(stack_str);
            var_map.insert(tokens[1], mapped);
        } else {
            display_hypotheses_num += 1;
            if stack_str != calc_substitution(hypothesis, &var_map) {
                return Err(Error::InvalidProofError);
            }
        }
    }
    Ok((
        calc_substitution(&step.statement, &var_map),
        display_hypotheses_num,
    ))
}

fn calc_substitution(statement: &str, var_mapping: &HashMap<&str, String>) -> String {
    let mut substitution = String::new();
    for token in statement.split_whitespace() {
        if !var_mapping.contains_key(token) {
            substitution.push_str(token);
        } else {
            substitution.push_str(var_mapping.get(token).unwrap().as_str());
        }
        substitution.push(' ');
    }
    substitution.pop();
    substitution
}

fn statement_as_string_without_typecode(statement: &str) -> String {
    let mut res = String::new();
    let mut first = true;

    for token in statement.split_whitespace() {
        if !first {
            res.push_str(token);
            res.push(' ');
        } else {
            first = false;
        }
    }

    res.pop();
    res
}

fn calc_proof_steps_and_numbers_uncompressed(
    theorem: &Theorem,
    metamath_data: &MetamathData,
) -> Result<(Vec<ProofStep>, Vec<(u32, bool)>), Error> {
    let mut proof_steps: Vec<ProofStep> = Vec::new();
    let mut proof_step_numbers: Vec<(u32, bool)> = Vec::new();

    if let Some(proof) = theorem.proof.as_ref() {
        for token in proof.split_ascii_whitespace() {
            if let Some((i, _)) = proof_steps
                .iter()
                .enumerate()
                .find(|(_, ps)| ps.label == token)
            {
                proof_step_numbers.push(((i + 1) as u32, false));
            } else {
                proof_steps.push(calc_proof_step_from_label(token, theorem, metamath_data)?);
                proof_step_numbers.push((proof_steps.len() as u32, false))
            }
        }
    }

    Ok((proof_steps, proof_step_numbers))
}

fn calc_proof_step_from_label(
    label: &str,
    theorem: &Theorem,
    metamath_data: &MetamathData,
) -> Result<ProofStep, Error> {
    if let Some(hyp) = theorem.hypotheses.iter().find(|h| h.label == label) {
        return Ok(ProofStep {
            label: label.to_string(),
            hypotheses: Vec::new(),
            statement: hyp.expression.clone(),
            display_step_number: -1,
        });
    }

    if let Some(floating_hypothesis) = metamath_data
        .database_header
        .floating_hypohesis_iter()
        .find(|fh| fh.label == label)
    {
        return Ok(ProofStep {
            label: label.to_string(),
            hypotheses: Vec::new(),
            statement: floating_hypothesis.to_assertions_string(),
            display_step_number: -1,
        });
    }

    if let Some(theorem) = metamath_data
        .database_header
        .theorem_iter()
        .find(|t| t.label == label)
    {
        let label_theorem_hypotheses = calc_all_hypotheses_of_theorem(theorem, metamath_data);
        return Ok(ProofStep {
            label: label.to_string(),
            hypotheses: label_theorem_hypotheses
                .into_iter()
                .map(|(hyp, _label)| hyp)
                .collect(),
            statement: theorem.assertion.clone(),
            display_step_number: -1,
        });
    }

    Err(Error::NotFoundError)
}

fn calc_proof_step_numbers_compressed(theorem: &Theorem) -> Result<Vec<(u32, bool)>, Error> {
    let proof = match theorem.proof.as_ref() {
        Some(proof) => &**proof,
        None => return Ok(Vec::new()),
    };

    let mut passed_labels = false;
    let mut compressed_steps = String::new();

    for token in proof.split_whitespace() {
        match token {
            ")" => passed_labels = true,
            s if passed_labels => {
                compressed_steps.push_str(s);
            }
            _ => {}
        }
    }

    let mut step_numbers = Vec::new();

    let mut char_iter = compressed_steps.chars();

    let mut current_compressed_num = String::new();

    while let Some(character) = char_iter.next() {
        match character {
            c @ 'A'..='T' => {
                current_compressed_num.push(c);
                step_numbers.push((compressed_num_to_num(&current_compressed_num)?, false));
                current_compressed_num = String::new();
            }
            c @ 'U'..='Y' => current_compressed_num.push(c),
            'Z' if step_numbers.len() != 0 => {
                let len = step_numbers.len();
                step_numbers[len - 1].1 = true;
            }
            _ => return Err(Error::InvalidFormatError),
        }
    }

    Ok(step_numbers)
}

fn compressed_num_to_num(compressed_num: &str) -> Result<u32, Error> {
    let mut first = true;
    let mut num = 0;
    let mut multiplier = 20;
    for ch in compressed_num.chars().rev() {
        match ch {
            ch @ 'A'..='T' if first => {
                num = (ch as u32) - 64;
                first = false;
            }
            ch @ 'U'..='Y' if !first => {
                num += ((ch as u32) - 84) * multiplier;
                multiplier *= 5;
            }
            _ => return Err(Error::InvalidFormatError),
        }
    }
    if num == 0 {
        return Err(Error::InvalidFormatError);
    }
    Ok(num)
}

fn calc_proof_steps_compressed(
    theorem: &Theorem,
    metamath_data: &MetamathData,
) -> Result<Vec<ProofStep>, Error> {
    let proof = match theorem.proof.as_ref() {
        Some(proof) => &**proof,
        None => return Ok(Vec::new()),
    };

    let mut steps = Vec::new();

    let hypotheses = calc_all_hypotheses_of_theorem(theorem, metamath_data);

    for (hypothesis, label) in hypotheses {
        steps.push(ProofStep {
            label,
            hypotheses: Vec::new(),
            statement: hypothesis,
            display_step_number: -1,
        })
    }

    for token in proof.split_whitespace() {
        match token {
            "(" => {}
            ")" => break,
            label => {
                let theorem_option = metamath_data.database_header.find_theorem_by_label(label);
                //.ok_or(Error::NotFoundError)?;
                if let Some(theorem) = theorem_option {
                    let label_theorem_hypotheses =
                        calc_all_hypotheses_of_theorem(theorem, metamath_data);
                    steps.push(ProofStep {
                        label: label.to_string(),
                        hypotheses: label_theorem_hypotheses
                            .into_iter()
                            .map(|(hyp, _label)| hyp)
                            .collect(),
                        statement: theorem.assertion.clone(),
                        display_step_number: -1,
                    });
                } else {
                    let floating_hypothesis =
                        get_floating_hypothesis_by_label(metamath_data, label)
                            .ok_or(Error::NotFoundError)?;

                    steps.push(ProofStep {
                        label: label.to_string(),
                        hypotheses: Vec::new(),
                        statement: floating_hypothesis.to_assertions_string(),
                        display_step_number: -1,
                    });
                }
            }
        };
    }

    Ok(steps)
}

fn calc_all_hypotheses_of_theorem(
    theorem: &Theorem,
    metamath_data: &MetamathData,
) -> Vec<(String, String)> {
    let mut hypotheses = Vec::new();

    // Calculate variables occuring in assertion and hypotheses
    let variables = calc_variables_of_theorem(theorem, metamath_data);

    // Calculate proof steps of floating hypotheses
    for floating_hypothesis in &metamath_data.optimized_data.floating_hypotheses {
        for &variable in &variables {
            if floating_hypothesis.variable == variable {
                let mut statement = floating_hypothesis.typecode.clone();
                statement.push(' ');
                statement.push_str(&floating_hypothesis.variable);
                hypotheses.push((statement, floating_hypothesis.label.clone()));
                break;
            }
        }
    }

    // Calculate proof steps of essential hypotheses
    for hypothesis in &theorem.hypotheses {
        hypotheses.push((hypothesis.expression.clone(), hypothesis.label.clone()));
    }

    hypotheses
}

fn calc_variables_of_theorem<'a>(
    theorem: &'a Theorem,
    metamath_data: &MetamathData,
) -> Vec<&'a str> {
    let mut variables = get_variables_from_statement(&theorem.assertion, metamath_data);

    for hypothesis in &theorem.hypotheses {
        let hypothesis_vars = get_variables_from_statement(&hypothesis.expression, metamath_data);
        for var in hypothesis_vars {
            if !variables.contains(&var) {
                variables.push(var);
            }
        }
    }
    variables
}

fn get_variables_from_statement<'a>(
    statement: &'a str,
    metamath_data: &MetamathData,
) -> Vec<&'a str> {
    let mut vars = Vec::new();
    for token in statement.split_whitespace() {
        if !vars.contains(&token) && metamath_data.is_variable(token) {
            vars.push(token);
        }
    }
    vars
}
