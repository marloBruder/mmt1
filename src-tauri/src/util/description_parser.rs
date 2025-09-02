use std::collections::{HashMap, HashSet};

use crate::{
    metamath::mm_parser::html_validation::verify_html,
    model::{Header, ParsedDescriptionSegment},
    util,
};

pub fn parse_description(
    description: &str,
    database_header: &Header,
    allowed_tags_and_attributes: &HashMap<String, HashSet<String>>,
    allowed_css_properties: &HashSet<String>,
) -> (Vec<ParsedDescriptionSegment>, Vec<String>) {
    let mut result = Vec::new();

    let mut last_segment: Option<ParsedDescriptionSegment> = None;

    for parsed_comment_char in comment_to_parsed_comment_chars(description) {
        match parsed_comment_char {
            ParsedDescriptionChar::Character(char) => {
                last_segment.as_mut().map(|segement| segement.push(char));
            }
            ParsedDescriptionChar::NormalModeStart => {
                last_segment.map(|segment| result.push(segment));
                last_segment = Some(ParsedDescriptionSegment::Text(String::new()));
            }
            ParsedDescriptionChar::MathModeStart => {
                last_segment.map(|segment| result.push(segment));
                last_segment = Some(ParsedDescriptionSegment::MathMode(String::new()));
            }
            ParsedDescriptionChar::LabelModeStart => {
                last_segment.map(|segment| result.push(segment));
                last_segment = Some(ParsedDescriptionSegment::Label(String::new(), None));
            }
            ParsedDescriptionChar::ItalicStart => {
                last_segment.map(|segment| result.push(segment));
                last_segment = Some(ParsedDescriptionSegment::Italic(String::new()));
            }
            ParsedDescriptionChar::SubscriptStart => {
                last_segment.map(|segment| result.push(segment));
                last_segment = Some(ParsedDescriptionSegment::Subscript(String::new()));
            }
            ParsedDescriptionChar::HtmlModeStart => {
                last_segment.map(|segment| result.push(segment));
                last_segment = Some(ParsedDescriptionSegment::Html(String::new()));
            }
            ParsedDescriptionChar::HtmlCharStart => {
                last_segment.map(|segment| result.push(segment));
                last_segment = Some(ParsedDescriptionSegment::HtmlCharacterRef(String::new()));
            }
        }
    }

    last_segment.map(|segment| result.push(segment));

    let mut invalid_html = Vec::new();

    (
        result
            .into_iter()
            .map(|segment| match segment {
                ParsedDescriptionSegment::Label(label, _) => {
                    if label.starts_with("http:") || label.starts_with("https:") {
                        ParsedDescriptionSegment::Link(label)
                    } else {
                        let theorem_number = database_header
                            .find_theorem_and_index_by_label(&label)
                            .map(|(i, _)| (i + 1) as u32);
                        ParsedDescriptionSegment::Label(label, theorem_number)
                    }
                }
                ParsedDescriptionSegment::Html(html) => {
                    if !verify_html(&html, allowed_tags_and_attributes, allowed_css_properties) {
                        invalid_html.push(html.clone());
                    }
                    ParsedDescriptionSegment::Html(html)
                }
                _ => segment,
            })
            .collect(),
        invalid_html,
    )
}

enum ParsedDescriptionChar {
    Character(char),
    NormalModeStart,
    MathModeStart,
    LabelModeStart,
    ItalicStart,
    SubscriptStart,
    HtmlModeStart,
    HtmlCharStart,
}

fn comment_to_parsed_comment_chars(comment: &str) -> ParsedCommentCharIterator {
    ParsedCommentCharIterator::new(comment)
}

struct ParsedCommentCharIterator<'a> {
    comment: &'a str,
    next_char_i: usize,
    last_start: Option<ParsedDescriptionChar>,
}

impl<'a> ParsedCommentCharIterator<'a> {
    fn new(comment: &str) -> ParsedCommentCharIterator {
        ParsedCommentCharIterator {
            comment,
            next_char_i: 0,
            last_start: None,
        }
    }
}

impl<'a> Iterator for ParsedCommentCharIterator<'a> {
    type Item = ParsedDescriptionChar;

