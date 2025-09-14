use std::collections::HashMap;

use serde::Deserialize;
use tauri::async_runtime::Mutex;

use crate::{
    model::{
        Header, ListEntry, MetamathData, ParseTree, ParseTreeNode, Theorem, TheoremListData,
        TheoremParseTrees,
    },
    util::earley_parser_optimized::WorkVariable,
    AppState, Error,
};

// page starts at 0
#[derive(Deserialize)]
pub struct SearchParameters {
    pub page: u32,
    pub label: String,
    #[serde(rename = "searchByParseTree")]
    pub search_by_parse_tree: Vec<SearchByParseTreeCondition>,
    #[serde(rename = "allAxiomDependencies")]
    pub all_axiom_dependencies: Vec<String>,
    #[serde(rename = "anyAxiomDependencies")]
    pub any_axiom_dependencies: Vec<String>,
    #[serde(rename = "avoidAxiomDependencies")]
    pub avoid_axiom_dependencies: Vec<String>,
    #[serde(rename = "allDefinitionDependencies")]
    pub all_definition_dependencies: Vec<String>,
    #[serde(rename = "anyDefinitionDependencies")]
    pub any_definition_dependencies: Vec<String>,
    #[serde(rename = "avoidDefinitionDependencies")]
    pub avoid_definition_dependencies: Vec<String>,
    #[serde(rename = "allowTheorems")]
    pub allow_theorems: bool,
    #[serde(rename = "allowAxioms")]
    pub allow_axioms: bool,
    #[serde(rename = "allowDefinitions")]
    pub allow_definitions: bool,
    #[serde(rename = "allowSyntaxAxioms")]
    pub allow_syntax_axioms: bool,
}

#[derive(Deserialize)]
pub struct SearchByParseTreeCondition {
    #[serde(rename = "searchTarget")]
    search_target: String, // "anyHypothesis" | "allHpotheses" | "assertion" | "anyExpressions" | "allExpressions"
    #[serde(rename = "searchCondition")]
    search_condition: String, // "matches" | "contains"
    search: String,
}

