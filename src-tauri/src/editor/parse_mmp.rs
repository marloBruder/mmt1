use crate::{AppState, Error};
use tauri::async_runtime::Mutex;

struct MmpStructuredInfo {
    theorem_label: Option<String>,
    distinct_vars: Vec<String>,
    mmj2_steps: Vec<(String, String)>,
    allow_discouraged: bool,
    locate_after: Option<LocateAfter>,
    comments: Vec<String>,
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

    let app_state = state.lock().await;

    Ok(())
}

fn statements_to_mmp_structured_info(
    statements: Vec<Vec<&str>>,
) -> Result<MmpStructuredInfo, Error> {
    let mut theorem_label: Option<String> = None;
    let mut distinct_vars: Vec<String> = Vec::new();
    let mut mmj2_steps: Vec<(String, String)> = Vec::new();
    let mut allow_discouraged: bool = false;
    let mut locate_after: Option<LocateAfter> = None;
    let mut comments: Vec<String> = Vec::new();

    for tokens in statements {
        // "\n" denote an empty line, which are only relevant for comments
        let mut token_iter = tokens.iter().map(|t| *t).filter(|t| *t != "\n");

        match token_iter.next().ok_or(Error::InternalLogicError)? {
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
            "$d" => {
                let mut distinct_var: String = token_iter.fold(String::new(), |mut s, t| {
                    s.push_str(t);
                    s.push(' ');
                    s
                });
                distinct_var.pop();
                if distinct_var.len() >= 2 {
                    distinct_vars.push(distinct_var);
                } else {
                    return Err(Error::ZeroOrOneSymbolDisjError);
                }
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
                while comment.as_bytes()[comment.len() - 1].is_ascii_whitespace() {
                    comment.pop();
                }
                comments.push(comment);
            }
            t if !t.starts_with("$") => {
                let mut expression: String = token_iter.fold(String::new(), |mut s, t| {
                    s.push_str(t);
                    s.push(' ');
                    s
                });
                expression.pop();
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
        theorem_label,
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
