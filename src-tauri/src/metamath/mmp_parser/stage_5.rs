use crate::{
    metamath::mmp_parser::{
        MmpParserStage2Success, MmpParserStage4Success, MmpParserStage5, UnifyLine,
    },
    model::{MetamathData, ParseTree},
    Error,
};

pub fn stage_5(
    stage_2: &MmpParserStage2Success,
    stage_4: &MmpParserStage4Success,
    mm_data: &MetamathData,
) -> Result<MmpParserStage5, Error> {
    let mut unify_result: Vec<UnifyLine> = Vec::new();

    for (proof_line, proof_line_parsed) in stage_2
        .proof_lines
        .iter()
        .zip(stage_4.proof_lines_parsed.iter())
    {
        let mut step_ref: Option<String> = None;

        if proof_line.step_ref == "" {
            let proof_line_parse_trees_res = proof_line_parsed
                .hypotheses_parsed
                .iter()
                .map(|hyp| match hyp {
                    Some(index) => Ok(stage_4
                        .proof_lines_parsed
                        .get(*index)
                        .ok_or(Some(Error::InternalLogicError))?
                        .parse_tree
                        .as_ref()
                        .ok_or(None)?),
                    None => Err(None),
                })
                .collect::<Result<Vec<&ParseTree>, Option<Error>>>();

            match proof_line_parse_trees_res {
                // If one of the hyps was "?", do nothing
                Err(None) => {}
                // Return potential InternalLogicError
                Err(Some(err)) => return Err(err),
                Ok(mut proof_line_parse_trees) => {
                    if let Some(ref parse_tree) = proof_line_parsed.parse_tree {
                        proof_line_parse_trees.push(parse_tree);

                        for theorem in mm_data.database_header.theorem_iter() {
                            if let Some(theorem_data) =
                                mm_data.optimized_data.theorem_data.get(&theorem.label)
                            {
                                let parse_trees = theorem_data
                                    .parse_trees
                                    .as_ref()
                                    .ok_or(Error::InternalLogicError)?;

                                let mut theorem_parse_trees: Vec<&ParseTree> =
                                    parse_trees.hypotheses_parsed.iter().collect();
                                theorem_parse_trees.push(&parse_trees.assertion_parsed);

                                if ParseTree::are_substitutions(
                                    &theorem_parse_trees,
                                    &proof_line_parse_trees,
                                    &theorem_data.distinct_variable_pairs,
                                    &stage_4.distinct_variable_pairs,
                                    &mm_data.optimized_data.grammar,
                                    &mm_data.optimized_data.symbol_number_mapping,
                                )? {
                                    step_ref = Some(theorem.label.clone());
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }

        unify_result.push(UnifyLine {
            new_line: false,
            step_name: None,
            hypotheses: None,
            step_ref,
            expression: None,
        });
    }

    Ok(MmpParserStage5 { unify_result })
}
