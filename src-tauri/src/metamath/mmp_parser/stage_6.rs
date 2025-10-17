use std::collections::{HashMap, HashSet};

use crate::{
    metamath::{
        mmp_parser::{MmpParserStage4Success, MmpParserStage5, MmpParserStage6, UnifyLine},
        verify::ProofNumber,
    },
    model::{MetamathData, ParseTree, ParseTreeNode},
    util::StrIterToSpaceSeperatedString,
    Error, ProofFormatOption, Settings,
};

pub fn stage_6(
    stage_4: &MmpParserStage4Success,
    stage_5: &MmpParserStage5,
    mm_data: &MetamathData,
    settings: &Settings,
) -> Result<MmpParserStage6, Error> {
    let Some(proof_tree) = calc_proof_tree(
        &stage_5.unify_result,
        &stage_4.distinct_variable_pairs,
        mm_data,
    )?
    else {
        return Ok(MmpParserStage6 { proof: None });
    };

    Ok(MmpParserStage6 {
        proof: Some(match settings.proof_format {
            ProofFormatOption::Uncompressed => calc_uncompressed_proof(&proof_tree),
            ProofFormatOption::Compressed => {
                calc_compressed_proof(proof_tree, &stage_5.unify_result, mm_data)?
            }
        }),
    })
}

fn calc_proof_tree<'a>(
    unify_result: &'a Vec<UnifyLine>,
    distinct_variable_pairs: &HashSet<(String, String)>,
    mm_data: &'a MetamathData,
) -> Result<Option<ProofTree<'a>>, Error> {
    let Some(qed_step_i) = unify_result.iter().position(|ul| ul.step_name == "qed") else {
        return Ok(None);
    };

    if unify_result
        .iter()
        .filter(|ul| ul.is_hypothesis || ul.step_name == "qed")
        .any(|ul| ul.parse_tree.is_none())
    {
        return Ok(None);
    }

    let mut proofs: Vec<Option<ProofTree>> = Vec::new();

    for (i, unify_line) in unify_result.iter().enumerate() {
        if unify_line.is_hypothesis {
            proofs.push(Some(ProofTree {
                label: &*unify_line.step_ref,
                children: Vec::new(),
            }));
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
                        pt_node.calc_proof_tree(&mm_data.optimized_data.grammar)?,
                    ))
                })
                .collect::<Result<HashMap<&str, ProofTree>, Error>>()?;

            let vars_proof_children: Vec<ProofTree> = mm_data
                .optimized_data
                .floating_hypotheses
                .iter()
                .filter_map(|fh| var_proofs.remove(&*fh.variable))
                .collect();

            let Some(mut essential_proof_children) = unify_line_hypotheses_parsed
                .into_iter()
                .map(|hyp_i| Ok((*proofs.get(hyp_i).ok_or(Error::InternalLogicError)?).clone()))
                .collect::<Result<Option<Vec<ProofTree>>, Error>>()?
            else {
                proofs.push(None);
                continue;
            };

            let mut children = vars_proof_children;
            children.append(&mut essential_proof_children);

            let label = &*unify_line.step_ref;

            proofs.push(Some(ProofTree { label, children }));

            if unify_line.step_name == "qed" {
                break;
            }
        }

        // println!("{:#?}", proofs.last().unwrap());
    }

    Ok(proofs.swap_remove(qed_step_i))
}

fn calc_uncompressed_proof(proof_tree: &ProofTree) -> String {
    proof_tree
        .iter()
        .map(|pt| pt.label)
        .fold_to_space_seperated_string()
}

