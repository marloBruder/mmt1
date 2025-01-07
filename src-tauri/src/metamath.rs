use crate::{
    database::{
        constant::set_constants_database,
        floating_hypothesis::set_floating_hypotheses_database,
        in_progress_theorem::delete_in_progress_theorem_database,
        theorem::{add_theorem_database, calc_db_index_for_theorem},
        variable::set_variables_database,
    },
    local_state::{
        constant::set_constants_local,
        floating_hypothesis::set_floating_hypotheses_local,
        in_progress_theorem::delete_in_progress_theorem_local,
        theorem::{add_theorem_local, get_theorem_insert_position},
        variable::set_variables_local,
    },
    model::{
        Constant, FloatingHypohesis, Hypothesis, InProgressTheorem, MetamathData, ProofLine,
        Theorem, TheoremPageData, TheoremPath, Variable,
    },
    AppState, Error,
};
use std::collections::HashMap;
use tauri::{async_runtime::Mutex, State};

pub mod parse;

#[tauri::command]
pub async fn turn_into_theorem(
    state: tauri::State<'_, Mutex<AppState>>,
    in_progress_theorem: InProgressTheorem,
    position_name: &str,
) -> Result<TheoremPath, Error> {
    if !in_progress_theorem.text.is_ascii() {
        return Err(Error::InvalidCharactersError);
    }

    let mut last_token: Option<&str> = None;

    let mut name: Option<String> = None;
    let mut description = String::from("");
    let mut disjoints: Vec<String> = Vec::new();
    let mut hypotheses: Vec<Hypothesis> = Vec::new();
    let mut assertion: Option<String> = None;
    let mut proof: Option<String> = None;

    let mut token_iter = in_progress_theorem.text.split_whitespace();
    while let Some(token) = token_iter.next() {
        match token {
            "$(" => description = get_next_as_string_until(&mut token_iter, "$)"),
            "$d" => {
                let disjoint_cond = get_next_as_string_until(&mut token_iter, "$.");
                disjoints.push(disjoint_cond);
            }
            "$e" => {
                let label = last_token.ok_or(Error::InvalidFormatError)?.to_string();
                let hypothesis = get_next_as_string_until(&mut token_iter, "$.");
                hypotheses.push(Hypothesis { label, hypothesis })
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

    if name != in_progress_theorem.name {
        return Err(Error::InvalidFormatError);
    }

    let mut app_state = state.lock().await;
    let db_state = app_state.db_state.as_mut().ok_or(Error::NoDatabaseError)?;

    let insert_path = get_theorem_insert_position(&db_state.metamath_data, position_name)?;

    add_theorem_local(
        &mut db_state.metamath_data,
        &name,
        &description,
        &disjoints,
        &hypotheses,
        &assertion,
        proof.as_deref(),
        &insert_path,
    )?;

    let db_index = calc_db_index_for_theorem(&db_state.metamath_data, &insert_path)?;

    add_theorem_database(
        &mut db_state.db_conn,
        db_index,
        &name,
        &description,
        &disjoints,
        &hypotheses,
        &assertion,
        proof.as_deref(),
    )
    .await
    .or(Err(Error::SqlError))?;

    delete_in_progress_theorem_database(&mut db_state.db_conn, &name)
        .await
        .or(Err(Error::SqlError))?;

    delete_in_progress_theorem_local(&mut db_state.metamath_data, &name);

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

#[tauri::command]
pub async fn text_to_constants(
    state: tauri::State<'_, Mutex<AppState>>,
    text: &str,
) -> Result<Vec<Constant>, Error> {
    if !text.is_ascii() {
        return Err(Error::InvalidCharactersError);
    }

    let symbols = text_to_constant_or_variable_symbols(text, true)?;

    let mut app_state = state.lock().await;
    let db_state = app_state.db_state.as_mut().ok_or(Error::NoDatabaseError)?;

    set_constants_database(&mut db_state.db_conn, &symbols)
        .await
        .or(Err(Error::SqlError))?;

    set_constants_local(&mut db_state.metamath_data, &symbols);

    let mut constants = Vec::new();

    for symbol in symbols {
        constants.push(Constant {
            symbol: symbol.to_string(),
        })
    }

    Ok(constants)
}

#[tauri::command]
pub async fn text_to_variables(
    state: tauri::State<'_, Mutex<AppState>>,
    text: &str,
) -> Result<Vec<Variable>, Error> {
    if !text.is_ascii() {
        return Err(Error::InvalidCharactersError);
    }

    let symbols = text_to_constant_or_variable_symbols(text, false)?;

    let mut app_state = state.lock().await;
    let db_state = app_state.db_state.as_mut().ok_or(Error::NoDatabaseError)?;

    set_variables_database(&mut db_state.db_conn, &symbols)
        .await
        .or(Err(Error::SqlError))?;

    set_variables_local(&mut db_state.metamath_data, &symbols);

    let mut variables = Vec::new();

    for symbol in symbols {
        variables.push(Variable {
            symbol: symbol.to_string(),
        })
    }

    Ok(variables)
}

// Takes a text and returns references to the symbols between
// "$c" and "$.", if constant is true,
// "$v" and "$.", if constant is false
// If there is a string not between these, the function returns an Error
fn text_to_constant_or_variable_symbols(text: &str, constant: bool) -> Result<Vec<&str>, Error> {
    let mut symbols = Vec::new();

    // True if token is after "$c", but before the next "$."
    let mut within_statement = false;

    for token in text.split_whitespace() {
        if !within_statement {
            match token {
                "$c" if constant => within_statement = true,
                "$v" if !constant => within_statement = true,
                _ => return Err(Error::InvalidFormatError),
            }
        } else {
            match token {
                "$." => within_statement = false,
                s => symbols.push(s),
            }
        }
    }

    Ok(symbols)
}

#[tauri::command]
pub async fn text_to_floating_hypotheses(
    state: State<'_, Mutex<AppState>>,
    text: &str,
) -> Result<Vec<FloatingHypohesis>, Error> {
    if !text.is_ascii() {
        return Err(Error::InvalidCharactersError);
    }

    let mut floating_hypotheses = Vec::new();

    let mut token_iter = text.split_whitespace();

    let mut next_label: Option<&str> = None;

    while let Some(token) = token_iter.next() {
        match (token, next_label) {
            ("$f", Some(label)) => {
                let typecode = token_iter.next();
                let variable = token_iter.next();

                if token_iter.next() == Some("$.") {
                    floating_hypotheses.push(FloatingHypohesis {
                        label: label.to_string(),
                        // Safe unwraps, because the if branch requires a later call of next to have returned Some
                        typecode: typecode.unwrap().to_string(),
                        variable: variable.unwrap().to_string(),
                    });
                    next_label = None;
                } else {
                    return Err(Error::InvalidFormatError);
                }
            }
            (label, None) => next_label = Some(label),
            (_, _) => return Err(Error::InvalidFormatError),
        }
    }

    let mut app_state = state.lock().await;
    let db_state = app_state.db_state.as_mut().ok_or(Error::NoDatabaseError)?;

    set_floating_hypotheses_database(&mut db_state.db_conn, &floating_hypotheses)
        .await
        .or(Err(Error::SqlError))?;

    set_floating_hypotheses_local(&mut db_state.metamath_data, &floating_hypotheses);

    Ok(floating_hypotheses)
}

#[derive(Debug)]
struct ProofStep {
    pub label: String,
    pub hypotheses: Vec<String>,
    pub statement: String,
}

#[derive(Debug)]
struct StackLine {
    pub statement: String,
    pub display_step_number: i32,
}

pub fn calc_theorem_page_data(
    theorem: &Theorem,
    metamath_data: &MetamathData,
) -> Result<TheoremPageData, Error> {
    if theorem.proof == None {
        return Ok(TheoremPageData {
            theorem: theorem.clone(),
            proof_lines: Vec::new(),
        });
    }
    let mut proof_lines = Vec::new();

    let mut proof_steps = calc_proof_steps(theorem, metamath_data)?;
    let step_numbers = calc_proof_step_numbers(theorem)?;

    // println!("Steps:\n{:?}\n", proof_steps);
    // println!("Numbers:\n{:?}\n", step_numbers);

    let mut stack: Vec<StackLine> = Vec::new();

    let mut next_hypotheses_num = 1;

    for (step_num, save) in step_numbers {
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

        if stack[stack.len() - 1].statement.split_whitespace().next() == Some("|-") {
            hypotheses_nums.reverse();
            proof_lines.push(ProofLine {
                hypotheses: hypotheses_nums,
                reference: step.label.clone(),
                indention: 1,
                assertion: stack[stack.len() - 1].statement.clone(),
            });
            update_indention(&mut proof_lines);
            stack
                .last_mut()
                .ok_or(Error::InvalidProofError)?
                .display_step_number = next_hypotheses_num;
            next_hypotheses_num += 1;
        }

        if save {
            proof_steps.push(ProofStep {
                label: String::new(),
                hypotheses: Vec::new(),
                statement: stack[stack.len() - 1].statement.clone(),
            });
        }

        // println!("Stack:\n{:?}\n", stack);
    }

    Ok(TheoremPageData {
        theorem: theorem.clone(),
        proof_lines,
    })
}

fn update_indention(proof_lines: &mut Vec<ProofLine>) {
    let mut update: Vec<usize> = Vec::new();
    let mut find_update = Vec::new();
    find_update.push(proof_lines.len() - 1);

    while let Some(&find_new) = find_update.first() {
        for &hypothesis in &proof_lines[find_new].hypotheses {
            let potential_update = (hypothesis as usize) - 1;
            if !update.contains(&potential_update) {
                find_update.push(potential_update);
                update.push(potential_update);
            }
        }
        find_update.swap_remove(0);
    }

    for i in update {
        proof_lines[i].indention += 1;
    }
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

        if tokens.len() == 2 && tokens[0] != "|-" && is_variable(&tokens[1], metamath_data) {
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

fn is_variable(symbol: &str, metamath_data: &MetamathData) -> bool {
    for variable in &metamath_data.variables {
        if variable.symbol == symbol {
            return true;
        }
    }
    false
}

fn calc_proof_step_numbers(theorem: &Theorem) -> Result<Vec<(u32, bool)>, Error> {
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

fn calc_proof_steps(
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
        })
    }

    for token in proof.split_whitespace() {
        match token {
            "(" => {}
            ")" => break,
            label => {
                let label_theorem = metamath_data
                    .theorem_list_header
                    .find_theorem_by_name(label)
                    .ok_or(Error::NotFoundError)?;
                let label_theorem_hypotheses =
                    calc_all_hypotheses_of_theorem(label_theorem, metamath_data);
                steps.push(ProofStep {
                    label: label.to_string(),
                    hypotheses: label_theorem_hypotheses
                        .iter()
                        .map(|(hyp, _label)| hyp.clone())
                        .collect(),
                    statement: label_theorem.assertion.clone(),
                })
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
    for floating_hypothesis in &metamath_data.floating_hypotheses {
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
        hypotheses.push((hypothesis.hypothesis.clone(), hypothesis.label.clone()));
    }

    hypotheses
}

fn calc_variables_of_theorem<'a>(
    theorem: &'a Theorem,
    metamath_data: &MetamathData,
) -> Vec<&'a str> {
    let mut variables = get_variables_from_statement(&theorem.assertion, metamath_data);

    for hypothesis in &theorem.hypotheses {
        let hypothesis_vars = get_variables_from_statement(&hypothesis.hypothesis, metamath_data);
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
        if !vars.contains(&token) && is_variable(token, metamath_data) {
            vars.push(token);
        }
    }
    vars
}
