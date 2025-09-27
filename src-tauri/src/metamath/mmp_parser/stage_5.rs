use std::collections::{HashMap, HashSet};

use crate::{
    metamath::mmp_parser::{
        LocateAfterRef, MmpParserStage2Success, MmpParserStage4Success, MmpParserStage5, ProofLine,
        UnifyLine,
    },
    model::{MetamathData, ParseTree, ParseTreeNode},
    util::{
        earley_parser_optimized::WorkVariable, work_variable_manager::WorkVariableManager,
        StrIterToDelimiterSeperatedString,
    },
    Error,
};

#[derive(Debug)]
struct InProgressUnifyLine {
    pub new_line: bool,
    pub is_hypothesis: bool,
    pub step_name: String,
    pub hypotheses: Vec<String>,
    pub step_ref: String,
    pub parse_tree: Option<ParseTree>,
}

pub fn stage_5(
    stage_2: &MmpParserStage2Success,
    stage_4: &MmpParserStage4Success,
    mm_data: &MetamathData,
) -> Result<MmpParserStage5, Error> {
    let mut work_variable_manager = WorkVariableManager::new(
        &stage_4
            .proof_lines_parsed
            .iter()
            .filter_map(|pt| pt.parse_tree.as_ref())
            .collect(),
        &mm_data.optimized_data.symbol_number_mapping,
    )?;

    let mut step_name_manager = StepNameManager::new(&stage_2.proof_lines);

    let mut in_progress_unify_lines: Vec<InProgressUnifyLine> = stage_2
        .proof_lines
        .iter()
        .zip(stage_4.proof_lines_parsed.iter())
        .map(|(pl, pl_p)| InProgressUnifyLine {
            new_line: false,
            is_hypothesis: pl.is_hypothesis,
            step_name: pl.step_name.to_string(),
            hypotheses: if !pl.hypotheses.is_empty() {
                pl.hypotheses.split(',').map(|s| s.to_string()).collect()
            } else {
                Vec::new()
            },
            step_ref: pl.step_ref.to_string(),
            parse_tree: pl_p.parse_tree.clone(),
        })
        .collect();

    let mut changes = true;
    let mut iters = 1;

    while changes {
        println!("Iter: {}", iters);

        changes = false;

        in_progress_unify_lines.reverse();
        let mut new_in_progress_unify_lines: Vec<InProgressUnifyLine> = Vec::new();

        while let Some(mut unify_line) = in_progress_unify_lines.pop() {
            if unify_line.step_ref != "" && !unify_line.is_hypothesis {
                match unify_step_with_reference(
                    &unify_line,
                    &new_in_progress_unify_lines,
                    mm_data,
                    &mut work_variable_manager,
                    &mut step_name_manager,
                ) {
                    Ok((substitutions, hypotheses, new_lines, new_parse_trees)) => {
                        let work_variables_to_substitute: HashSet<WorkVariable> =
                            substitutions.keys().map(|wv| *wv).collect();

                        for ul in in_progress_unify_lines
                            .iter_mut()
                            .chain(new_in_progress_unify_lines.iter_mut())
                            .chain(vec![&mut unify_line].into_iter())
                        {
                            if let Some(pt) = new_parse_trees.get(&ul.step_name) {
                                changes = true;
                                ul.parse_tree = Some(ParseTree {
                                    typecode: pt.typecode,
                                    top_node: pt
                                        .top_node
                                        .clone_and_apply_substitutions(&substitutions),
                                })
                            } else {
                                if let Some(ref pt) = ul.parse_tree {
                                    if pt
                                        .top_node
                                        .any_work_variable_occurs_in(&work_variables_to_substitute)
                                    {
                                        changes = true;
                                        ul.parse_tree = Some(ParseTree {
                                            typecode: pt.typecode,
                                            top_node: pt
                                                .top_node
                                                .clone_and_apply_substitutions(&substitutions),
                                        });
                                    }
                                }
                            }
                        }

                        if unify_line.hypotheses != hypotheses {
                            changes = true;
                            unify_line.hypotheses = hypotheses;
                        }

                        if !new_lines.is_empty() {
                            changes = true;
                            new_in_progress_unify_lines.extend(new_lines.into_iter());
                        }
                    }
                    Err(Error::UnificationError) => {}
                    Err(error) => return Err(error),
                }
            }

            new_in_progress_unify_lines.push(unify_line);
        }

        in_progress_unify_lines = new_in_progress_unify_lines;
        new_in_progress_unify_lines = Vec::new();

        for mut unify_line in in_progress_unify_lines {
            if unify_line.step_ref == "" {
                if let Some(new_step_ref) = derive_step_ref(
                    &unify_line,
                    &new_in_progress_unify_lines,
                    mm_data,
                    stage_2.locate_after,
                    &stage_4.distinct_variable_pairs,
                )? {
                    changes = true;
                    unify_line.step_ref = new_step_ref;
                }
            }

            new_in_progress_unify_lines.push(unify_line);
        }

        in_progress_unify_lines = new_in_progress_unify_lines;

        iters += 1;
        if iters > 5 {
            break;
        }
    }

    let unify_result = in_progress_unify_lines
        .into_iter()
        .map(|ip_ul| {
            Ok(UnifyLine {
                new_line: ip_ul.new_line,
                step_name: Some(ip_ul.step_name),
                hypotheses: Some(
                    ip_ul
                        .hypotheses
                        .into_iter()
                        .fold_to_delimiter_seperated_string(","),
                ),
                step_ref: Some(ip_ul.step_ref),
                expression: ip_ul
                    .parse_tree
                    .map(|pt| {
                        pt.to_expression(
                            &mm_data.optimized_data.symbol_number_mapping,
                            &mm_data.optimized_data.grammar,
                        )
                    })
                    .transpose()?,
            })
        })
        .collect::<Result<Vec<UnifyLine>, Error>>()?;

    Ok(MmpParserStage5 { unify_result })
}