fn calc_compressed_proof(
    proof_tree: ProofTree,
    unify_result: &Vec<UnifyLine>,
    mm_data: &MetamathData,
) -> Result<String, Error> {
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

    let mut nodes_seen: HashMap<&ProofTree<'_>, usize> = HashMap::new();
    let compressed_proof_tree = compress_proof_tree(&proof_tree, &mut nodes_seen);

    let mut steps: Vec<&str> = Vec::new();
    let mut compressed_steps: Vec<usize> = Vec::new();

    for node in compressed_proof_tree.iter() {
        match node {
            CompressedProofTree::ProofTree(pt, _) => {
                if floating_hypotheses
                    .iter()
                    .chain(hypotheses.iter())
                    .chain(steps.iter())
                    .find(|s| **s == pt.label)
                    .is_none()
                {
                    steps.push(pt.label);
                }
            }
            CompressedProofTree::Compressed(compressed_i) => {
                match compressed_steps.binary_search(compressed_i) {
                    Ok(_) => {}
                    Err(i) => compressed_steps.insert(i, *compressed_i),
                }
            }
        }
    }

    let mut compressed_steps_in_order: Vec<usize> = Vec::new();

    for node in compressed_proof_tree.iter() {
        if let CompressedProofTree::ProofTree(_, Some(compressed_i)) = node {
            if compressed_steps.binary_search(compressed_i).is_ok() {
                compressed_steps_in_order.push(*compressed_i);
            }
        }
    }

    let mut compressed_steps_string = String::new();

    for node in compressed_proof_tree.iter() {
        let step_num = match node {
            CompressedProofTree::ProofTree(pt, i) => ProofNumber {
                number: (floating_hypotheses
                    .iter()
                    .chain(hypotheses.iter())
                    .chain(steps.iter())
                    .position(|s| *s == pt.label)
                    .ok_or(Error::InternalLogicError)?
                    + 1) as u32,
                save: i.is_some_and(|i| compressed_steps.binary_search(&i).is_ok()),
            },
            CompressedProofTree::Compressed(compressed_i) => ProofNumber {
                number: (floating_hypotheses.len()
                    + hypotheses.len()
                    + steps.len()
                    + compressed_steps_in_order
                        .iter()
                        .position(|ci| ci == compressed_i)
                        .ok_or(Error::InternalLogicError)?
                    + 1) as u32,
                save: false,
            },
        };
        compressed_steps_string
            .push_str(&number_to_compressed_proof_format_number(step_num.number));
        if step_num.save {
            compressed_steps_string.push('Z');
        }
    }

    let mut result = steps.iter().fold("( ".to_string(), |mut s, pus| {
        s.push_str(pus);
        s.push(' ');
        s
    });

    result.push_str(") ");
    result.push_str(&compressed_steps_string);

    Ok(result)
}

fn compress_proof_tree<'a, 'b>(
    proof_tree: &'b ProofTree<'a>,
    nodes_seen: &mut HashMap<&'b ProofTree<'a>, usize>,
) -> CompressedProofTree<'a> {
    match nodes_seen.get(proof_tree) {
        Some(compressed_i) => CompressedProofTree::Compressed(*compressed_i),
        None => {
            let i = if proof_tree.children.len() != 0 {
                nodes_seen.insert(proof_tree, nodes_seen.len());
                Some(nodes_seen.len() - 1)
            } else {
                None
            };
            CompressedProofTree::ProofTree(
                CompressedProofTreeNode {
                    label: proof_tree.label,
                    children: proof_tree
                        .children
                        .iter()
                        .map(|pt| compress_proof_tree(pt, nodes_seen))
                        .collect(),
                },
                i,
            )
        }
    }
}

fn number_to_compressed_proof_format_number(mut number: u32) -> String {
    if number == 0 {
        return String::new();
    }

    let mut compressed_number = String::new();

    compressed_number.push(('A' as u8 + ((number - 1) % 20) as u8) as char);

    number = (number - ((number - 1) % 20) + 1) / 20;

    while number != 0 {
        compressed_number.push(('U' as u8 + ((number - 1) % 5) as u8) as char);
        number = (number - ((number - 1) % 5) + 1) / 5;
    }

    compressed_number.chars().rev().collect()
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ProofTree<'a> {
    pub label: &'a str,
    pub children: Vec<ProofTree<'a>>,
}

