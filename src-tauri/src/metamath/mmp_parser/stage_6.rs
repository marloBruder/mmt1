use std::collections::{HashMap, HashSet};

use crate::{
    metamath::mmp_parser::{MmpParserStage4Success, MmpParserStage5, MmpParserStage6, UnifyLine},
    model::{MetamathData, ParseTree, ParseTreeNode},
    Error,
};

pub fn stage_6(
    stage_4: &MmpParserStage4Success,
    stage_5: &MmpParserStage5,
    mm_data: &MetamathData,
) -> Result<MmpParserStage6, Error> {
    let Some(uncompressed_proof) = calc_uncompressed_proof(
        &stage_5.unify_result,
        &stage_4.distinct_variable_pairs,
        mm_data,
    )?
    else {
        return Ok(MmpParserStage6 { proof: None });
    };

    let compressed_proof =
        calc_compressed_proof(&stage_5.unify_result, uncompressed_proof, mm_data);

    Ok(MmpParserStage6 {
        proof: Some(compressed_proof),
    })
}

fn calc_uncompressed_proof(
    unify_result: &Vec<UnifyLine>,
    distinct_variable_pairs: &HashSet<(String, String)>,
    mm_data: &MetamathData,
) -> Result<Option<String>, Error> {
    let Some(qed_step_i) = unify_result.iter().position(|ul| ul.step_name == "qed") else {
        return Ok(None);
    };

    let mut proofs: Vec<Option<String>> = Vec::new();

    for (i, unify_line) in unify_result.iter().enumerate() {
        if unify_line.is_hypothesis {
            proofs.push(Some(unify_line.step_ref.to_string()));
        } else {
            if unify_line.step_ref == "" || unify_line.deleted_line {
                proofs.push(None);
                continue;
            }

            let Some(unify_line_hypotheses_parsed) = unify_line
                .hypotheses
                .iter()
                .map(|hyp| {
                    if hyp == "?" {
                        None
                    } else {
                        unify_result
                            .iter()
                            .take(i)
                            .position(|ul| ul.step_name == *hyp)
                    }
                })
                .collect::<Option<Vec<usize>>>()
            else {
                proofs.push(None);
                continue;
            };

            let Some(unify_line_parse_trees) = unify_line_hypotheses_parsed
                .iter()
                .map(|hyp_i| {
                    Ok(unify_result
                        .get(*hyp_i)
                        .ok_or(Error::InternalLogicError)?
                        .parse_tree
                        .as_ref())
                })
                .chain(vec![Ok(unify_line.parse_tree.as_ref())])
                .collect::<Result<Option<Vec<&ParseTree>>, Error>>()?
            else {
                proofs.push(None);
                continue;
            };

            let theorem_data = mm_data
                .optimized_data
                .theorem_data
                .get(&unify_line.step_ref)
                .ok_or(Error::InternalLogicError)?;

            let theorem_parse_trees = theorem_data
                .parse_trees
                .as_ref()
                .ok_or(Error::InternalLogicError)?
                .to_ref_parse_tree_vec();

            let Some(substitutions) = ParseTree::calc_substitutions(
                &theorem_parse_trees,
                &unify_line_parse_trees,
                &theorem_data.distinct_variable_pairs,
                distinct_variable_pairs,
                &mm_data.optimized_data.grammar,
                &mm_data.optimized_data.symbol_number_mapping,
            )?
            else {
                proofs.push(None);
                continue;
            };

            let mut var_proofs = substitutions
                .into_iter()
                .map(|(rule_i, pt_node)| {
                    let rule = mm_data
                        .optimized_data
                        .grammar
                        .rules
                        .get(rule_i as usize)
                        .ok_or(Error::InternalLogicError)?;

                    let var_symbol = rule.right_side.first().ok_or(Error::InternalLogicError)?;

                    let var_str = mm_data
                        .optimized_data
                        .symbol_number_mapping
                        .symbols
                        .get(&var_symbol.symbol_i)
                        .ok_or(Error::InternalLogicError)?
                        .as_str();

                    Ok((
                        var_str,
                        pt_node.calc_proof(&mm_data.optimized_data.grammar)?,
                    ))
                })
                .collect::<Result<HashMap<&str, String>, Error>>()?;

            let vars_proof = mm_data
                .optimized_data
                .floating_hypotheses
                .iter()
                .filter_map(|fh| var_proofs.remove(&*fh.variable))
                .fold(String::new(), |mut s, vp| {
                    s.push_str(&vp);
                    s.push(' ');
                    s
                });

            let Some(mut proof) = unify_line_hypotheses_parsed
                .into_iter()
                .filter_map(|hyp_i| proofs.get(hyp_i))
                .fold(Some(vars_proof), |s, p| {
                    let Some(mut s) = s else { return None };
                    let Some(p) = p else { return None };
                    s.push_str(p);
                    s.push(' ');
                    Some(s)
                })
            else {
                proofs.push(None);
                continue;
            };

            proof.push_str(&unify_line.step_ref);

            proofs.push(Some(proof));

            if unify_line.step_name == "qed" {
                break;
            }
        }

        // println!("{}", proofs.last().unwrap());
    }

    Ok(proofs.swap_remove(qed_step_i))
}

