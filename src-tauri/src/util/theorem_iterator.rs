use std::iter::FilterMap;

use crate::model::{
    Constant, Header,
    Statement::{self, *},
    Theorem, Variable,
};

pub struct HeaderIterator<'a> {
    curr_header: &'a Header,
    next_content_index: usize,
    next_header_index: usize,
    past: Vec<(&'a Header, usize)>,
}

impl<'a> HeaderIterator<'a> {
    pub fn new(top_header: &'a Header) -> HeaderIterator<'a> {
        HeaderIterator {
            curr_header: &top_header,
            next_content_index: 0,
            next_header_index: 0,
            past: Vec::new(),
        }
    }
}

impl<'a> Iterator for HeaderIterator<'a> {
    type Item = &'a Statement;

    fn next(&mut self) -> Option<&'a Statement> {
        loop {
            if self.next_content_index < self.curr_header.content.len() {
                self.next_content_index += 1;
                return self.curr_header.content.get(self.next_content_index - 1);
            }

            if self.next_header_index < self.curr_header.sub_headers.len() {
                self.past.push((self.curr_header, self.next_header_index));
                self.curr_header = self
                    .curr_header
                    .sub_headers
                    .get(self.next_header_index)
                    .unwrap();
                self.next_content_index = 0;
                self.next_header_index = 0;
                continue;
            }

            if self.past.len() != 0 {
                let (past_header, past_header_index) = self.past.pop().unwrap();
                self.curr_header = past_header;
                self.next_content_index = past_header.content.len();
                self.next_header_index = past_header_index + 1;
                continue;
            }

            break None;
        }
    }
}

// struct TheoremFilterMap;

// impl FnOnce<(&Statement)> for TheoremFilterMap {
//     type Output = Option<&Theorem>;

//     extern "rust-call" fn call_once(self, s: (&Statement)) -> Option<&Theorem> {
//         if let TheoremStatement(theorem) = s {
//             Some(theorem)
//         } else {
//             None
//         }
//     }
// }

pub struct ConstantIterator<'a> {
    inner: FilterMap<HeaderIterator<'a>, Box<dyn FnMut(&Statement) -> Option<&Constant>>>,
}

impl<'a> ConstantIterator<'a> {
    pub fn new(top_header: &'a Header) -> ConstantIterator<'a> {
        ConstantIterator {
            inner: top_header.iter().filter_map(Box::new(|s| {
                if let ConstantStatement(constant) = s {
                    Some(constant)
                } else {
                    None
                }
            })),
        }
    }
}

impl<'a> Iterator for ConstantIterator<'a> {
    type Item = &'a Constant;

    fn next(&mut self) -> Option<&'a Constant> {
        self.inner.next()
    }
}

pub struct VariableIterator<'a> {
    inner: FilterMap<HeaderIterator<'a>, Box<dyn FnMut(&Statement) -> Option<&Variable>>>,
}

impl<'a> VariableIterator<'a> {
    pub fn new(top_header: &'a Header) -> VariableIterator<'a> {
        VariableIterator {
            inner: top_header.iter().filter_map(Box::new(|s| {
                if let VariableStatement(variable) = s {
                    Some(variable)
                } else {
                    None
                }
            })),
        }
    }
}

impl<'a> Iterator for VariableIterator<'a> {
    type Item = &'a Variable;

    fn next(&mut self) -> Option<&'a Variable> {
        self.inner.next()
    }
}

pub struct TheoremIterator<'a> {
    inner: FilterMap<HeaderIterator<'a>, Box<dyn FnMut(&Statement) -> Option<&Theorem>>>,
}

impl<'a> TheoremIterator<'a> {
    pub fn new(top_header: &'a Header) -> TheoremIterator<'a> {
        TheoremIterator {
            inner: top_header.iter().filter_map(Box::new(|s| {
                if let TheoremStatement(theorem) = s {
                    Some(theorem)
                } else {
                    None
                }
            })),
        }
    }
}

impl<'a> Iterator for TheoremIterator<'a> {
    type Item = &'a Theorem;

    fn next(&mut self) -> Option<&'a Theorem> {
        self.inner.next()
    }
}
