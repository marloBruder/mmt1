use std::collections::{HashMap, HashSet};

use crate::{
    metamath::mmp_parser::{
        LocateAfterRef, MmpParserStage2Success, MmpParserStage3Theorem, MmpParserStage4Success,
        MmpParserStage5, ProofLine, ProofLineStatus, UnifyLine,
    },
    model::{MetamathData, ParseTree, ParseTreeNode},
    util::{
        earley_parser_optimized::{Grammar, WorkVariable},
        work_variable_manager::WorkVariableManager,
    },
    Error,
};

pub fn stage_5(
    stage_2: &MmpParserStage2Success,
    stage_3: &MmpParserStage3Theorem,
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

    let mut hypotheses_name_manager =
        HypothesesNameManager::new(stage_3.label, &stage_2.proof_lines);

    let mut unify_lines: Vec<UnifyLine> = stage_2
        .proof_lines
        .iter()
        .zip(stage_4.proof_lines_parsed.iter())
        .zip(stage_4.proof_line_statuses.iter())
        .rev()
        .map(|((pl, pl_p), pl_s)| UnifyLine {
            new_line: false,
            deleted_line: false,
            advanced_unification: pl.advanced_unification,
            is_hypothesis: pl.is_hypothesis,
            step_name: pl.step_name.to_string(),
            hypotheses: if !pl.hypotheses.is_empty() {
                pl.hypotheses.split(',').map(|s| s.to_string()).collect()
            } else {
                Vec::new()
            },
            step_ref: pl.step_ref.to_string(),
            parse_tree: pl_p.parse_tree.clone(),
            status: *pl_s,
        })
        .collect();

    let mut new_unify_lines: Vec<UnifyLine> = Vec::new();

    while let Some(mut unify_line) = unify_lines.pop() {
        if unify_line.step_ref != "" && !unify_line.is_hypothesis {
            match unify_step_with_reference(
                &unify_line,
                &new_unify_lines,
                mm_data,
                &mut work_variable_manager,
                &mut step_name_manager,
            ) {
                Ok((substitutions, hypotheses, new_lines, new_parse_trees)) => {
                    let work_variables_to_substitute: HashSet<WorkVariable> =
                        substitutions.keys().map(|wv| *wv).collect();

                    for ul in unify_lines
                        .iter_mut()
                        .chain(new_unify_lines.iter_mut())
                        .chain(vec![&mut unify_line].into_iter())
                    {
                        if let Some(pt) = new_parse_trees.get(&ul.step_name) {
                            ul.parse_tree = Some(ParseTree {
                                typecode: pt.typecode,
                                top_node: pt.top_node.clone_and_apply_substitutions(&substitutions),
                            });

                            set_unify_status(&mut ul.status, 3)?;
                        } else if let Some(ref pt) = ul.parse_tree {
                            if pt
                                .top_node
                                .any_work_variable_occurs_in(&work_variables_to_substitute)
                            {
                                ul.parse_tree = Some(ParseTree {
                                    typecode: pt.typecode,
                                    top_node: pt
                                        .top_node
                                        .clone_and_apply_substitutions(&substitutions),
                                });

                                set_unify_status(&mut ul.status, 3)?;
                            }
                        }
                    }

                    if unify_line.hypotheses != hypotheses {
                        unify_line.hypotheses = hypotheses;

                        set_unify_status(&mut unify_line.status, 1)?;
                    }

                    new_unify_lines.extend(new_lines.into_iter().map(|mut nl| {
                        nl.parse_tree = nl.parse_tree.map(|pt| ParseTree {
                            typecode: pt.typecode,
                            top_node: pt.top_node.clone_and_apply_substitutions(&substitutions),
                        });
                        nl
                    }));
                }
                Err(Error::UnificationError) => {}
                Err(error) => return Err(error),
            }
        }

        if unify_line.step_name == "" {
            unify_line.step_name = step_name_manager.next_step_name();

            set_unify_status(&mut unify_line.status, 0)?;
        }

        new_unify_lines.push(unify_line);
    }

    unify_lines = new_unify_lines;
    new_unify_lines = Vec::new();

    for mut unify_line in unify_lines {
        if unify_line.step_ref == "" {
            if let Some((new_step_ref, option_new_hypotheses)) = derive_step_ref(
                &unify_line,
                &new_unify_lines,
                mm_data,
                stage_2.locate_after,
                &stage_4.distinct_variable_pairs,
                &mut hypotheses_name_manager,
            )? {
                unify_line.step_ref = new_step_ref;

                set_unify_status(&mut unify_line.status, 2)?;

                if let Some(new_hyps) = option_new_hypotheses {
                    unify_line.hypotheses = new_hyps;

                    set_unify_status(&mut unify_line.status, 1)?;
                }
            }
        }

        new_unify_lines.push(unify_line);
    }

    unify_lines = new_unify_lines;
    new_unify_lines = Vec::new();

    unify_lines.reverse();

    while let Some(mut unify_line) = unify_lines.pop() {
        set_unify_status_recursively_correct(&mut unify_line, &new_unify_lines);

        if let Some(other_unify_line) = new_unify_lines
            .iter()
            .find(|oul| oul.parse_tree == unify_line.parse_tree)
        {
            if !(unify_line_is_recursively_correct(&unify_line)
                && !unify_line_is_recursively_correct(other_unify_line))
            {
                unify_line.deleted_line = true;
                for ul in &mut unify_lines {
                    let mut hypothesis_replaced = false;
                    for hyp in &mut ul.hypotheses {
                        if *hyp == unify_line.step_name {
                            *hyp = other_unify_line.step_name.clone();
                            hypothesis_replaced = true;
                        }
                    }
                    if hypothesis_replaced {
                        set_unify_status(&mut ul.status, 1)?;
                    }
                }
            }
        }

        new_unify_lines.push(unify_line);
    }

    unify_lines = new_unify_lines;

    Ok(MmpParserStage5 {
        unify_result: unify_lines,
    })
}

