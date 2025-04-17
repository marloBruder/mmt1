use std::collections::HashSet;

use crate::{
    model::{
        Comment, Constant, FloatingHypohesis, Header, HeaderPath, Hypothesis, MetamathData,
        Statement, Theorem, Variable,
    },
    AppState, Error,
};
use tauri::async_runtime::Mutex;

struct MmpStructuredInfo {
    constants: Vec<Constant>,
    variables: Vec<Vec<Variable>>,
    floating_hypotheses: Vec<FloatingHypohesis>,
    theorem_label: Option<String>,
    axiom_label: Option<String>,
    header: Option<(String, String)>,
    distinct_vars: Vec<String>,
    mmj2_steps: Vec<(String, String)>,
    allow_discouraged: bool,
    locate_after: Option<LocateAfter>,
    comments: Vec<Comment>,
}

enum LocateAfter {
    LocateAfter(String),
    LocateAfterConst(String),
    LocateAfterVar(String),
}

#[tauri::command]
pub async fn add_to_database(
    state: tauri::State<'_, Mutex<AppState>>,
    text: &str,
) -> Result<(), Error> {
    if !text.is_ascii() {
        return Err(Error::InvalidCharactersError);
    }

    let statements = text_to_statements(text)?;
    let mmp_structured_info = statements_to_mmp_structured_info(statements)?;

    let mut app_state = state.lock().await;
    let mm_data = app_state.metamath_data.as_mut().ok_or(Error::NoMmDbError)?;

    if mmp_structured_info.theorem_label.is_some() {
        add_theorem_to_database(mm_data, mmp_structured_info)?;
    } else if mmp_structured_info.axiom_label.is_some() {
        add_axiom_to_database(mm_data, mmp_structured_info)?;
    } else if mmp_structured_info.header.is_some() {
        add_header_to_database(mm_data, mmp_structured_info)?;
    } else if !mmp_structured_info.floating_hypotheses.is_empty() {
        add_floating_hypothesis_to_database(mm_data, mmp_structured_info)?;
    } else if !mmp_structured_info.variables.is_empty() {
        add_variables_to_database(mm_data, mmp_structured_info)?;
    } else if !mmp_structured_info.constants.is_empty() {
        add_constants_to_database(mm_data, mmp_structured_info)?;
    } else if mmp_structured_info.comments.len() > 0 {
        add_comment_to_database(mm_data, mmp_structured_info)?;
    }

    Ok(())
}