#[tauri::command]
pub async fn search_theorems(
    state: tauri::State<'_, Mutex<AppState>>,
    search_parameters: SearchParameters,
) -> Result<TheoremListData, Error> {
    let app_state = state.lock().await;
    let metamath_data = app_state.metamath_data.as_ref().ok_or(Error::NoMmDbError)?;

    let all_axiom_dependencies_indexes: Vec<usize> =
        calc_theorem_label_vec_to_ordered_theorem_i_vec_if_non_empty(
            &search_parameters.all_axiom_dependencies,
            metamath_data,
        );

    let any_axiom_dependencies_indexes: Vec<usize> =
        calc_theorem_label_vec_to_ordered_theorem_i_vec_if_non_empty(
            &search_parameters.any_axiom_dependencies,
            metamath_data,
        );

    let avoid_axiom_dependencies_indexes: Vec<usize> =
        calc_theorem_label_vec_to_ordered_theorem_i_vec_if_non_empty(
            &search_parameters.avoid_axiom_dependencies,
            metamath_data,
        );

    let all_definition_dependencies_indexes: Vec<usize> =
        calc_theorem_label_vec_to_ordered_theorem_i_vec_if_non_empty(
            &search_parameters.all_definition_dependencies,
            metamath_data,
        );

    let any_definition_dependencies_indexes: Vec<usize> =
        calc_theorem_label_vec_to_ordered_theorem_i_vec_if_non_empty(
            &search_parameters.any_definition_dependencies,
            metamath_data,
        );

    let avoid_definition_dependencies_indexes: Vec<usize> =
        calc_theorem_label_vec_to_ordered_theorem_i_vec_if_non_empty(
            &search_parameters.avoid_definition_dependencies,
            metamath_data,
        );

    let mut theorem_amount: i32 = 0;
    let mut list: Vec<ListEntry> = Vec::new();
    let mut page_limits: Vec<(u32, u32)> = Vec::new();
    let mut last_page_start: Option<u32> = None;
    let mut last_theorem_number: Option<u32> = None;

    metamath_data
        .database_header
        .theorem_iter()
        .enumerate()
        .filter(|(_, theorem)| {
            let optimized_theorem_data = metamath_data
                .optimized_data
                .theorem_data
                .get(&theorem.label)
                .unwrap();

            theorem.label.contains(&search_parameters.label)
                && (search_parameters.search_by_parse_tree.len() == 0
                    || optimized_theorem_data.parse_trees.as_ref().is_some_and(
                        |theorem_parse_trees| {
                            search_parameters
                                .search_by_parse_tree
                                .iter()
                                .all(|condition| {
                                    check_search_by_parse_tree_condition(
                                        condition,
                                        theorem_parse_trees,
                                        metamath_data,
                                    )
                                })
                        },
                    ))
                && ordered_list_contained_in_other_ordered_list(
                    &all_axiom_dependencies_indexes,
                    &optimized_theorem_data.axiom_dependencies,
                )
                && (any_axiom_dependencies_indexes.is_empty()
                    || !ordered_list_disjoint_from_other_ordered_list(
                        &any_axiom_dependencies_indexes,
                        &optimized_theorem_data.axiom_dependencies,
                    ))
                && ordered_list_disjoint_from_other_ordered_list(
                    &avoid_axiom_dependencies_indexes,
                    &optimized_theorem_data.axiom_dependencies,
                )
                && ordered_list_contained_in_other_ordered_list(
                    &all_definition_dependencies_indexes,
                    &optimized_theorem_data.definition_dependencies,
                )
                && (any_definition_dependencies_indexes.is_empty()
                    || !ordered_list_disjoint_from_other_ordered_list(
                        &any_definition_dependencies_indexes,
                        &optimized_theorem_data.definition_dependencies,
                    ))
                && ordered_list_disjoint_from_other_ordered_list(
                    &avoid_definition_dependencies_indexes,
                    &optimized_theorem_data.definition_dependencies,
                )
                && (search_parameters.allow_theorems
                    || !optimized_theorem_data.theorem_type.is_theorem())
                && (search_parameters.allow_axioms
                    || !optimized_theorem_data.theorem_type.is_axiom())
                && (search_parameters.allow_definitions
                    || !optimized_theorem_data.theorem_type.is_definition())
                && (search_parameters.allow_syntax_axioms
                    || !optimized_theorem_data.theorem_type.is_syntax_axiom())
        })
        .for_each(|(theorem_number, theorem)| {
            last_theorem_number = Some((theorem_number + 1) as u32);

            if theorem_amount % 100 == 0 {
                last_page_start = Some((theorem_number + 1) as u32);
            } else if theorem_amount % 100 == 99 {
                page_limits.push((
                    last_page_start.take().unwrap_or(0),
                    (theorem_number + 1) as u32,
                ));
            }

            if search_parameters.page * 100 <= theorem_amount as u32
                && (theorem_amount as u32) < (search_parameters.page + 1) * 100
            {
                list.push(ListEntry::Theorem(theorem.to_theorem_list_entry(
                    (theorem_number + 1) as u32,
                    &metamath_data.optimized_data,
                )));
            }

            theorem_amount += 1;
        });

    if let Some(last_theorem_number) = last_theorem_number {
        if let Some(last_page_start) = last_page_start {
            page_limits.push((last_page_start, last_theorem_number));
        }
    }

    let page_amount = (((theorem_amount - 1) / 100) + 1) as u32;

    Ok(TheoremListData {
        list,
        page_amount,
        page_limits: Some(page_limits),
    })
}

fn calc_theorem_label_vec_to_ordered_theorem_i_vec_if_non_empty(
    list: &Vec<String>,
    metamath_data: &MetamathData,
) -> Vec<usize> {
    if !list.is_empty() {
        metamath_data
            .database_header
            .theorem_label_vec_to_ordered_theorem_i_vec(list)
    } else {
        Vec::new()
    }
}

fn check_search_by_parse_tree_condition(
    condition: &SearchByParseTreeCondition,
    parse_trees: &TheoremParseTrees,
    metamath_data: &MetamathData,
) -> bool {
    let search_parse_tree = metamath_data
        .expression_to_parse_tree(&condition.search)
        // Safe unwrap due to syntax check
        .unwrap();

    let (parse_trees_vec, and) = match condition.search_target.as_str() {
        "anyHypothesis" => (parse_trees.hypotheses_parsed.clone(), false),
        "allHypotheses" => (parse_trees.hypotheses_parsed.clone(), true),
        "assertion" => (vec![parse_trees.assertion_parsed.clone()], false),
        "anyExpressions" => (parse_trees.to_cloned_parse_tree_vec(), false),
        "allExpressions" => (parse_trees.to_cloned_parse_tree_vec(), true),
        _ => return false,
    };

    let check_parse_tree_pair_closure =
        |parse_tree: &ParseTree| match condition.search_condition.as_str() {
            "matches" => parse_tree_matches(
                &search_parse_tree.top_node,
                search_parse_tree.typecode,
                &parse_tree.top_node,
                parse_tree.typecode,
                metamath_data,
            ),
            "contains" => parse_tree_contains(&search_parse_tree, parse_tree, metamath_data),
            _ => false,
        };

    if and {
        parse_trees_vec.iter().all(check_parse_tree_pair_closure)
    } else {
        parse_trees_vec.iter().any(check_parse_tree_pair_closure)
    }
}