fn set_unify_status(status: &mut ProofLineStatus, position: u32) -> Result<(), Error> {
    match status {
        ProofLineStatus::Unified(unified_status, _) => match position {
            0 => unified_status.0 = true,
            1 => unified_status.1 = true,
            2 => unified_status.2 = true,
            3 => unified_status.3 = true,
            _ => return Err(Error::InternalLogicError),
        },
        ProofLineStatus::None | ProofLineStatus::Correct | ProofLineStatus::CorrectRecursively => {
            *status = ProofLineStatus::Unified(
                match position {
                    0 => (true, false, false, false),
                    1 => (false, true, false, false),
                    2 => (false, false, true, false),
                    3 => (false, false, false, true),
                    _ => return Err(Error::InternalLogicError),
                },
                false,
            )
        }
        _ => return Err(Error::InternalLogicError),
    }

    Ok(())
}

fn set_unify_status_recursively_correct(
    unify_line: &mut UnifyLine,
    previous_unify_lines: &Vec<UnifyLine>,
) {
    if let ProofLineStatus::Unified(_, recursively_correct) = &mut unify_line.status {
        if unify_line.hypotheses.iter().all(|hyp| {
            previous_unify_lines
                .iter()
                .find(|ul| ul.step_name == *hyp)
                .is_some_and(|pul| unify_line_is_recursively_correct(pul))
        }) {
            *recursively_correct = true;
        }
    }
}