fn unify_step_with_reference(
    unify_line: &InProgressUnifyLine,
    previous_unify_lines: &Vec<InProgressUnifyLine>,
    mm_data: &MetamathData,
    work_variable_manager: &mut WorkVariableManager,
    step_name_manager: &mut StepNameManager,
) -> Result<
    (
        HashMap<WorkVariable, ParseTreeNode>,
        Vec<String>,
        Vec<InProgressUnifyLine>,
        HashMap<String, ParseTree>,
    ),
    Error,
> {
    let theorem_data = mm_data
        .optimized_data
        .theorem_data
        .get(&unify_line.step_ref)
        .ok_or(Error::InternalLogicError)?;

    let theorem_parse_trees = theorem_data
        .parse_trees
        .as_ref()
        .ok_or(Error::InternalLogicError)?
        .to_cloned_parse_tree_vec_replace_floating_hypotheses(
            &mm_data.optimized_data.symbol_number_mapping,
            &mm_data.optimized_data.grammar,
            work_variable_manager,
        )?;

    let mut proof_line_option_parse_trees: Vec<(Option<&String>, Option<&ParseTree>)> = unify_line
        .hypotheses
        .iter()
        .map(|hyp| {
            Ok(if hyp != "?" {
                (
                    Some(hyp),
                    previous_unify_lines
                        .iter()
                        .find(|ul| &ul.step_name == hyp)
                        .ok_or(Error::InternalLogicError)?
                        .parse_tree
                        .as_ref(),
                )
            } else {
                (None, None)
            })
        })
        .collect::<Result<Vec<(Option<&String>, Option<&ParseTree>)>, Error>>()?;

    while proof_line_option_parse_trees.len() < theorem_parse_trees.len() - 1 {
        proof_line_option_parse_trees.push((None, None));
    }

    proof_line_option_parse_trees
        .push((Some(&unify_line.step_name), unify_line.parse_tree.as_ref()));

    let mut new_lines: Vec<InProgressUnifyLine> = Vec::new();
    let mut new_parse_trees: HashMap<String, ParseTree> = HashMap::new();

    let proof_line_parse_trees: Vec<ParseTree> = proof_line_option_parse_trees
        .iter()
        .zip(theorem_parse_trees.iter())
        .map(|((step_name, o_pt), t_pt)| {
            Ok(match o_pt {
                Some(pt) => (*pt).clone(),
                None => {
                    let pt = ParseTree {
                        typecode: t_pt.typecode,
                        top_node: ParseTreeNode::WorkVariable(
                            work_variable_manager
                                .next_var(
                                    mm_data
                                        .syntax_typecode_of_logical_typecode(t_pt.typecode)
                                        .ok_or(Error::InternalLogicError)?,
                                )
                                .ok_or(Error::InternalLogicError)?,
                        ),
                    };

                    match step_name {
                        Some(name) => {
                            new_parse_trees.insert((*name).clone(), pt.clone());
                        }
                        None => {
                            new_lines.push(InProgressUnifyLine {
                                new_line: true,
                                is_hypothesis: false,
                                step_name: step_name_manager.next_step_name(),
                                hypotheses: Vec::new(),
                                step_ref: String::new(),
                                parse_tree: Some(pt.clone()),
                            });
                        }
                    }

                    pt
                }
            })
        })
        .collect::<Result<Vec<ParseTree>, Error>>()?;

    let substitutions =
        martelli_montanari_unification(theorem_parse_trees, proof_line_parse_trees)?;

    let mut new_line_iter = new_lines.iter();

    let mut hypotheses = unify_line
        .hypotheses
        .iter()
        .map(|hyp_name| {
            Ok(if hyp_name != "?" {
                hyp_name.clone()
            } else {
                new_line_iter
                    .next()
                    .ok_or(Error::InternalLogicError)?
                    .step_name
                    .clone()
            })
        })
        .collect::<Result<Vec<String>, Error>>()?;

    for new_line in new_line_iter {
        hypotheses.push(new_line.step_name.clone());
    }

    Ok((substitutions, hypotheses, new_lines, new_parse_trees))
}