    fn next(&mut self) -> Option<Self::Item> {
        let next_minus_1_char = if self.next_char_i != 0 {
            self.comment
                .as_bytes()
                .get(self.next_char_i - 1)
                .map(|num| char::from(*num))
        } else {
            None
        };
        let next_char = self
            .comment
            .as_bytes()
            .get(self.next_char_i)
            .map(|num| char::from(*num))?;
        let next_plus_1_char = self
            .comment
            .as_bytes()
            .get(self.next_char_i + 1)
            .map(|num| char::from(*num));

        match &self.last_start {
            None => {
                if next_char == '`' {
                    if next_plus_1_char.is_some_and(|c| c == '`') {
                        self.last_start = Some(ParsedDescriptionChar::NormalModeStart);
                        return Some(ParsedDescriptionChar::NormalModeStart);
                    } else {
                        self.next_char_i += 1;
                        self.last_start = Some(ParsedDescriptionChar::MathModeStart);
                        return Some(ParsedDescriptionChar::MathModeStart);
                    }
                } else if next_char == '~' {
                    if next_plus_1_char.is_some_and(|c| c == '~') {
                        self.last_start = Some(ParsedDescriptionChar::NormalModeStart);
                        return Some(ParsedDescriptionChar::NormalModeStart);
                    } else {
                        self.next_char_i =
                            next_non_whitespace_i(self.comment, self.next_char_i + 1);
                        self.last_start = Some(ParsedDescriptionChar::LabelModeStart);
                        return Some(ParsedDescriptionChar::LabelModeStart);
                    }
                } else if next_char == '_' {
                    self.next_char_i += 1;
                    self.last_start = Some(ParsedDescriptionChar::ItalicStart);
                    return Some(ParsedDescriptionChar::ItalicStart);
                } else if self.comment.len() >= (self.next_char_i + 6)
                    && &self.comment[self.next_char_i..(self.next_char_i + 6)] == "<HTML>"
                {
                    self.next_char_i += 6;
                    self.last_start = Some(ParsedDescriptionChar::HtmlModeStart);
                    return Some(ParsedDescriptionChar::HtmlModeStart);
                } else if next_char == '&' {
                    self.next_char_i += 1;
                    self.last_start = Some(ParsedDescriptionChar::HtmlCharStart);
                    return Some(ParsedDescriptionChar::HtmlCharStart);
                } else {
                    self.last_start = Some(ParsedDescriptionChar::NormalModeStart);
                    return Some(ParsedDescriptionChar::NormalModeStart);
                }
            }
            Some(ParsedDescriptionChar::Character(_)) => {
                // Should never be the case
                return None;
            }
            Some(ParsedDescriptionChar::NormalModeStart) => {
                if next_char == '`' {
                    if next_plus_1_char.is_some_and(|c| c == '`') {
                        self.next_char_i += 2;
                        return Some(ParsedDescriptionChar::Character('`'));
                    } else {
                        self.next_char_i += 1;
                        self.last_start = Some(ParsedDescriptionChar::MathModeStart);
                        return Some(ParsedDescriptionChar::MathModeStart);
                    }
                } else if next_char == '~' {
                    if next_plus_1_char.is_some_and(|c| c == '~') {
                        self.next_char_i += 2;
                        return Some(ParsedDescriptionChar::Character('~'));
                    } else {
                        self.next_char_i =
                            next_non_whitespace_i(self.comment, self.next_char_i + 1);
                        self.last_start = Some(ParsedDescriptionChar::LabelModeStart);
                        return Some(ParsedDescriptionChar::LabelModeStart);
                    }
                } else if next_char == '_' {
                    if next_minus_1_char
                        .is_none_or(|c| c.is_ascii_whitespace() || c.is_ascii_punctuation())
                    {
                        self.next_char_i += 1;
                        self.last_start = Some(ParsedDescriptionChar::ItalicStart);
                        return Some(ParsedDescriptionChar::ItalicStart);
                    } else {
                        self.next_char_i += 1;
                        self.last_start = Some(ParsedDescriptionChar::SubscriptStart);
                        return Some(ParsedDescriptionChar::SubscriptStart);
                    }
                } else if self.comment.len() >= (self.next_char_i + 6)
                    && &self.comment[self.next_char_i..(self.next_char_i + 6)] == "<HTML>"
                {
                    self.next_char_i += 6;
                    self.last_start = Some(ParsedDescriptionChar::HtmlModeStart);
                    return Some(ParsedDescriptionChar::HtmlModeStart);
                } else if next_char == '&' {
                    self.next_char_i += 1;
                    self.last_start = Some(ParsedDescriptionChar::HtmlCharStart);
                    return Some(ParsedDescriptionChar::HtmlCharStart);
                } else if next_char.is_ascii_whitespace() {
                    let next_non_whitespace_i =
                        next_non_whitespace_i(self.comment, self.next_char_i);
                    let whitespace_str = &self.comment[self.next_char_i..next_non_whitespace_i];
                    let new_line_count = util::new_lines_in_str(whitespace_str);
                    self.next_char_i = next_non_whitespace_i;
                    if new_line_count <= 1 {
                        return Some(ParsedDescriptionChar::Character(' '));
                    } else {
                        return Some(ParsedDescriptionChar::Character('\n'));
                    }
                } else {
                    self.next_char_i += 1;
                    return Some(ParsedDescriptionChar::Character(next_char));
                }
            }
            Some(ParsedDescriptionChar::MathModeStart) => {
                if next_char == '`' {
                    self.next_char_i += 1;
                    self.last_start = None;
                    return self.next();
                } else if next_char.is_ascii_whitespace() {
                    self.next_char_i = next_non_whitespace_i(self.comment, self.next_char_i);
                    return Some(ParsedDescriptionChar::Character(' '));
                } else {
                    self.next_char_i += 1;
                    return Some(ParsedDescriptionChar::Character(next_char));
                }
            }
            Some(ParsedDescriptionChar::LabelModeStart) => {
                if next_char.is_ascii_whitespace() {
                    self.last_start = Some(ParsedDescriptionChar::NormalModeStart);
                    return Some(ParsedDescriptionChar::NormalModeStart);
                } else if next_char == '~' && next_plus_1_char.is_some_and(|c| c == '~') {
                    self.next_char_i += 2;
                    return Some(ParsedDescriptionChar::Character('~'));
                } else {
                    self.next_char_i += 1;
                    return Some(ParsedDescriptionChar::Character(next_char));
                }
            }
            Some(ParsedDescriptionChar::ItalicStart) => {
                if next_char == '_'
                    && next_plus_1_char
                        .is_none_or(|c| c.is_ascii_whitespace() || c.is_ascii_punctuation())
                {
                    self.next_char_i += 1;
                    self.last_start = None;
                    return self.next();
                } else {
                    self.next_char_i += 1;
                    return Some(ParsedDescriptionChar::Character(next_char));
                }
            }
            Some(ParsedDescriptionChar::SubscriptStart) => {
                if next_char.is_ascii_whitespace() {
                    self.last_start = Some(ParsedDescriptionChar::NormalModeStart);
                    return Some(ParsedDescriptionChar::NormalModeStart);
                } else {
                    self.next_char_i += 1;
                    return Some(ParsedDescriptionChar::Character(next_char));
                }
            }
            Some(ParsedDescriptionChar::HtmlModeStart) => {
                if self.comment.len() >= (self.next_char_i + 7)
                    && &self.comment[self.next_char_i..(self.next_char_i + 7)] == "</HTML>"
                {
                    self.next_char_i += 7;
                    self.last_start = None;
                    return self.next();
                } else {
                    self.next_char_i += 1;
                    return Some(ParsedDescriptionChar::Character(next_char));
                }
            }
            Some(ParsedDescriptionChar::HtmlCharStart) => {
                if next_char == ';' {
                    self.next_char_i += 1;
                    self.last_start = None;
                    return self.next();
                } else {
                    self.next_char_i += 1;
                    return Some(ParsedDescriptionChar::Character(next_char));
                }
            }
        }
    }
}

fn next_non_whitespace_i(str: &str, start: usize) -> usize {
    let mut res = start;
    while str
        .as_bytes()
        .get(res)
        .is_some_and(|b| b.is_ascii_whitespace())
    {
        res += 1;
    }
    res
}