fn unify_step_with_reference(
    unify_line: &UnifyLine,
    previous_unify_lines: &Vec<UnifyLine>,
    mm_data: &MetamathData,
    work_variable_manager: &mut WorkVariableManager,
    step_name_manager: &mut StepNameManager,
) -> Result<
    (
        HashMap<WorkVariable, ParseTreeNode>,
        Vec<String>,
        Vec<UnifyLine>,
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

    let mut new_lines: Vec<UnifyLine> = Vec::new();
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
                            new_lines.push(UnifyLine {
                                new_line: true,
                                deleted_line: false,
                                advanced_unification: true,
                                is_hypothesis: false,
                                step_name: step_name_manager.next_step_name(),
                                hypotheses: Vec::new(),
                                step_ref: String::new(),
                                parse_tree: Some(pt.clone()),
                                status: ProofLineStatus::Unified((true, true, true, true), false),
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
    unify_line: &UnifyLine,
    previous_unify_lines: &Vec<UnifyLine>,
    mm_data: &MetamathData,
    locate_after: Option<LocateAfterRef>,
    distinct_variable_pairs: &HashSet<(String, String)>,
    hypotheses_name_manager: &mut HypothesesNameManager,
) -> Result<Option<(String, Option<Vec<String>>)>, Error> {
    if !unify_line.advanced_unification {
        if !unify_line.is_hypothesis {
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
                // If one of the hyps was "?", nothing to do
                Err(None) => return Ok(None),
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
                                let theorem_parse_trees = parse_trees.to_ref_parse_tree_vec();

                                if ParseTree::are_substitutions(
                                    &theorem_parse_trees,
                                    &proof_line_parse_trees,
                                    &theorem_data.distinct_variable_pairs,
                                    distinct_variable_pairs,
                                    &mm_data.optimized_data.grammar,
                                    &mm_data.optimized_data.symbol_number_mapping,
                                )? {
                                    return Ok(Some((theorem.label.clone(), None)));
                                }
                            }
                        }
                    }
                }
            }
        } else {
            return Ok(Some((hypotheses_name_manager.next_hypothesis_name(), None)));
        }
    } else {
        let proof_line_hypotheses_parse_trees_res = unify_line
            .hypotheses
            .iter()
            .map(|hyp| {
                Ok(match hyp.as_str() {
                    "?" => None,
                    hyp_name => Some(
                        previous_unify_lines
                            .iter()
                            .find(|ul| &ul.step_name == hyp_name)
                            .ok_or(Some(Error::InternalLogicError))?
                            .parse_tree
                            .as_ref()
                            .ok_or(None)?,
                    ),
                })
            })
            .collect::<Result<Vec<Option<&ParseTree>>, Option<Error>>>();

        match proof_line_hypotheses_parse_trees_res {
            // if one of the hypotheses doens't have a parse tree, nothing to do
            Err(None) => return Ok(None),
            // return potentiel InternalLogicError
            Err(Some(err)) => return Err(err),
            Ok(proof_line_hypotheses_parse_trees) => {
                let Some(proof_line_assertion_parse_tree) = unify_line.parse_tree.as_ref() else {
                    return Ok(None);
                };

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
                        let theorem_hypotheses_parse_trees: Vec<&ParseTree> =
                            parse_trees.hypotheses_parsed.iter().collect();

                        let theorem_assertion_parse_tree = &parse_trees.assertion_parsed;

                        if let Some(mut new_hypotheses) = find_hypotheses(
                            previous_unify_lines,
                            &proof_line_hypotheses_parse_trees,
                            proof_line_assertion_parse_tree,
                            &theorem_hypotheses_parse_trees,
                            theorem_assertion_parse_tree,
                            &mm_data.optimized_data.grammar,
                        )? {
                            let mut hypotheses_combined = Vec::new();

                            for hypothesis in &unify_line.hypotheses {
                                match hypothesis.as_str() {
                                    "?" => hypotheses_combined.push(
                                        new_hypotheses.pop().ok_or(Error::InternalLogicError)?,
                                    ),
                                    hyp => {
                                        hypotheses_combined.push(hyp.to_string());
                                    }
                                }
                            }

                            while let Some(hyp) = new_hypotheses.pop() {
                                hypotheses_combined.push(hyp);
                            }

                            return Ok(Some((theorem.label.clone(), Some(hypotheses_combined))));
                        }
                    }
                }
            }
        }
    }

    Ok(None)
}

#[derive(Clone, Copy)]
enum HypothesisCompareStatus {
    PossiblyResolvable,
    Unresolvable,
}

