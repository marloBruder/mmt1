use std::{borrow::Borrow, collections::HashMap, slice::Iter, str::Split};

use crate::{
    metamath::mmp_parser::{ProofLine, UnifyLine},
    Error,
};

pub trait CalcIndention<'a> {
    type HypothesesType: Borrow<str> + ?Sized + 'a + PartialEq<str>;
    type HypothesesIter: Iterator<Item = &'a Self::HypothesesType>;

    fn step_name(&self) -> &str;

    fn hypotheses(&'a self) -> Self::HypothesesIter;
}

struct Tree<'a> {
    pub i: usize,
    pub label: &'a str,
    pub nodes: Vec<Tree<'a>>,
}

pub fn calc_indention<'a, T>(lines: &'a Vec<T>) -> Result<Vec<u32>, Error>
where
    T: CalcIndention<'a>,
{
    // calc tree rep
    let mut trees: Vec<Tree> = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        let mut nodes: Vec<Tree> = Vec::new();

        for hypothesis in line.hypotheses() {
            for (tree_i, tree) in trees.iter().enumerate() {
                if hypothesis == tree.label && hypothesis != "" {
                    nodes.push(trees.remove(tree_i));
                    break;
                }
            }
        }

        trees.push(Tree {
            i,
            label: line.step_name(),
            nodes,
        })
    }

    // calc indentions based on trees
    let mut indentions: HashMap<(usize, &str), u32> = HashMap::new();
    let mut current_indention = 1;
    let mut next_level: Vec<&Tree> = trees.iter().collect();
    let mut current_level: Vec<&Tree>;

    while next_level.len() != 0 {
        current_level = next_level;
        next_level = Vec::new();

        for tree in current_level {
            indentions.insert((tree.i, tree.label), current_indention);
            next_level.extend(tree.nodes.iter());
        }

        current_indention += 1;
    }

    let mut indentions_vec: Vec<u32> = Vec::new();

    for (i, proof_line) in lines.iter().enumerate() {
        indentions_vec.push(
            *indentions
                .get(&(i, proof_line.step_name()))
                .ok_or(Error::InternalLogicError)?,
        );
    }

    Ok(indentions_vec)
}

impl<'a, 'b> CalcIndention<'b> for ProofLine<'a>
where
    'a: 'b,
{
    type HypothesesType = str;
    type HypothesesIter = Split<'b, char>;

    fn step_name(&self) -> &str {
        self.step_name
    }

    fn hypotheses(&self) -> Self::HypothesesIter {
        self.hypotheses.split(',')
    }
}

impl<'a> CalcIndention<'a> for UnifyLine {
    type HypothesesType = String;
    type HypothesesIter = Iter<'a, String>;

    fn step_name(&self) -> &str {
        self.step_name.as_str()
    }

    fn hypotheses(&'a self) -> Self::HypothesesIter {
        self.hypotheses.iter()
    }
}