fn calc_compressed_proof(
    unify_result: &Vec<UnifyLine>,
    uncompressed_proof: String,
    mm_data: &MetamathData,
) -> String {
    let vars_in_theorem: HashSet<&str> = unify_result
        .iter()
        .filter(|ul| ul.is_hypothesis || ul.step_name == "qed")
        // Should never filter
        .filter_map(|ul| Some(ul.parse_tree.as_ref()?.top_node.iter()))
        .flatten()
        .filter_map(|ptn| {
            if let ParseTreeNode::Node { rule_i, .. } = ptn {
                let rule = mm_data.optimized_data.grammar.rules.get(*rule_i as usize)?;

                if rule.is_floating_hypothesis {
                    let symbol_i = rule.right_side.first()?.symbol_i;

                    Some(
                        mm_data
                            .optimized_data
                            .symbol_number_mapping
                            .symbols
                            .get(&symbol_i)?
                            .as_str(),
                    )
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    let floating_hypotheses: Vec<&str> = mm_data
        .optimized_data
        .floating_hypotheses
        .iter()
        .filter(|fh| vars_in_theorem.contains(&*fh.variable))
        .map(|fh| &*fh.label)
        .collect();

    let hypotheses: Vec<&str> = unify_result
        .iter()
        .filter(|ul| ul.is_hypothesis)
        .map(|ul| &*ul.step_ref)
        .collect();

    let mut previously_used_steps: Vec<&str> = Vec::new();

    let mut compressed_steps = String::new();

    for step in uncompressed_proof.split_ascii_whitespace() {
        let number = match floating_hypotheses
            .iter()
            .chain(hypotheses.iter())
            .chain(previously_used_steps.iter())
            .position(|prev_step| *prev_step == step)
        {
            Some(i) => i,
            None => {
                previously_used_steps.push(step);
                floating_hypotheses.len() + hypotheses.len() + previously_used_steps.len() - 1
            }
        };

        let compressed_number = number_to_compressed_proof_format_number(number);

        compressed_steps.push_str(&compressed_number);
    }

    let mut result = previously_used_steps
        .iter()
        .fold("( ".to_string(), |mut s, pus| {
            s.push_str(pus);
            s.push(' ');
            s
        });

    result.push_str(") ");
    result.push_str(&compressed_steps);

    result
}

fn number_to_compressed_proof_format_number(mut number: usize) -> String {
    let mut compressed_number = String::new();

    compressed_number.push(('A' as u8 + (number % 20) as u8) as char);

    number = number / 20;

    while number != 0 {
        compressed_number.push(('U' as u8 + ((number - 1) % 5) as u8) as char);
        number = number / 6;
    }

    compressed_number.chars().rev().collect()
}
