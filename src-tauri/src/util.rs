pub mod earley_parser;
pub mod earley_parser_optimized;
pub mod header_iterators;
pub mod last_curr_next_iterator;

pub fn str_to_space_seperated_string(str: &str) -> String {
    return str
        .split_ascii_whitespace()
        .fold((true, String::new()), |(first, mut s), t| {
            if !first {
                s.push(' ');
            }
            s.push_str(t);
            (false, s)
        })
        .1;
}
