pub mod earley_parser;
pub mod earley_parser_optimized;
pub mod header_iterators;
pub mod last_curr_next_iterator;

pub fn str_to_space_seperated_string(str: &str) -> String {
    str.split_ascii_whitespace()
        .fold_to_space_seperated_string()
}

pub fn spaces(num: u32) -> &'static str {
    "                                                                                "
        .split_at(num as usize)
        .0
}

pub trait StrIterToSpaceSeperatedString<'a>
where
    Self: Iterator<Item = &'a str>,
{
    fn fold_to_space_seperated_string(&mut self) -> String {
        self.fold((true, String::new()), |(first, mut s), t| {
            if !first {
                s.push(' ');
            }
            s.push_str(t);
            (false, s)
        })
        .1
    }
}

impl<'a, T> StrIterToSpaceSeperatedString<'a> for T where T: Iterator<Item = &'a str> {}