fn parse_tree_matches(
    search_parse_tree: &ParseTreeNode,
    search_parse_tree_typecode: u32,
    parse_tree: &ParseTreeNode,
    parse_tree_typecode: u32,
    metamath_data: &MetamathData,
) -> bool {
    if search_parse_tree_typecode != parse_tree_typecode {
        return false;
    }

    let mut work_variable_substitutions: HashMap<WorkVariable, &ParseTreeNode> = HashMap::new();
    let mut floating_hypothesis_substitutions: HashMap<u32, u32> = HashMap::new();

    let mut nodes_to_check = vec![(search_parse_tree, parse_tree)];

    while let Some((search_node, node)) = nodes_to_check.pop() {
        let ParseTreeNode::Node { rule_i, sub_nodes } = node else {
            // Should never happen
            return false;
        };

        match search_node {
            ParseTreeNode::Node {
                rule_i: search_rule_i,
                sub_nodes: search_sub_nodes,
            } => {
                let Some(rule) = metamath_data
                    .optimized_data
                    .grammar
                    .rules
                    .get(*rule_i as usize)
                else {
                    // Should never happen
                    return false;
                };
                let Some(search_rule) = metamath_data
                    .optimized_data
                    .grammar
                    .rules
                    .get(*search_rule_i as usize)
                else {
                    // Should never happen
                    return false;
                };

                match (
                    rule.is_floating_hypothesis,
                    search_rule.is_floating_hypothesis,
                ) {
                    (true, true) => match floating_hypothesis_substitutions.get(search_rule_i) {
                        Some(sub_rule_i) => {
                            if sub_rule_i != rule_i {
                                println!("test");
                                return false;
                            }
                        }
                        None => {
                            floating_hypothesis_substitutions.insert(*search_rule_i, *rule_i);
                        }
                    },
                    (true, false) | (false, true) => return false,
                    (false, false) => {
                        if rule_i != search_rule_i {
                            return false;
                        }
                    }
                }

                nodes_to_check.extend(search_sub_nodes.iter().zip(sub_nodes.iter()));
            }
            ParseTreeNode::WorkVariable(work_variable) => {
                match work_variable_substitutions.get(work_variable) {
                    Some(substitution) => {
                        if node != *substitution {
                            println!("test");
                            return false;
                        }
                    }
                    None => {
                        work_variable_substitutions.insert(*work_variable, node);
                    }
                }
            }
        }
    }

    true
}

fn parse_tree_contains(
    search_parse_tree: &ParseTree,
    parse_tree: &ParseTree,
    metamath_data: &MetamathData,
) -> bool {
    if parse_tree_matches(
        &search_parse_tree.top_node,
        search_parse_tree.typecode,
        &parse_tree.top_node,
        parse_tree.typecode,
        metamath_data,
    ) {
        return true;
    }

    let mut nodes_to_check = vec![&parse_tree.top_node];

    while let Some(node) = nodes_to_check.pop() {
        let ParseTreeNode::Node { rule_i, sub_nodes } = node else {
            // should never happen
            return false;
        };

        let Some(rule) = metamath_data
            .optimized_data
            .grammar
            .rules
            .get(*rule_i as usize)
        else {
            // should never happen
            return false;
        };

        let typecode_with_dollar_i = rule.left_side.symbol_i;

        let Some(typecode_with_dollar) = metamath_data
            .optimized_data
            .symbol_number_mapping
            .symbols
            .get(&typecode_with_dollar_i)
        else {
            // should never happen
            return false;
        };

        let typecode = &typecode_with_dollar[1..];

        let Some(typecode_i) = metamath_data
            .optimized_data
            .symbol_number_mapping
            .numbers
            .get(typecode)
        else {
            // should never happen
            return false;
        };

        if parse_tree_matches(
            &search_parse_tree.top_node,
            search_parse_tree.typecode,
            node,
            *typecode_i,
            metamath_data,
        ) {
            return true;
        }

        nodes_to_check.extend(sub_nodes.iter());
    }

    false
}