fn find_hypotheses(
    previous_unify_lines: &Vec<UnifyLine>,
    proof_line_hypotheses_parse_trees: &Vec<Option<&ParseTree>>,
    proof_line_assertion_parse_tree: &ParseTree,
    theorem_hypotheses_parse_trees: &Vec<&ParseTree>,
    theorem_assertion_parse_tree: &ParseTree,
    grammar: &Grammar,
) -> Result<Option<Vec<String>>, Error> {
    if theorem_hypotheses_parse_trees.len() < proof_line_hypotheses_parse_trees.len() {
        return Ok(None);
    }

    let mut substitutions: HashMap<u32, &ParseTreeNode> = HashMap::new();

    let mut hypotheses_to_find: Vec<&ParseTree> = Vec::new();

    if !update_substitutions(
        &mut substitutions,
        theorem_assertion_parse_tree,
        proof_line_assertion_parse_tree,
        grammar,
    )? {
        return Ok(None);
    }

    for (parse_tree, opt_other_parse_tree) in theorem_hypotheses_parse_trees
        .iter()
        .zip(proof_line_hypotheses_parse_trees.iter())
    {
        if let Some(other_parse_tree) = opt_other_parse_tree {
            if !update_substitutions(&mut substitutions, parse_tree, other_parse_tree, grammar)? {
                return Ok(None);
            }
        } else {
            hypotheses_to_find.push(parse_tree);
        }
    }

    hypotheses_to_find.extend(
        theorem_hypotheses_parse_trees
            .iter()
            .skip(proof_line_hypotheses_parse_trees.len()),
    );

    let mut hypotheses_found: Vec<usize> = Vec::new();

    let mut compare_statuses: Vec<Vec<HypothesisCompareStatus>> = vec![
        vec![HypothesisCompareStatus::PossiblyResolvable; previous_unify_lines.len()];
        hypotheses_to_find.len()
        ];

    let mut substitutions = vec![substitutions, HashMap::new()];

    if !find_hypothesis_recursive(
        &hypotheses_to_find,
        &mut hypotheses_found,
        &mut compare_statuses,
        &mut substitutions,
        previous_unify_lines,
        grammar,
    )? {
        return Ok(None);
    }

    Ok(Some(
        hypotheses_found
            .into_iter()
            .map(|hyp_i| {
                Ok(previous_unify_lines
                    .get(hyp_i)
                    .ok_or(Error::InternalLogicError)?
                    .step_name
                    .clone())
            })
            .collect::<Result<Vec<String>, Error>>()?,
    ))
}

// tree may not contain work variables
fn update_substitutions<'a>(
    substitutions: &mut HashMap<u32, &'a ParseTreeNode>,
    tree: &ParseTree,
    other_tree: &'a ParseTree,
    grammar: &Grammar,
    // symbol_number_mapping: &SymbolNumberMapping,
) -> Result<bool, Error> {
    if tree.typecode != other_tree.typecode {
        return Ok(false);
    }

    let mut nodes_to_check: Vec<(&ParseTreeNode, &ParseTreeNode)> =
        vec![(&tree.top_node, &other_tree.top_node)];

    while let Some((subtree, other_subtree)) = nodes_to_check.pop() {
        let ParseTreeNode::Node { rule_i, sub_nodes } = subtree else {
            return Err(Error::InternalLogicError);
        };
        let subtree_rule = grammar
            .rules
            .get(*rule_i as usize)
            .ok_or(Error::InternalLogicError)?;

        match other_subtree {
            ParseTreeNode::Node {
                rule_i: other_rule_i,
                sub_nodes: other_sub_nodes,
            } => {
                let other_subtree_rule = grammar
                    .rules
                    .get(*other_rule_i as usize)
                    .ok_or(Error::InternalLogicError)?;

                if subtree_rule.is_floating_hypothesis {
                    match substitutions.get(rule_i) {
                        Some(&sub) => {
                            if sub != other_subtree {
                                return Ok(false);
                            }
                        }
                        None => {
                            if subtree_rule.left_side == other_subtree_rule.left_side {
                                substitutions.insert(*rule_i, other_subtree);
                            } else {
                                return Ok(false);
                            }
                        }
                    }
                } else {
                    if *rule_i != *other_rule_i || sub_nodes.len() != other_sub_nodes.len() {
                        return Ok(false);
                    }
                    for (node, other_node) in sub_nodes.iter().zip(other_sub_nodes.iter()) {
                        nodes_to_check.push((node, other_node));
                    }
                }
            }
            ParseTreeNode::WorkVariable(work_variable) => {
                if subtree_rule.is_floating_hypothesis {
                    match substitutions.get(rule_i) {
                        Some(&sub) => {
                            if sub != other_subtree {
                                return Ok(false);
                            }
                        }
                        None => {
                            if subtree_rule.left_side.symbol_i == work_variable.typecode_i {
                                substitutions.insert(*rule_i, other_subtree);
                            } else {
                                return Ok(false);
                            }
                        }
                    }
                } else {
                    return Ok(false);
                }
            }
        }
    }

    Ok(true)
}

