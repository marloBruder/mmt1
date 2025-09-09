use std::collections::{HashMap, HashSet};

use crate::{
    metamath::mmp_parser::{
        MmpParserStage2Success, MmpParserStage4Success, MmpParserStage5, UnifyLine,
    },
    model::{MetamathData, ParseTree, ParseTreeNode, SymbolNumberMapping},
    util::earley_parser_optimized::{Grammar, WorkVariable},
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

                        for theorem in mm_data
                            .database_header
                            .theorem_locate_after_iter(stage_2.locate_after)
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
        } else {
            if !proof_line.is_hypothesis {
                if let Some(theorem_data) =
                    mm_data.optimized_data.theorem_data.get(proof_line.step_ref)
                {
                    let theorem_parse_trees_struct = theorem_data
                        .parse_trees
                        .as_ref()
                        .ok_or(Error::InternalLogicError)?;

                    let mut theorem_parse_trees: Vec<&ParseTree> = theorem_parse_trees_struct
                        .hypotheses_parsed
                        .iter()
                        .collect();
                    theorem_parse_trees.push(&theorem_parse_trees_struct.assertion_parsed);

                    let mut proof_line_parse_trees: Vec<Option<&ParseTree>> = proof_line_parsed
                        .hypotheses_parsed
                        .iter()
                        .map(|hyp| match hyp {
                            Some(hyp_num) => stage_4
                                .proof_lines_parsed
                                // Should always be Some(_)
                                .get(*hyp_num)
                                // parse_tree should also always be Some(_)
                                .and_then(|pl| pl.parse_tree.as_ref()),
                            None => None,
                        })
                        .collect();

                    while proof_line_parse_trees.len() < theorem_parse_trees.len() - 1 {
                        proof_line_parse_trees.push(None);
                    }

                    proof_line_parse_trees.push(proof_line_parsed.parse_tree.as_ref());

                    let new_parse_trees = unify_step_with_ref(
                        &theorem_parse_trees,
                        &proof_line_parse_trees,
                        &mm_data.optimized_data.grammar,
                        &mm_data.optimized_data.symbol_number_mapping,
                    );
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

fn unify_step_with_ref(
    theorem_parse_trees: &Vec<&ParseTree>,
    proof_line_parse_trees: &Vec<Option<&ParseTree>>,
    grammar: &Grammar,
    symbol_number_mapping: &SymbolNumberMapping,
) -> Result<Vec<ParseTree>, Error> {
    let mut nodes: Vec<(&ParseTreeNode, &ParseTreeNode)> = theorem_parse_trees
        .iter()
        .zip(proof_line_parse_trees.iter())
        .filter_map(|(t_pt, pl_pt)| match pl_pt {
            Some(pt) => Some((&t_pt.top_node, &pt.top_node)),
            None => None,
        })
        .collect();

    let mut work_variable_substitutions: HashMap<WorkVariable, ParseTreeNode> = HashMap::new();

    let mut floating_hypothesis_rules: HashSet<u32> = HashSet::new();
    for theorem_pt in theorem_parse_trees {
        let pt_floating_hypothesis_rules =
            theorem_pt.top_node.get_floating_hypotheses_rules(grammar)?;
        floating_hypothesis_rules.extend(pt_floating_hypothesis_rules.into_iter());
    }

    let mut work_variable_manager = WorkVariableManager::new(
        &proof_line_parse_trees.iter().filter_map(|pt| *pt).collect(),
        symbol_number_mapping,
    )?;

    let mut theorem_variable_substitutions: HashMap<u32, ParseTreeNode> = floating_hypothesis_rules
        .into_iter()
        .map(|rule_i| {
            Ok((
                rule_i,
                ParseTreeNode::WorkVariable(
                    work_variable_manager
                        .next_var(
                            grammar
                                .rules
                                .get(rule_i as usize)
                                .ok_or(Error::InternalLogicError)?
                                .left_side
                                .symbol_i,
                        )
                        .ok_or(Error::InternalLogicError)?,
                ),
            ))
        })
        .collect::<Result<HashMap<u32, ParseTreeNode>, Error>>()?;

    while let Some((theorem_pt, proof_line_pt)) = nodes.pop() {
        match theorem_pt {
            ParseTreeNode::WorkVariable(_) => return Err(Error::InternalLogicError),
            ParseTreeNode::Node {
                rule_i: theorem_rule_i,
                sub_nodes: theorem_sub_nodes,
            } => {
                if grammar
                    .rules
                    .get(*theorem_rule_i as usize)
                    .ok_or(Error::InternalLogicError)?
                    .is_floating_hypothesis
                {
                    let Some(pt) = theorem_variable_substitutions.get(theorem_rule_i) else {
                        return Err(Error::InternalLogicError);
                    };

                    unify_theorem_variable_substitutions(
                        proof_line_pt,
                        pt,
                        &mut work_variable_substitutions,
                        grammar,
                    )?;
                } else {
                    match proof_line_pt {
                        ParseTreeNode::Node {
                            rule_i: proof_line_rule_i,
                            sub_nodes: proof_line_sub_nodes,
                        } => {
                            if theorem_rule_i != proof_line_rule_i {
                                return Err(Error::UnificationError);
                            }
                            if theorem_sub_nodes.len() != proof_line_sub_nodes.len() {
                                return Err(Error::InternalLogicError);
                            }
                            nodes.extend(theorem_sub_nodes.iter().zip(proof_line_sub_nodes.iter()));
                        }
                        ParseTreeNode::WorkVariable(work_variable) => {
                            match work_variable_substitutions.get(work_variable) {
                                Some(pt) => {
                                    unify_work_variable_substitutions(pt, theorem_pt, grammar)?;
                                }
                                None => {
                                    work_variable_substitutions
                                        .insert(*work_variable, theorem_pt.clone());
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(Vec::new())
}

fn unify_work_variable_substitutions(
    tree_1: &ParseTreeNode,
    tree_2: &ParseTreeNode,
    grammar: &Grammar,
) -> Result<(), Error> {
    let mut nodes: Vec<(&ParseTreeNode, &ParseTreeNode)> = vec![(tree_1, tree_2)];

    while let Some((node_1, node_2)) = nodes.pop() {
        let ParseTreeNode::Node {
            rule_i: rule_i_1,
            sub_nodes: sub_nodes_1,
        } = node_1
        else {
            return Err(Error::InternalLogicError);
        };
        let ParseTreeNode::Node {
            rule_i: rule_i_2,
            sub_nodes: sub_nodes_2,
        } = node_2
        else {
            return Err(Error::InternalLogicError);
        };

        match (
            grammar
                .rules
                .get(*rule_i_1 as usize)
                .ok_or(Error::InternalLogicError)?
                .is_floating_hypothesis,
            grammar
                .rules
                .get(*rule_i_2 as usize)
                .ok_or(Error::InternalLogicError)?
                .is_floating_hypothesis,
        ) {
            (true, true) => {}
            (true, false) => {}
            (false, true) => {}
            (false, false) => {
                if *rule_i_1 != *rule_i_2 {
                    return Err(Error::UnificationError);
                }
                if sub_nodes_1.len() != sub_nodes_2.len() {
                    return Err(Error::InternalLogicError);
                }
                sub_nodes_1
                    .iter()
                    .zip(sub_nodes_2.iter())
                    .for_each(|sub_nodes_tuple| nodes.push(sub_nodes_tuple));
            }
        }
    }

    Ok(())
}

fn unify_theorem_variable_substitutions(
    tree_1: &ParseTreeNode,
    tree_2: &ParseTreeNode,
    work_variable_substitutions: &mut HashMap<WorkVariable, ParseTreeNode>,
    grammar: &Grammar,
) -> Result<ParseTreeNode, Error> {
    let mut parse_tree_result: ParseTreeNode = tree_1.clone();

    let mut nodes: Vec<(&ParseTreeNode, &ParseTreeNode, Vec<usize>)> =
        vec![(tree_1, tree_2, Vec::new())];

    while let Some((node_1, node_2, path)) = nodes.pop() {
        match (node_1, node_2) {
            (
                ParseTreeNode::Node {
                    rule_i: rule_i_1,
                    sub_nodes: sub_nodes_1,
                },
                ParseTreeNode::Node {
                    rule_i: rule_i_2,
                    sub_nodes: sub_nodes_2,
                },
            ) => {
                if *rule_i_1 != *rule_i_2 {
                    return Err(Error::UnificationError);
                }
                if sub_nodes_1.len() != sub_nodes_2.len() {
                    return Err(Error::InternalLogicError);
                }
                nodes.extend(sub_nodes_1.iter().zip(sub_nodes_2.iter()).enumerate().map(
                    |(i, (node_1, node_2))| {
                        let mut new_path = path.clone();
                        new_path.push(i);
                        (node_1, node_2, new_path)
                    },
                ));
            }
            (
                ParseTreeNode::Node {
                    rule_i: _,
                    sub_nodes: _,
                },
                ParseTreeNode::WorkVariable(work_var),
            ) => match work_variable_substitutions.get(work_var) {
                Some(pt) => {
                    unify_work_variable_substitutions(pt, node_1, grammar)?;
                }
                None => {
                    work_variable_substitutions.insert(*work_var, node_1.clone());
                }
            },
            (
                ParseTreeNode::WorkVariable(work_var),
                ParseTreeNode::Node {
                    rule_i: _,
                    sub_nodes: _,
                },
            ) => match work_variable_substitutions.get(work_var) {
                Some(pt) => {
                    unify_work_variable_substitutions(pt, node_2, grammar)?;
                }
                None => {
                    work_variable_substitutions.insert(*work_var, node_2.clone());
                }
            },
            (ParseTreeNode::WorkVariable(work_var_1), ParseTreeNode::WorkVariable(work_var_2)) => {}
        }
    }

    Ok(parse_tree_result)
}

struct WorkVariableManager {
    next_vars: Vec<WorkVariable>,
}

impl WorkVariableManager {
    fn new(
        parse_trees: &Vec<&ParseTree>,
        symbol_number_mapping: &SymbolNumberMapping,
    ) -> Result<WorkVariableManager, Error> {
        let mut next_vars: Vec<WorkVariable> = symbol_number_mapping
            .typecode_default_vars
            .iter()
            .map(|(typecode_i, default_var_i)| WorkVariable {
                typecode_i: *typecode_i,
                variable_i: *default_var_i,
                number: 0,
            })
            .collect();

        let mut nodes: Vec<&ParseTreeNode> = parse_trees.iter().map(|pt| &pt.top_node).collect();

        while let Some(node) = nodes.pop() {
            if let ParseTreeNode::WorkVariable(work_var_in_pt) = node {
                if work_var_in_pt.variable_i
                    == symbol_number_mapping
                        .get_typecode_default_variable_i(work_var_in_pt.typecode_i)
                        .ok_or(Error::InternalLogicError)?
                {
                    let Some(work_var) = next_vars
                        .iter_mut()
                        .find(|work_var| work_var.typecode_i == work_var_in_pt.typecode_i)
                    else {
                        return Err(Error::InternalLogicError);
                    };

                    if work_var.number < work_var_in_pt.number + 1 {
                        work_var.number = work_var_in_pt.number + 1;
                    }
                }
            }
        }

        Ok(WorkVariableManager { next_vars })
    }

    fn next_var(&mut self, typecode_i: u32) -> Option<WorkVariable> {
        self.next_vars.iter_mut().find_map(|work_var| {
            if work_var.typecode_i == typecode_i {
                let return_var = work_var.clone();
                work_var.number += 1;
                Some(return_var)
            } else {
                None
            }
        })
    }
}