fn martelli_montanari_unification(
    mut theorem_parse_trees: Vec<ParseTree>,
    mut proof_line_parse_trees: Vec<ParseTree>,
) -> Result<HashMap<WorkVariable, ParseTreeNode>, Error> {
    let mut equations: Vec<(&mut ParseTreeNode, &mut ParseTreeNode)> = Vec::new();

    let mut solution_set: Vec<(WorkVariable, ParseTreeNode)> = Vec::new();

    // println!("Theorem parse trees:");
    // println!("{:#?}", theorem_parse_trees);
    // println!("Proof line parse trees:");
    // println!("{:#?}", proof_line_parse_trees);

    for (theorem_parse_tree, proof_line_parse_tree) in theorem_parse_trees
        .iter_mut()
        .zip(proof_line_parse_trees.iter_mut())
    {
        if theorem_parse_tree.typecode != proof_line_parse_tree.typecode {
            return Err(Error::UnificationError);
        }

        equations.push((
            &mut theorem_parse_tree.top_node,
            &mut proof_line_parse_tree.top_node,
        ));
    }

    while let Some((theorem_node, proof_line_node)) = equations.pop() {
        match (theorem_node, proof_line_node) {
            (
                ParseTreeNode::Node {
                    rule_i: theorem_rule_i,
                    sub_nodes: theorem_sub_nodes,
                },
                ParseTreeNode::Node {
                    rule_i: proof_line_rule_i,
                    sub_nodes: proof_line_sub_nodes,
                },
            ) => {
                if theorem_rule_i != proof_line_rule_i {
                    return Err(Error::UnificationError);
                }

                equations.extend(
                    theorem_sub_nodes
                        .iter_mut()
                        .zip(proof_line_sub_nodes.iter_mut()),
                );
            }
            (
                ParseTreeNode::Node { rule_i, sub_nodes },
                ParseTreeNode::WorkVariable(work_variable),
            )
            | (
                ParseTreeNode::WorkVariable(work_variable),
                ParseTreeNode::Node { rule_i, sub_nodes },
            ) => {
                if sub_nodes
                    .iter()
                    .all(|sub_node| !sub_node.work_variable_occurs_in(*work_variable))
                {
                    let parse_tree = ParseTreeNode::Node {
                        rule_i: *rule_i,
                        sub_nodes: sub_nodes.clone(),
                    };

                    for (left_parse_tree, right_parse_tree) in &mut equations {
                        **left_parse_tree = left_parse_tree
                            .clone_and_replace_work_variable(*work_variable, &parse_tree);
                        **right_parse_tree = right_parse_tree
                            .clone_and_replace_work_variable(*work_variable, &parse_tree);
                    }

                    solution_set.push((*work_variable, parse_tree));
                } else {
                    return Err(Error::UnificationError);
                }
            }
            (
                ParseTreeNode::WorkVariable(theorem_work_variable),
                ParseTreeNode::WorkVariable(proof_line_work_variable),
            ) => {
                if *theorem_work_variable != *proof_line_work_variable {
                    let parse_tree = ParseTreeNode::WorkVariable(*proof_line_work_variable);

                    for (left_parse_tree, right_parse_tree) in &mut equations {
                        **left_parse_tree = left_parse_tree
                            .clone_and_replace_work_variable(*theorem_work_variable, &parse_tree);
                        **right_parse_tree = right_parse_tree
                            .clone_and_replace_work_variable(*theorem_work_variable, &parse_tree);
                    }

                    solution_set.push((*theorem_work_variable, parse_tree));
                }
            }
        }
    }

    // println!("Solution set:");
    // println!("{:#?}", solution_set);

    let mut substitutions: HashMap<WorkVariable, ParseTreeNode> = HashMap::new();

    while !solution_set.is_empty() {
        let left_side_variables: HashSet<WorkVariable> =
            solution_set.iter().map(|(var, _)| *var).collect();

        let i = solution_set
            .iter()
            .position(|(_, pt)| !pt.any_work_variable_occurs_in(&left_side_variables))
            .ok_or(Error::InternalLogicError)?;

        let (work_var, parse_tree) = solution_set.remove(i);

        for (_, pt) in &mut solution_set {
            *pt = pt.clone_and_replace_work_variable(work_var, &parse_tree);
        }

        substitutions.insert(work_var, parse_tree);
    }

    Ok(substitutions)
}

