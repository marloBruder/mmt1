use std::{collections::HashSet, ops::Deref};

use sha2::{Digest, Sha256};

use crate::{
    metamath::mmp_parser::LocateAfterRef,
    model::{DatabaseElement, HeaderPath, MetamathData, Statement},
    util::last_curr_next_iterator::IntoLastCurrNextIterator,
    Error,
};

pub mod description_parser;
pub mod earley_parser;
pub mod earley_parser_optimized;
pub mod header_iterators;
pub mod last_curr_next_iterator;
pub mod parse_tree_node_iterator;
pub mod work_variable_manager;

pub fn spaces(num: u32) -> &'static str {
    &"                                                                                                                                                                                                                                                                                                                                                                                                                "
        [0..(std::cmp::min(num as usize, 400))]
}

pub fn str_to_space_seperated_string(str: &str) -> String {
    str.split_ascii_whitespace()
        .fold_to_space_seperated_string()
}

pub trait StrIterToDelimiterSeperatedString
where
    Self: Sized,
    Self: Iterator,
    Self::Item: AsRef<str>,
{
    fn fold_to_delimiter_seperated_string(self, delimiter: &str) -> String {
        self.fold((true, String::new()), |(first, mut s), t| {
            if !first {
                s.push_str(delimiter);
            }
            s.push_str(t.as_ref());
            (false, s)
        })
        .1
    }
}

impl<T> StrIterToDelimiterSeperatedString for T
where
    T: Iterator,
    T::Item: AsRef<str>,
{
}

pub trait StrIterToSpaceSeperatedString
where
    Self: Sized,
    Self: Iterator,
    Self::Item: AsRef<str>,
{
    fn fold_to_space_seperated_string(self) -> String {
        self.fold_to_delimiter_seperated_string(" ")
    }
}

impl<T> StrIterToSpaceSeperatedString for T
where
    T: Iterator,
    T::Item: AsRef<str>,
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

// Returns (a, b), where a is the line number and b is the column number of the last non-whitespace character
pub fn last_non_whitespace_pos(str: &str) -> (u32, u32) {
    let mut last_non_whitespace_line_number = 1;
    let mut last_non_whitespace_column_number = 1;

    let mut line_number = 1;
    let mut column_number = 0;

    for char in str.chars() {
        column_number += 1;

        if char == '\n' {
            line_number += 1;
            column_number = 0;
        }

        if !char.is_whitespace() {
            last_non_whitespace_line_number = line_number;
            last_non_whitespace_column_number = column_number;
        }
    }

    (
        last_non_whitespace_line_number,
        last_non_whitespace_column_number,
    )
}

pub fn nth_token_start_pos(str: &str, n: u32) -> (u32, u32) {
    let mut tokens_seen = 0;
    let mut seeing_token = false;

    let mut line_number = 1;
    let mut column_number = 0;

    for char in str.chars() {
        column_number += 1;

        if char == '\n' {
            line_number += 1;
            column_number = 0;
        }

        if char.is_whitespace() {
            if seeing_token {
                tokens_seen += 1;
            }

            seeing_token = false;
        } else {
            if tokens_seen == n {
                break;
            }

            seeing_token = true;
        }
    }

    (line_number, column_number)
}

pub fn nth_token_end_pos(str: &str, n: u32) -> (u32, u32) {
    let mut tokens_seen = 0;
    let mut seeing_token = false;

    let mut line_number = 1;
    let mut column_number = 0;

    for char in str.chars() {
        column_number += 1;

        if char.is_whitespace() {
            if seeing_token {
                if tokens_seen == n {
                    column_number -= 1;
                    break;
                }

                tokens_seen += 1;
            }

            seeing_token = false;
        } else {
            seeing_token = true;
        }

        if char == '\n' {
            line_number += 1;
            column_number = 0;
        }
    }

    (line_number, column_number)
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

pub fn calc_next_header_path(header_path: &mut HeaderPath, depth: u32) -> Result<(), Error> {
    if depth > header_path.path.len() as u32 {
        header_path.path.push(0);
    } else if depth == header_path.path.len() as u32 {
        *header_path
            .path
            .last_mut()
            .ok_or(Error::InternalLogicError)? += 1;
    } else if depth < header_path.path.len() as u32 {
        while depth < header_path.path.len() as u32 {
            header_path.path.pop();
        }
        *header_path
            .path
            .last_mut()
            .ok_or(Error::InternalLogicError)? += 1;
    }

    Ok(())
}

pub fn str_to_hash_string(str: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(str);
    let hash_result = hasher.finalize();

    hash_result
        .into_iter()
        .map(|byte| format!("{:02x}", byte))
        .collect()
}

pub fn is_valid_comment_path(str: &str) -> bool {
    str.split_once('#')
        .is_some_and(|(header_path, comment_num)| {
            HeaderPath::from_str(header_path).is_some()
                && comment_num.parse::<usize>().is_ok_and(|num| num != 0)
                && !comment_num.contains('+')
        })
}

pub fn locate_after_to_mmp_file_format_of_statement_it_refers_to(
    locate_after: LocateAfterRef,
    mm_data: &MetamathData,
) -> Result<String, Error> {
    let mut header_path_of_last = HeaderPath::new();
    let mut comment_i_of_last = 0;
    let mut statement_and_locate_after: Option<(&Statement, LocateAfterRef)> = None;

    let mut locate_after_ref_string;

    for (last, curr, next) in mm_data
        .database_header
        .locate_after_iter(Some(locate_after))
        .last_curr_next()
    {
        if let Some(DatabaseElement::Header(_, depth)) = last {
            calc_next_header_path(&mut header_path_of_last, depth)?;
            comment_i_of_last = 0;
        }
        if let Some(DatabaseElement::Statement(Statement::CommentStatement(_))) = last {
            comment_i_of_last += 1;
        }

        if next.is_none() {
            let DatabaseElement::Statement(statement) = curr else {
                return Err(Error::InternalLogicError);
            };

            let locate_after = match last {
                None => LocateAfterRef::LocateAfterStart,
                Some(e) => match e {
                    DatabaseElement::Header(_, _) => {
                        locate_after_ref_string = header_path_of_last.to_string();
                        LocateAfterRef::LocateAfterHeader(&locate_after_ref_string)
                    }
                    DatabaseElement::Statement(s) => match s {
                        Statement::CommentStatement(_) => {
                            locate_after_ref_string = format!(
                                "{}#{}",
                                header_path_of_last.to_string(),
                                comment_i_of_last
                            );
                            LocateAfterRef::LocateAfterComment(&locate_after_ref_string)
                        }
                        Statement::ConstantStatement(consts) => {
                            let c = consts.first().ok_or(Error::InternalLogicError)?;
                            LocateAfterRef::LocateAfterConst(&c.symbol)
                        }
                        Statement::VariableStatement(vars) => {
                            let v = vars.first().ok_or(Error::InternalLogicError)?;
                            LocateAfterRef::LocateAfterConst(&v.symbol)
                        }
                        Statement::FloatingHypohesisStatement(fh) => {
                            LocateAfterRef::LocateAfter(&fh.label)
                        }
                        Statement::TheoremStatement(theorem) => {
                            LocateAfterRef::LocateAfter(&theorem.label)
                        }
                    },
                },
            };

            statement_and_locate_after = Some((statement, locate_after));
        }
    }

    if let Some((comment_statement, locate_after)) = statement_and_locate_after {
        comment_statement.to_mmp_format(locate_after, mm_data)
    } else {
        Err(Error::NotFoundError)
    }
}