fn add_theorem_to_database(
    mm_data: &mut MetamathData,
    mmp_structured_info: MmpStructuredInfo,
) -> Result<(), Error> {
    if mmp_structured_info.axiom_label.is_some()
        || !mmp_structured_info.constants.is_empty()
        || mmp_structured_info.header.is_some()
    {
        return Err(Error::StatementOutOfPlaceError);
    }

    let mut new_symbols: Vec<&str> = Vec::new();

    let thoerem_label = mmp_structured_info
        .theorem_label
        .ok_or(Error::InternalLogicError)?;

    new_symbols.push(&thoerem_label);

    if mmp_structured_info.mmj2_steps.is_empty() {
        return Err(Error::MissingMmj2StepsError);
    }

    let mut hypotheses: Vec<Hypothesis> = Vec::new();
    let mut assertion: String = String::new();

    let mut mmj2_steps_processed: Vec<Mmj2StepProcessed> = Vec::new();

    for (prefix, expression) in &mmp_structured_info.mmj2_steps {
        let prefix_parts: Vec<&str> = prefix.split(':').collect();
        if prefix_parts.len() != 3 {
            return Err(Error::InvalidMmj2StepPrefixError);
        }

        let label = *prefix_parts.get(2).unwrap();

        let prefix_step_name = prefix_parts.get(0).unwrap();

        let mut hypothesis = false;
        let step_name: &str;

        if prefix_step_name.starts_with('h') {
            hypothesis = true;
            step_name = prefix_step_name.split_at(1).1;
            hypotheses.push(Hypothesis {
                label: label.to_string(),
                hypothesis: expression.to_string(),
            });
            new_symbols.push(label);
        } else {
            step_name = prefix_step_name;
        }

        if step_name == "qed" {
            assertion = expression.clone();
        }

        if step_name.contains(',') || step_name == "" {
            return Err(Error::InvalidMmj2StepPrefixError);
        }

        if mmj2_steps_processed
            .iter()
            .find(|msc| msc.step_name == step_name)
            .is_some()
        {
            return Err(Error::DuplicateStepNameError);
        }

        if !hypothesis
            && mm_data
                .database_header
                .theorem_iter()
                .find(|t| t.label == label)
                .is_none()
        {
            return Err(Error::TheoremLabelNotFoundError);
        }

        let prefix_hyps = prefix_parts.get(1).unwrap();

        let mut hyps: Vec<usize> = Vec::new();

        if !prefix_hyps.is_empty() {
            let hyp_strs: Vec<&str> = prefix_hyps.split(',').collect();
            for hyp_str in &hyp_strs {
                for (i, previous_step) in mmj2_steps_processed.iter().enumerate() {
                    if previous_step.step_name == *hyp_str {
                        hyps.push(i);
                    }
                }
            }

            if hyp_strs.len() != hyps.len() {
                return Err(Error::InvalidMmj2StepPrefixError);
            }
        }

        let expression_vec = mm_data
            .optimized_data
            .symbol_number_mapping
            .expression_to_number_vec(expression)
            .or(Err(Error::InactiveMathSymbolError))?;

        mmj2_steps_processed.push(Mmj2StepProcessed {
            hypothesis,
            step_name,
            hyps,
            label,
            expression: expression_vec,
        });
    }

    if assertion == "" {
        return Err(Error::MissingQedStepError);
    }

    if !mm_data.valid_new_symbols(&new_symbols) {
        return Err(Error::DuplicateSymbolError);
    }

    if !all_different_strs(&new_symbols) {
        return Err(Error::DuplicateSymbolError);
    }

    let proof = calc_proof(mmj2_steps_processed, mm_data)?;

    add_statement(
        &mut mm_data.database_header,
        &mmp_structured_info.locate_after,
        Statement::TheoremStatement(Theorem {
            label: thoerem_label,
            description: mmp_structured_info
                .comments
                .into_iter()
                .next()
                .map(|c| c.text)
                .unwrap_or(String::new()),
            disjoints: mmp_structured_info.distinct_vars,
            assertion,
            hypotheses,
            proof,
        }),
    );

    Ok(())
}

#[derive(Debug)]
struct Mmj2StepProcessed<'a> {
    hypothesis: bool,
    step_name: &'a str,
    hyps: Vec<usize>,
    label: &'a str,
    expression: Vec<u32>,
}

/**
Assumes:
- all labels are correct theorem labels
- all hyps are numbers between 0 and ` mmj2_steps_processed.len() - 1 `
- all hyps are lower than the index of the step they belong to (and therefore don't point recursivly at each other)
*/
fn calc_proof(
    mmj2_steps_processed: Vec<Mmj2StepProcessed>,
    mm_data: &MetamathData,
) -> Result<Option<String>, Error> {
    Ok(None)
}

// TODO: make sure step names are unique
fn add_axiom_to_database(
    mm_data: &mut MetamathData,
    mmp_structured_info: MmpStructuredInfo,
) -> Result<(), Error> {
    if !mmp_structured_info.constants.is_empty()
        || mmp_structured_info.allow_discouraged
        || mmp_structured_info.header.is_some()
    {
        return Err(Error::StatementOutOfPlaceError);
    }

    let mut symbols: Vec<&str> = Vec::new();

    let axiom_label = mmp_structured_info
        .axiom_label
        .ok_or(Error::InternalLogicError)?;

    symbols.push(&axiom_label);

    if mmp_structured_info.mmj2_steps.is_empty() {
        return Err(Error::MissingMmj2StepsError);
    }

    let mut hypotheses: Vec<Hypothesis> = Vec::new();

    let mut assertion: String = String::new();

    let mmj2_step_count = mmp_structured_info.mmj2_steps.len();

    for (i, (prefix, expression)) in mmp_structured_info.mmj2_steps.into_iter().enumerate() {
        let prefix_parts: Vec<&str> = prefix.split(':').collect();
        if i != mmj2_step_count - 1 {
            if prefix_parts.len() != 3
                || !prefix_parts.get(0).unwrap().starts_with('h')
                || prefix_parts.get(1).unwrap().len() != 0
                || prefix_parts.get(2).unwrap().len() == 0
                || expression.len() == 0
            {
                return Err(Error::InvalidMmj2StepPrefixError);
            }
            hypotheses.push(Hypothesis {
                label: prefix_parts.get(2).unwrap().to_string(),
                hypothesis: expression,
            });
        } else {
            if prefix_parts.len() != 3
                || prefix_parts.get(0).unwrap() != &"qed"
                || prefix_parts.get(1).unwrap().len() != 0
                || prefix_parts.get(2).unwrap().len() != 0
            {
                return Err(Error::InvalidMmj2StepPrefixError);
            }
            assertion = expression;
        }
    }
    for hypothesis in &hypotheses {
        symbols.push(&hypothesis.label)
    }

    if !mm_data.valid_new_symbols(&symbols) {
        return Err(Error::DuplicateSymbolError);
    }

    if !all_different_strs(&symbols) {
        return Err(Error::DuplicateSymbolError);
    }

    add_statement(
        &mut mm_data.database_header,
        &mmp_structured_info.locate_after,
        Statement::TheoremStatement(Theorem {
            label: axiom_label,
            description: mmp_structured_info
                .comments
                .into_iter()
                .next()
                .map(|c| c.text)
                .unwrap_or(String::new()),
            disjoints: mmp_structured_info.distinct_vars,
            assertion,
            hypotheses,
            proof: None,
        }),
    );

    Ok(())
}

