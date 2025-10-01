use std::{collections::HashSet, ops::Deref};

pub mod description_parser;
pub mod earley_parser;
pub mod earley_parser_optimized;
pub mod header_iterators;
pub mod last_curr_next_iterator;
pub mod parse_tree_node_iterator;
pub mod work_variable_manager;

pub fn spaces(num: u32) -> &'static str {
    &"                                                                                "
        [0..(std::cmp::min(num as usize, 80))]
}

pub fn str_to_space_seperated_string(str: &str) -> String {
    str.split_ascii_whitespace()
        .fold_to_space_seperated_string()
}

pub trait StrIterToDelimiterSeperatedString
where
    Self: Sized,
    Self: Iterator,
    Self::Item: Deref<Target = str>,
{
    fn fold_to_delimiter_seperated_string(self, delimiter: &str) -> String {
        self.fold((true, String::new()), |(first, mut s), t| {
            if !first {
                s.push_str(delimiter);
            }
            s.push_str(&t);
            (false, s)
        })
        .1
    }
}

impl<T> StrIterToDelimiterSeperatedString for T
where
    T: Iterator,
    T::Item: Deref<Target = str>,
{
}

pub trait StrIterToSpaceSeperatedString
where
    Self: Sized,
    Self: Iterator,
    Self::Item: Deref<Target = str>,
{
    fn fold_to_space_seperated_string(self) -> String {
        self.fold_to_delimiter_seperated_string(" ")
    }
}

impl<T> StrIterToSpaceSeperatedString for T
where
    T: Iterator,
    T::Item: Deref<Target = str>,
{
}

pub trait ForEachWhile<I>
where
    Self: Sized,
    Self: Iterator<Item = I>,
{
    fn for_each_while<F>(mut self, mut f: F)
    where
        F: FnMut(I) -> bool,
    {
        while self.next().is_some_and(|item| f(item)) {}
    }
}

impl<I, T> ForEachWhile<I> for T where T: Iterator<Item = I> {}

pub fn is_valid_label(label: &str) -> bool {
    label
        .chars()
        .all(|c| matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.'))
}

pub fn is_valid_math_symbol(symbol: &str) -> bool {
    // range of printable non-whitespace ascii characters excluding '$'
    symbol.bytes().all(|b| matches!(b, 33..=35 | 37..=126))
}

pub fn new_lines_in_str(str: &str) -> u32 {
    str.chars().filter(|c| *c == '\n').count() as u32
}

pub fn new_lines_at_end_of_str(str: &str) -> u32 {
    str.chars()
        .rev()
        .take_while(|c| c.is_ascii_whitespace())
        .filter(|c| *c == '\n')
        .count() as u32
}

pub fn calc_distinct_variable_pairs<T>(distinct_vars: &Vec<T>) -> HashSet<(String, String)>
where
    T: Deref<Target = str>,
{
    let mut distinct_variable_pairs: HashSet<(String, String)> = HashSet::new();

    for distinct_var_condition in distinct_vars {
        let distinct_vars: Vec<&str> = distinct_var_condition.split_ascii_whitespace().collect();

        for var_1 in distinct_vars.iter() {
            for var_2 in distinct_vars.iter() {
                if var_1 != var_2 {
                    distinct_variable_pairs.insert((var_1.to_string(), var_2.to_string()));
                }
            }
        }
    }

    distinct_variable_pairs
}
