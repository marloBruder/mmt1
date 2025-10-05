use std::collections::HashMap;

use crate::{
    metamath::mmp_parser::{MmpParserStage4Success, MmpParserStage5, MmpParserStage6},
    model::{MetamathData, ParseTree},
    Error,
};

pub fn stage_6(
    stage_4: &MmpParserStage4Success,
    stage_5: &MmpParserStage5,
    mm_data: &MetamathData,
) -> Result<MmpParserStage6, Error> {
    let Some(qed_step_i) = stage_5
        .unify_result
        .iter()
        .position(|ul| ul.step_name == "qed")
    else {
        return Ok(MmpParserStage6 { proof: None });
    };

    let mut proofs: Vec<Option<String>> = Vec::new();

    for (i, unify_line) in stage_5.unify_result.iter().enumerate() {
        if unify_line.is_hypothesis {
            proofs.push(Some(unify_line.step_ref.to_string()));
        } else {
            if unify_line.step_ref == "" {
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
                        stage_5
                            .unify_result
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
                    Ok(stage_5
                        .unify_result
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
                &stage_4.distinct_variable_pairs,
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
        }

        // println!("{}", proofs.last().unwrap());
    }

    Ok(MmpParserStage6 {
        proof: proofs.swap_remove(qed_step_i),
    })
}