fn derive_step_ref(
    unify_line: &InProgressUnifyLine,
    previous_unify_lines: &Vec<InProgressUnifyLine>,
    mm_data: &MetamathData,
    locate_after: Option<LocateAfterRef>,
    distinct_variable_pairs: &HashSet<(String, String)>,
) -> Result<Option<String>, Error> {
    let proof_line_parse_trees_res = unify_line
        .hypotheses
        .iter()
        .map(|hyp| match hyp.as_str() {
            "?" => Err(None),
            hyp_name => Ok(previous_unify_lines
                .iter()
                .find(|ul| &ul.step_name == hyp_name)
                .ok_or(Some(Error::InternalLogicError))?
                .parse_tree
                .as_ref()
                .ok_or(None)?),
        })
        .collect::<Result<Vec<&ParseTree>, Option<Error>>>();

    match proof_line_parse_trees_res {
        // If one of the hyps was "?", do nothing
        Err(None) => {}
        // Return potential InternalLogicError
        Err(Some(err)) => return Err(err),
        Ok(mut proof_line_parse_trees) => {
            if let Some(ref parse_tree) = unify_line.parse_tree {
                proof_line_parse_trees.push(parse_tree);

                for theorem in mm_data
                    .database_header
                    .theorem_locate_after_iter(locate_after)
                {
                    let theorem_data = mm_data
                        .optimized_data
                        .theorem_data
                        .get(&theorem.label)
                        .ok_or(Error::InternalLogicError)?;

                    if let Some(parse_trees) = theorem_data.parse_trees.as_ref() {
                        let mut theorem_parse_trees: Vec<&ParseTree> =
                            parse_trees.hypotheses_parsed.iter().collect();
                        theorem_parse_trees.push(&parse_trees.assertion_parsed);

                        if ParseTree::are_substitutions(
                            &theorem_parse_trees,
                            &proof_line_parse_trees,
                            &theorem_data.distinct_variable_pairs,
                            distinct_variable_pairs,
                            &mm_data.optimized_data.grammar,
                            &mm_data.optimized_data.symbol_number_mapping,
                        )? {
                            return Ok(Some(theorem.label.clone()));
                        }
                    }
                }
            }
        }
    }

    Ok(None)
}

struct StepNameManager {
    next_step_name_num: u32,
}

impl StepNameManager {
    fn new(proof_lines: &Vec<ProofLine>) -> StepNameManager {
        let mut next_step_name_num = 1;

        for proof_line in proof_lines {
            if proof_line.step_name.starts_with("d") {
                if let Some((_, step_num_str)) = proof_line.step_name.split_at_checked(1) {
                    if let Ok(step_num) = step_num_str.parse::<u32>() {
                        if step_num >= next_step_name_num {
                            next_step_name_num = step_num + 1;
                        }
                    }
                }
            }
        }

        StepNameManager { next_step_name_num }
    }

    fn next_step_name(&mut self) -> String {
        self.next_step_name_num += 1;
        format!("d{}", self.next_step_name_num - 1)
    }
}