fn add_header_to_database(
    mm_data: &mut MetamathData,
    mmp_structured_info: MmpStructuredInfo,
) -> Result<(), Error> {
    if !mmp_structured_info.floating_hypotheses.is_empty()
        || !mmp_structured_info.constants.is_empty()
        || !mmp_structured_info.variables.is_empty()
        || mmp_structured_info.allow_discouraged
        || !mmp_structured_info.distinct_vars.is_empty()
        || !mmp_structured_info.mmj2_steps.is_empty()
        || mmp_structured_info.locate_after.is_some()
    {
        return Err(Error::StatementOutOfPlaceError);
    }

    let (header_path_string, header_title) = mmp_structured_info
        .header
        .ok_or(Error::InternalLogicError)?;

    let mut header_path =
        HeaderPath::from_str(&header_path_string).or(Err(Error::InvalidHeaderPathStringError))?;

    let last_header_index = header_path.path.pop().ok_or(Error::InternalLogicError)?;

    let header = header_path
        .resolve_mut(&mut mm_data.database_header)
        .ok_or(Error::InvalidHeaderPathStringError)?;

    if last_header_index < header.subheaders.len() {
        header
            .subheaders
            .get_mut(last_header_index)
            .ok_or(Error::InternalLogicError)?
            .title = header_title;
    } else if last_header_index == header.subheaders.len() {
        header.subheaders.push(Header {
            title: header_title,
            content: Vec::new(),
            subheaders: Vec::new(),
        });
    } else if last_header_index > header.subheaders.len() {
        return Err(Error::InvalidHeaderPathStringError);
    }

    Ok(())
}

fn add_floating_hypothesis_to_database(
    mm_data: &mut MetamathData,
    mmp_structured_info: MmpStructuredInfo,
) -> Result<(), Error> {
    if mmp_structured_info.floating_hypotheses.len() > 1
        || !mmp_structured_info.constants.is_empty()
        || !mmp_structured_info.variables.is_empty()
        || mmp_structured_info.allow_discouraged
        || !mmp_structured_info.distinct_vars.is_empty()
        || !mmp_structured_info.mmj2_steps.is_empty()
    {
        return Err(Error::StatementOutOfPlaceError);
    }

    let flaoting_hypothesis = mmp_structured_info
        .floating_hypotheses
        .into_iter()
        .next()
        .ok_or(Error::InternalLogicError)?;

    if !mm_data.valid_new_symbols(&vec![&*flaoting_hypothesis.label]) {
        return Err(Error::DuplicateSymbolError);
    }

    add_statement(
        &mut mm_data.database_header,
        &mmp_structured_info.locate_after,
        Statement::FloatingHypohesisStatement(flaoting_hypothesis),
    );

    mm_data.recalc_optimized_floating_hypotheses_after_one_new()?;
    mm_data.recalc_symbol_number_mapping();

    Ok(())
}