fn find_hypothesis_recursive<'a>(
    hypotheses_to_find: &[&ParseTree],
    hypotheses_found: &mut Vec<usize>,
    compare_statuses: &mut Vec<Vec<HypothesisCompareStatus>>,
    substitutions: &mut Vec<HashMap<u32, &'a ParseTreeNode>>,
    previous_unify_lines: &'a Vec<UnifyLine>,
    grammar: &Grammar,
) -> Result<bool, Error> {
    let Some(&hypothesis_to_find) = hypotheses_to_find.last() else {
        // if there are no more hypotheses to find, the agorithm was successful
        return Ok(true);
    };

    let Some(mut hypothesis_compare_statuses) = compare_statuses.pop() else {
        return Err(Error::InternalLogicError);
    };

    for (i, (compare_status, unify_line)) in hypothesis_compare_statuses
        .iter_mut()
        .zip(previous_unify_lines.iter())
        .enumerate()
    {
        match compare_status {
            HypothesisCompareStatus::Unresolvable => {}
            HypothesisCompareStatus::PossiblyResolvable => match &unify_line.parse_tree {
                None => *compare_status = HypothesisCompareStatus::Unresolvable,
                Some(unify_line_pt) => {
                    match check_hypothesis(
                        hypothesis_to_find,
                        unify_line_pt,
                        substitutions,
                        grammar,
                    )? {
                        None => {
                            hypotheses_found.push(i);
                            substitutions.push(HashMap::new());

                            if find_hypothesis_recursive(
                                &hypotheses_to_find[..hypotheses_to_find.len() - 1],
                                hypotheses_found,
                                compare_statuses,
                                substitutions,
                                previous_unify_lines,
                                grammar,
                            )? {
                                return Ok(true);
                            }

                            substitutions.pop();
                            hypotheses_found.pop();
                        }
                        Some(HypothesisCompareStatus::PossiblyResolvable) => {}
                        Some(HypothesisCompareStatus::Unresolvable) => {
                            *compare_status = HypothesisCompareStatus::Unresolvable;
                        }
                    }
                }
            },
        }
    }

    compare_statuses.push(hypothesis_compare_statuses);

    Ok(false)
}

