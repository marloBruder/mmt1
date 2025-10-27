use crate::{
    metamath::verify::{Show, StepResult, Verifier, VerifierCreationResult},
    model::{MetamathData, ProofLine, TheoremPageData},
    util::last_curr_next_iterator::IntoLastCurrNextIterator,
    Error,
};

pub mod export;
pub mod mm_parser;
pub mod mmp_parser;
pub mod verify;

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

pub fn calc_theorem_page_data(
    label: &str,
    metamath_data: &MetamathData,
    show_all: bool,
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

    let optimized_theorem_data = metamath_data
        .optimized_data
        .theorem_data
        .get(label)
        .ok_or(Error::InternalLogicError)?;

    let axiom_dependencies = metamath_data
        .database_header
        .theorem_i_vec_to_theorem_label_vec(&optimized_theorem_data.axiom_dependencies)
        .map_err(|_| Error::InternalLogicError)?;

    let definition_dependencies = metamath_data
        .database_header
        .theorem_i_vec_to_theorem_label_vec(&optimized_theorem_data.definition_dependencies)
        .map_err(|_| Error::InternalLogicError)?;

    let references = metamath_data
        .database_header
        .theorem_i_vec_to_theorem_label_vec(&optimized_theorem_data.references)
        .map_err(|_| Error::InternalLogicError)?;

    let description_parsed = optimized_theorem_data.description_parsed.clone();

    let mut verifier = match Verifier::new(
        theorem,
        metamath_data,
        if show_all { Show::All } else { Show::Logical },
        None,
        None,
        None,
        None,
    )? {
        VerifierCreationResult::Verifier(v) => v,
        res @ (VerifierCreationResult::IsAxiom | VerifierCreationResult::IsIncomplete) => {
            return Ok(TheoremPageData {
                theorem: theorem.clone(),
                theorem_number,
                proof_lines: Vec::new(),
                preview_errors: None,
                preview_deleted_markers: None,
                preview_confirmations: None,
                preview_confirmations_recursive: None,
                preview_unify_markers: None,
                last_theorem_label,
                next_theorem_label,
                axiom_dependencies,
                definition_dependencies,
                references,
                description_parsed,
                invalid_html: false,
                proof_incomplete: matches!(res, VerifierCreationResult::IsIncomplete),
                theorem_type: optimized_theorem_data.theorem_type,
            })
        }
    };

    let mut proof_lines = Vec::new();

    loop {
        let step_result = verifier.proccess_next_step(metamath_data)?;
        match step_result {
            StepResult::VerifierFinished => break,
            StepResult::NoProofLine => {}
            StepResult::ProofLine(proof_line) => {
                proof_lines.push(proof_line);
            }
        }
    }

    // for pl in &proof_lines {
    //     println!("{:#?}", pl);
    // }

    calc_indention(&mut proof_lines)?;

    Ok(TheoremPageData {
        theorem: theorem.clone(),
        theorem_number,
        proof_lines,
        preview_errors: None,
        preview_deleted_markers: None,
        preview_confirmations: None,
        preview_confirmations_recursive: None,
        preview_unify_markers: None,
        last_theorem_label,
        next_theorem_label,
        axiom_dependencies,
        definition_dependencies,
        references,
        description_parsed,
        invalid_html: false,
        proof_incomplete: false,
        theorem_type: optimized_theorem_data.theorem_type,
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