fn add_variables_to_database(
    mm_data: &mut MetamathData,
    mmp_structured_info: MmpStructuredInfo,
) -> Result<(), Error> {
    if mmp_structured_info.variables.len() > 1
        || !mmp_structured_info.constants.is_empty()
        || mmp_structured_info.allow_discouraged
        || !mmp_structured_info.distinct_vars.is_empty()
        || !mmp_structured_info.mmj2_steps.is_empty()
    {
        return Err(Error::StatementOutOfPlaceError);
    }

    let variables = mmp_structured_info
        .variables
        .into_iter()
        .next()
        .ok_or(Error::InternalLogicError)?;

    let var_strs = variables.iter().map(|v| &*v.symbol).collect();

    if !mm_data.valid_new_symbols(&var_strs) {
        return Err(Error::DuplicateSymbolError);
    }

    if !all_different_strs(&var_strs) {
        return Err(Error::DuplicateSymbolError);
    }

    for var in &variables {
        mm_data.optimized_data.variables.insert(var.symbol.clone());
    }

    add_statement(
        &mut mm_data.database_header,
        &mmp_structured_info.locate_after,
        Statement::VariableStatement(variables),
    );

    mm_data.recalc_symbol_number_mapping();

    Ok(())
}

fn add_constants_to_database(
    mm_data: &mut MetamathData,
    mmp_structured_info: MmpStructuredInfo,
) -> Result<(), Error> {
    if mmp_structured_info.allow_discouraged
        || !mmp_structured_info.distinct_vars.is_empty()
        || !mmp_structured_info.mmj2_steps.is_empty()
    {
        return Err(Error::StatementOutOfPlaceError);
    }

    let const_strs = mmp_structured_info
        .constants
        .iter()
        .map(|c| &*c.symbol)
        .collect();

    if !mm_data.valid_new_symbols(&const_strs) {
        return Err(Error::DuplicateSymbolError);
    }

    if !all_different_strs(&const_strs) {
        return Err(Error::DuplicateSymbolError);
    }

    add_statement(
        &mut mm_data.database_header,
        &mmp_structured_info.locate_after,
        Statement::ConstantStatement(mmp_structured_info.constants),
    );

    mm_data.recalc_symbol_number_mapping();

    Ok(())
}

fn add_comment_to_database(
    mm_data: &mut MetamathData,
    mmp_structured_info: MmpStructuredInfo,
) -> Result<(), Error> {
    if mmp_structured_info.allow_discouraged
        || !mmp_structured_info.distinct_vars.is_empty()
        || !mmp_structured_info.mmj2_steps.is_empty()
    {
        return Err(Error::StatementOutOfPlaceError);
    }

    add_statement(
        &mut mm_data.database_header,
        &mmp_structured_info.locate_after,
        Statement::CommentStatement(
            mmp_structured_info
                .comments
                .into_iter()
                .next()
                .ok_or(Error::InternalLogicError)?,
        ),
    );

    Ok(())
}

fn all_different_strs(strs: &Vec<&str>) -> bool {
    let mut hash_set: HashSet<&str> = HashSet::new();

    for str in strs {
        if !hash_set.insert(*str) {
            return false;
        }
    }

    true
}

fn add_statement(
    header: &mut Header,
    locate_after: &Option<LocateAfter>,
    statement: Statement,
) -> Option<Statement> {
    match locate_after {
        Some(loc_after) => add_statement_locate_after(header, loc_after, statement),
        None => {
            add_statement_at_end(header, statement);
            None
        }
    }
}

fn add_statement_locate_after(
    header: &mut Header,
    locate_after: &LocateAfter,
    mut statement: Statement,
) -> Option<Statement> {
    for i in 0..header.content.len() {
        match locate_after {
            LocateAfter::LocateAfterConst(s) => {
                if let Some(Statement::ConstantStatement(constants)) = header.content.get(i) {
                    if constants.iter().find(|c| c.symbol == *s).is_some() {
                        header.content.insert(i + 1, statement);
                        return None;
                    }
                }
            }
            LocateAfter::LocateAfterVar(s) => {
                if let Some(Statement::VariableStatement(variables)) = header.content.get(i) {
                    if variables.iter().find(|c| c.symbol == *s).is_some() {
                        header.content.insert(i + 1, statement);
                        return None;
                    }
                }
            }
            LocateAfter::LocateAfter(s) => {
                if let Some(Statement::TheoremStatement(theorem)) = header.content.get(i) {
                    if theorem.label == *s {
                        header.content.insert(i + 1, statement);
                        return None;
                    }
                } else if let Some(Statement::FloatingHypohesisStatement(floating_hypothesis)) =
                    header.content.get(i)
                {
                    if floating_hypothesis.label == *s {
                        header.content.insert(i + 1, statement);
                        return None;
                    }
                }
            }
        }
    }

    for subheader in &mut header.subheaders {
        statement = match add_statement_locate_after(subheader, locate_after, statement) {
            Some(s) => s,
            None => {
                return None;
            }
        };
    }

    Some(statement)
}