// hypothesis may not contain any work variables
fn check_hypothesis<'a>(
    hypothesis: &ParseTree,
    potential_match: &'a ParseTree,
    substitutions: &mut Vec<HashMap<u32, &'a ParseTreeNode>>,
    grammar: &Grammar,
) -> Result<Option<HypothesisCompareStatus>, Error> {
    if hypothesis.typecode != potential_match.typecode {
        return Ok(Some(HypothesisCompareStatus::Unresolvable));
    }

    let mut nodes_to_check: Vec<(&ParseTreeNode, &ParseTreeNode)> =
        vec![(&hypothesis.top_node, &potential_match.top_node)];

    while let Some((subtree, other_subtree)) = nodes_to_check.pop() {
        let ParseTreeNode::Node { rule_i, sub_nodes } = subtree else {
            return Err(Error::InternalLogicError);
        };
        let subtree_rule = grammar
            .rules
            .get(*rule_i as usize)
            .ok_or(Error::InternalLogicError)?;

        match other_subtree {
            ParseTreeNode::Node {
                rule_i: other_rule_i,
                sub_nodes: other_sub_nodes,
            } => {
                let other_subtree_rule = grammar
                    .rules
                    .get(*other_rule_i as usize)
                    .ok_or(Error::InternalLogicError)?;

                if subtree_rule.is_floating_hypothesis {
                    match substitutions
                        .iter()
                        .map(|sub_map| sub_map.get(rule_i))
                        .find_map(|res| if let Some(pt) = res { Some(pt) } else { None })
                    {
                        Some(&sub) => {
                            if sub != other_subtree {
                                return Ok(Some(HypothesisCompareStatus::PossiblyResolvable));
                            }
                        }
                        None => {
                            if subtree_rule.left_side == other_subtree_rule.left_side {
                                substitutions
                                    .last_mut()
                                    .ok_or(Error::InternalLogicError)?
                                    .insert(*rule_i, other_subtree);
                            } else {
                                return Ok(Some(HypothesisCompareStatus::PossiblyResolvable));
                            }
                        }
                    }
                } else {
                    if *rule_i != *other_rule_i || sub_nodes.len() != other_sub_nodes.len() {
                        return Ok(Some(HypothesisCompareStatus::Unresolvable));
                    }
                    for (node, other_node) in sub_nodes.iter().zip(other_sub_nodes.iter()) {
                        nodes_to_check.push((node, other_node));
                    }
                }
            }
            ParseTreeNode::WorkVariable(work_variable) => {
                if subtree_rule.is_floating_hypothesis {
                    match substitutions
                        .iter()
                        .map(|sub_map| sub_map.get(rule_i))
                        .find_map(|res| if let Some(pt) = res { Some(pt) } else { None })
                    {
                        Some(&sub) => {
                            if sub != other_subtree {
                                return Ok(Some(HypothesisCompareStatus::PossiblyResolvable));
                            }
                        }
                        None => {
                            if subtree_rule.left_side.symbol_i == work_variable.typecode_i {
                                substitutions
                                    .last_mut()
                                    .ok_or(Error::InternalLogicError)?
                                    .insert(*rule_i, other_subtree);
                            } else {
                                return Ok(Some(HypothesisCompareStatus::PossiblyResolvable));
                            }
                        }
                    }
                } else {
                    return Ok(Some(HypothesisCompareStatus::Unresolvable));
                }
            }
        }
    }

    Ok(None)
}

fn unify_line_is_recursively_correct(unify_line: &UnifyLine) -> bool {
    match unify_line.status {
        ProofLineStatus::None => false,
        ProofLineStatus::Err(_) => false,
        ProofLineStatus::Correct => false,
        ProofLineStatus::CorrectRecursively => true,
        ProofLineStatus::Unified(_, correct_recursively) => correct_recursively,
    }
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

struct HypothesesNameManager {
    theorem_label: String,
    next_step_name_num: u32,
}

impl HypothesesNameManager {
    fn new(theorem_label: &str, proof_lines: &Vec<ProofLine>) -> HypothesesNameManager {
        let mut next_step_name_num = 1;

        for proof_line in proof_lines {
            if proof_line.is_hypothesis {
                if proof_line
                    .step_ref
                    .starts_with(&format!("{}.", theorem_label))
                {
                    if let Some((_, step_num_str)) = proof_line
                        .step_ref
                        .split_at_checked(theorem_label.len() + 1)
                    {
                        if let Ok(step_num) = step_num_str.parse::<u32>() {
                            if step_num >= next_step_name_num {
                                next_step_name_num = step_num + 1;
                            }
                        }
                    }
                }
            }
        }

        HypothesesNameManager {
            theorem_label: theorem_label.to_string(),
            next_step_name_num,
        }
    }

    fn next_hypothesis_name(&mut self) -> String {
        self.next_step_name_num += 1;
        format!("{}.{}", self.theorem_label, self.next_step_name_num - 1)
    }
}