fn ordered_list_contained_in_other_ordered_list(
    list: &Vec<usize>,
    other_list: &Vec<usize>,
) -> bool {
    let mut other_item_i = 0;

    for &item in list {
        loop {
            let Some(&other_item) = other_list.get(other_item_i) else {
                return false;
            };

            other_item_i += 1;

            if other_item == item {
                break;
            } else if other_item > item {
                return false;
            }
        }
    }

    true
}

fn ordered_list_disjoint_from_other_ordered_list(
    list: &Vec<usize>,
    other_list: &Vec<usize>,
) -> bool {
    let mut other_item_i = 0;

    for &item in list {
        loop {
            let Some(&other_item) = other_list.get(other_item_i) else {
                return true;
            };

            if other_item == item {
                return false;
            } else if other_item > item {
                break;
            }

            other_item_i += 1;
        }
    }

    true
}

#[tauri::command]
pub async fn search_by_parse_tree_syntax_check(
    state: tauri::State<'_, Mutex<AppState>>,
    search: &str,
) -> Result<bool, Error> {
    let app_state = state.lock().await;
    let metamath_data = app_state.metamath_data.as_ref().ok_or(Error::NoMmDbError)?;

    let _ = metamath_data
        .expression_to_parse_tree(search)
        .inspect(|pt| println!("{:?}", pt));

    Ok(metamath_data.expression_to_parse_tree(search).is_ok())
}

// If successful, returns a tuple (a,b) where:
// a is whether the query is a valid axiom label
// b is a list of 5 axiom labels to be shown as autocomplete
#[tauri::command]
pub async fn axiom_autocomplete(
    state: tauri::State<'_, Mutex<AppState>>,
    query: &str,
    items: Vec<&str>,
) -> Result<(bool, Vec<String>), Error> {
    let app_state = state.lock().await;
    let metamath_data = app_state.metamath_data.as_ref().ok_or(Error::NoMmDbError)?;

    Ok((
        metamath_data
            .database_header
            .find_theorem_by_label(query)
            .is_some_and(|theorem| {
                theorem.calc_theorem_type(&app_state.settings).is_axiom()
                    && !items.contains(&&*theorem.label)
            }),
        find_theorem_labels(&metamath_data.database_header, query, 5, |theorem| {
            theorem.label != query
                && theorem.calc_theorem_type(&app_state.settings).is_axiom()
                && !items.contains(&&*theorem.label)
        }),
    ))
}

// If successful, returns a tuple (a,b) where:
// a is whether the query is a valid definition label
// b is a list of 5 definition labels to be shown as autocomplete
#[tauri::command]
pub async fn definition_autocomplete(
    state: tauri::State<'_, Mutex<AppState>>,
    query: &str,
    items: Vec<&str>,
) -> Result<(bool, Vec<String>), Error> {
    let app_state = state.lock().await;
    let metamath_data = app_state.metamath_data.as_ref().ok_or(Error::NoMmDbError)?;

    Ok((
        metamath_data
            .database_header
            .find_theorem_by_label(query)
            .is_some_and(|theorem| {
                theorem
                    .calc_theorem_type(&app_state.settings)
                    .is_definition()
                    && !items.contains(&&*theorem.label)
            }),
        find_theorem_labels(&metamath_data.database_header, query, 5, |theorem| {
            theorem.label != query
                && theorem
                    .calc_theorem_type(&app_state.settings)
                    .is_definition()
                && !items.contains(&&*theorem.label)
        }),
    ))
}

// Find all theorem labels that match the query in the following order:
// 1: The name that fully matches the query (if it exists)
// 2: Labels that start with the query
// 3: Labels that contain the query
pub fn find_theorem_labels<T>(header: &Header, query: &str, limit: u32, filter: T) -> Vec<String>
where
    T: Fn(&Theorem) -> bool,
{
    let mut theorems = Vec::new();

    let exact_match = header.find_theorem_by_label(query);

    if let Some(theorem) = exact_match {
        if filter(theorem) {
            theorems.push(theorem.label.clone())
        }
    }

    theorems.extend(
        header
            .theorem_iter()
            .filter(|t| t.label != query && t.label.starts_with(query) && filter(t))
            .take((limit as usize) - theorems.len())
            .map(|t| t.label.clone()),
    );

    theorems.extend(
        header
            .theorem_iter()
            .filter(|t| {
                t.label != query
                    && !t.label.starts_with(query)
                    && t.label.contains(query)
                    && filter(t)
            })
            .take((limit as usize) - theorems.len())
            .map(|t| t.label.clone()),
    );

    theorems
}