fn add_statement_at_end(header: &mut Header, statement: Statement) {
    let mut last_header = header;
    while last_header.subheaders.len() > 0 {
        last_header = last_header.subheaders.last_mut().unwrap();
    }

    last_header.content.push(statement);
}

fn statements_to_mmp_structured_info(
    statements: Vec<Vec<&str>>,
) -> Result<MmpStructuredInfo, Error> {
    let mut constants: Vec<Constant> = Vec::new();
    let mut variables: Vec<Vec<Variable>> = Vec::new();
    let mut floating_hypotheses: Vec<FloatingHypohesis> = Vec::new();
    let mut theorem_label: Option<String> = None;
    let mut axiom_label: Option<String> = None;
    let mut header: Option<(String, String)> = None;
    let mut distinct_vars: Vec<String> = Vec::new();
    let mut mmj2_steps: Vec<(String, String)> = Vec::new();
    let mut allow_discouraged: bool = false;
    let mut locate_after: Option<LocateAfter> = None;
    let mut comments: Vec<Comment> = Vec::new();

    for tokens in statements {
        // "\n" denote an empty line, which are only relevant for comments
        let mut token_iter = tokens.iter().map(|t| *t).filter(|t| *t != "\n");

        match token_iter.next().ok_or(Error::InternalLogicError)? {
            "$header" => {
                if header.is_some() {
                    return Err(Error::MultipleHeaderStatementError);
                }

                let pos = token_iter
                    .next()
                    .ok_or(Error::TooFewHeaderTokensError)?
                    .to_string();
                let mut title = token_iter.fold(String::new(), |mut s, t| {
                    s.push_str(t);
                    s.push(' ');
                    s
                });
                title.pop();

                if title.len() == 0 {
                    return Err(Error::TooFewHeaderTokensError);
                }

                header = Some((pos, title))
            }
            "$c" => {
                if !constants.is_empty() {
                    return Err(Error::StatementOutOfPlaceError);
                }

                for token in token_iter {
                    constants.push(Constant {
                        symbol: token.to_string(),
                    });
                }

                if constants.is_empty() {
                    return Err(Error::EmptyConstStatementError);
                }
            }
            "$v" => {
                let mut variable_vec: Vec<Variable> = Vec::new();

                for token in token_iter {
                    variable_vec.push(Variable {
                        symbol: token.to_string(),
                    });
                }

                if variable_vec.is_empty() {
                    return Err(Error::EmptyVarStatementError);
                }

                variables.push(variable_vec);
            }
            "$f" => {
                let label = token_iter
                    .next()
                    .ok_or(Error::FloatHypStatementFormatError)?
                    .to_string();
                let typecode = token_iter
                    .next()
                    .ok_or(Error::FloatHypStatementFormatError)?
                    .to_string();
                let variable = token_iter
                    .next()
                    .ok_or(Error::FloatHypStatementFormatError)?
                    .to_string();

                floating_hypotheses.push(FloatingHypohesis {
                    label,
                    typecode,
                    variable,
                });

                if token_iter.next().is_some() {
                    return Err(Error::FloatHypStatementFormatError);
                }
            }
            "$theorem" => {
                if theorem_label.is_some() {
                    return Err(Error::MultipleTheoremLabelError);
                }

                theorem_label = Some(
                    token_iter
                        .next()
                        .ok_or(Error::MissingTheoremLabelError)?
                        .to_string(),
                );

                if token_iter.next().is_some() {
                    return Err(Error::TooManyTheoremLabelTokensError);
                }
            }
            "$axiom" => {
                if axiom_label.is_some() {
                    return Err(Error::MultipleAxiomLabelError);
                }

                axiom_label = Some(
                    token_iter
                        .next()
                        .ok_or(Error::MissingAxiomLabelError)?
                        .to_string(),
                );

                if token_iter.next().is_some() {
                    return Err(Error::TooManyAxiomLabelTokensError);
                }
            }
            "$d" => {
                let (count, mut distinct_var) =
                    token_iter.fold((0, String::new()), |(i, mut s), t| {
                        s.push_str(t);
                        s.push(' ');
                        (i + 1, s)
                    });
                distinct_var.pop();
                if count < 2 {
                    return Err(Error::ZeroOrOneSymbolDisjError);
                }
                distinct_vars.push(distinct_var);
            }
            "$allowdiscouraged" => {
                if allow_discouraged {
                    return Err(Error::MultipleAllowDiscouragedError);
                }

                allow_discouraged = true;
                if token_iter.next().is_some() {
                    return Err(Error::TokensAfterAllowDiscouragedError);
                }
            }
            "$locateafter" => {
                if locate_after.is_some() {
                    return Err(Error::MultipleLocateAfterError);
                }

                locate_after = Some(LocateAfter::LocateAfter(
                    token_iter
                        .next()
                        .ok_or(Error::MissingLocateAfterLabelError)?
                        .to_string(),
                ));
                if token_iter.next().is_some() {
                    return Err(Error::TooManyLocateAfterTokensError);
                }
            }
            "$locateafterconst" => {
                if locate_after.is_some() {
                    return Err(Error::MultipleLocateAfterError);
                }

                locate_after = Some(LocateAfter::LocateAfterConst(
                    token_iter
                        .next()
                        .ok_or(Error::MissingLocateAfterLabelError)?
                        .to_string(),
                ));
                if token_iter.next().is_some() {
                    return Err(Error::TooManyLocateAfterTokensError);
                }
            }
            "$locateaftervar" => {
                if locate_after.is_some() {
                    return Err(Error::MultipleLocateAfterError);
                }

                locate_after = Some(LocateAfter::LocateAfterVar(
                    token_iter
                        .next()
                        .ok_or(Error::MissingLocateAfterLabelError)?
                        .to_string(),
                ));
                if token_iter.next().is_some() {
                    return Err(Error::TooManyLocateAfterTokensError);
                }
            }
            t if t.starts_with("*") => {
                let mut comment = String::new();

                // Dont push the * at the beginning of the first token
                comment.push_str(&t[1..t.len()]);
                if comment.len() > 0 {
                    comment.push(' ');
                }
                for &token in tokens.iter().skip(1) {
                    if token == "\n" {
                        // Note for future me: This code makes it so that any number of empty
                        // lines are treated as just one. Might want to change this in the future
                        comment.pop();
                        comment.push_str(token);
                    } else {
                        comment.push_str(token);
                        comment.push(' ');
                    }
                }
                while comment.len() > 0
                    && comment.as_bytes()[comment.len() - 1].is_ascii_whitespace()
                {
                    comment.pop();
                }
                comments.push(Comment { text: comment });
            }
            t if !t.starts_with("$") => {
                let mut expression: String = token_iter.fold(String::new(), |mut s, t| {
                    s.push_str(t);
                    s.push(' ');
                    s
                });
                expression.pop();
                if expression.len() == 0 {
                    return Err(Error::MissingMmj2StepExpressionError);
                }
                mmj2_steps.push((t.to_string(), expression));
            }
            _ => return Err(Error::InvalidDollarTokenError),
        }
    }
    // println!("Theorem Label: {:?}\n", theorem_label);
    // println!("Distinct Vars: {:?}\n", distinct_vars);
    // println!("mmj2 Steps: {:?}\n", mmj2_steps);
    // println!("Allow Discouraged: {:?}\n", allow_discouraged);
    // println!("Locate After: {:?}\n", locate_after);
    // println!("Comments: {:?}\n", comments);

    Ok(MmpStructuredInfo {
        constants,
        variables,
        floating_hypotheses,
        theorem_label,
        axiom_label,
        header,
        distinct_vars,
        mmj2_steps,
        allow_discouraged,
        locate_after,
        comments,
    })
}

fn text_to_statements(text: &str) -> Result<Vec<Vec<&str>>, Error> {
    let mut statements_vec: Vec<Vec<&str>> = Vec::new();

    let mut line_iter = text.lines().peekable();

    while let Some(line) = line_iter.next() {
        if line
            .chars()
            .next()
            .is_some_and(|c| !c.is_ascii_whitespace())
        {
            // if the line starts with a non-whitespace token
            statements_vec.push(line.split_ascii_whitespace().collect());
        } else if line.split_ascii_whitespace().next().is_some() {
            // if the line starts with whitespace, but has any non-whitespace tokens
            statements_vec
                .last_mut()
                .ok_or(Error::WhitespaceBeforeFirstTokenError)?
                .extend(line.split_ascii_whitespace());
        } else {
            // if the line is empty or only whitespace
            statements_vec.last_mut().map(|s| s.push(&"\n"));
        }
    }

    Ok(statements_vec)
}