impl<'a> ProofTree<'a> {
    pub fn iter<'b>(&'b self) -> ProofTreeIterator<'a, 'b> {
        ProofTreeIterator::new(self)
    }
}

pub struct ProofTreeIterator<'a, 'b> {
    proof_tree: &'b ProofTree<'a>,
    current_path: Vec<(usize, &'b ProofTree<'a>)>,
    finished: bool,
}

impl<'a, 'b> ProofTreeIterator<'a, 'b> {
    pub fn new(proof_tree: &'b ProofTree<'a>) -> ProofTreeIterator<'a, 'b> {
        let mut current_path = Vec::new();

        let mut current_node = proof_tree;

        while let Some(child) = current_node.children.first() {
            current_path.push((0, current_node));

            current_node = child;
        }

        ProofTreeIterator {
            proof_tree,
            current_path,
            finished: false,
        }
    }
}

impl<'a, 'b> Iterator for ProofTreeIterator<'a, 'b> {
    type Item = &'b ProofTree<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        let Some((next_node_i, node)) = self.current_path.last_mut() else {
            self.finished = true;
            return Some(self.proof_tree);
        };

        //should never return None
        let target = node.children.get(*next_node_i)?;

        *next_node_i += 1;

        if let Some(next_child) = node.children.get(*next_node_i) {
            let mut child = next_child;
            while let Some(next_child) = child.children.first() {
                self.current_path.push((0, child));
                child = next_child;
            }
        } else {
            self.current_path.pop();
        }

        Some(target)
    }
}

enum CompressedProofTree<'a> {
    ProofTree(CompressedProofTreeNode<'a>, Option<usize>),
    Compressed(usize),
}

struct CompressedProofTreeNode<'a> {
    pub label: &'a str,
    pub children: Vec<CompressedProofTree<'a>>,
}

impl<'a> CompressedProofTree<'a> {
    pub fn iter<'b>(&'b self) -> CompressedProofTreeIterator<'a, 'b> {
        CompressedProofTreeIterator::new(self)
    }
}
struct CompressedProofTreeIterator<'a, 'b> {
    proof_tree: &'b CompressedProofTree<'a>,
    current_path: Vec<(usize, &'b CompressedProofTree<'a>)>,
    finished: bool,
}

impl<'a, 'b> CompressedProofTreeIterator<'a, 'b> {
    pub fn new(proof_tree: &'b CompressedProofTree<'a>) -> CompressedProofTreeIterator<'a, 'b> {
        let mut current_path = Vec::new();

        let mut current_node = proof_tree;

        loop {
            let CompressedProofTree::ProofTree(current_pt_node, _) = current_node else {
                break;
            };
            let Some(child) = current_pt_node.children.first() else {
                break;
            };

            current_path.push((0, current_node));

            current_node = child;
        }

        CompressedProofTreeIterator {
            proof_tree,
            current_path,
            finished: false,
        }
    }
}

impl<'a, 'b> Iterator for CompressedProofTreeIterator<'a, 'b> {
    type Item = &'b CompressedProofTree<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        let Some((next_node_i, node)) = self.current_path.last_mut() else {
            self.finished = true;
            return Some(self.proof_tree);
        };

        //should never return None
        let CompressedProofTree::ProofTree(pt_node, _) = node else {
            return None;
        };
        //should never return None
        let target = pt_node.children.get(*next_node_i)?;

        *next_node_i += 1;

        if let Some(next_child) = pt_node.children.get(*next_node_i) {
            let mut child = next_child;
            loop {
                let CompressedProofTree::ProofTree(current_pt_node, _) = child else {
                    break;
                };
                let Some(next_child) = current_pt_node.children.first() else {
                    break;
                };
                self.current_path.push((0, child));
                child = next_child;
            }
        } else {
            self.current_path.pop();
        }

        Some(target)
    }
}
