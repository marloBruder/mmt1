use std::iter::FilterMap;

use crate::model::{
    Constant, DatabaseElement, FloatingHypohesis, Header, Statement::*, Theorem, Variable,
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
    type Item = DatabaseElement<'a>;

    fn next(&mut self) -> Option<DatabaseElement<'a>> {
        loop {
            if self.next_content_index < self.curr_header.content.len() {
                self.next_content_index += 1;
                return Some(DatabaseElement::Statement(
                    self.curr_header
                        .content
                        .get(self.next_content_index - 1)
                        .unwrap(),
                ));
            }

            if self.next_header_index < self.curr_header.subheaders.len() {
                self.past.push((self.curr_header, self.next_header_index));
                self.curr_header = self
                    .curr_header
                    .subheaders
                    .get(self.next_header_index)
                    .unwrap();
                self.next_content_index = 0;
                self.next_header_index = 0;
                return Some(DatabaseElement::Header(
                    self.curr_header,
                    self.past.len() as u32,
                ));
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

pub struct ConstantIterator<'a> {
    inner: FilterMap<HeaderIterator<'a>, Box<dyn FnMut(DatabaseElement) -> Option<&Vec<Constant>>>>,
    curr_const_vec: Option<&'a Vec<Constant>>,
    next_const_i: usize,
}

impl<'a> ConstantIterator<'a> {
    pub fn new(top_header: &'a Header) -> ConstantIterator<'a> {
        ConstantIterator {
            inner: top_header.iter().filter_map(Box::new(|e| {
                if let DatabaseElement::Statement(s) = e {
                    if let ConstantStatement(constants) = s {
                        return Some(constants);
                    }
                }
                None
            })),
            curr_const_vec: None,
            next_const_i: 0,
        }
    }
}

impl<'a> Iterator for ConstantIterator<'a> {
    type Item = &'a Constant;

    fn next(&mut self) -> Option<&'a Constant> {
        if self.curr_const_vec.is_none() {
            self.curr_const_vec = Some(self.inner.next()?);
        }

        if self.next_const_i < self.curr_const_vec.unwrap().len() {
            self.next_const_i += 1;
            return self.curr_const_vec.unwrap().get(self.next_const_i - 1);
        }

        self.next_const_i = 1;
        self.curr_const_vec = Some(self.inner.next()?);
        while self.curr_const_vec.unwrap().is_empty() {
            self.curr_const_vec = Some(self.inner.next()?);
        }
        self.curr_const_vec.unwrap().get(0)
    }
}

pub struct VariableIterator<'a> {
    inner: FilterMap<HeaderIterator<'a>, Box<dyn FnMut(DatabaseElement) -> Option<&Vec<Variable>>>>,
    curr_var_vec: Option<&'a Vec<Variable>>,
    next_var_i: usize,
}

impl<'a> VariableIterator<'a> {
    pub fn new(top_header: &'a Header) -> VariableIterator<'a> {
        VariableIterator {
            inner: top_header.iter().filter_map(Box::new(|e| {
                if let DatabaseElement::Statement(s) = e {
                    if let VariableStatement(variables) = s {
                        return Some(variables);
                    }
                }
                None
            })),
            curr_var_vec: None,
            next_var_i: 0,
        }
    }
}

impl<'a> Iterator for VariableIterator<'a> {
    type Item = &'a Variable;

    fn next(&mut self) -> Option<&'a Variable> {
        if self.curr_var_vec.is_none() {
            self.curr_var_vec = Some(self.inner.next()?);
        }

        if self.next_var_i < self.curr_var_vec.unwrap().len() {
            self.next_var_i += 1;
            return self.curr_var_vec.unwrap().get(self.next_var_i - 1);
        }

        self.next_var_i = 1;
        self.curr_var_vec = Some(self.inner.next()?);
        while self.curr_var_vec.unwrap().is_empty() {
            self.curr_var_vec = Some(self.inner.next()?);
        }
        self.curr_var_vec.unwrap().get(0)
    }
}

pub struct FloatingHypothesisIterator<'a> {
    inner: FilterMap<
        HeaderIterator<'a>,
        Box<dyn FnMut(DatabaseElement) -> Option<&FloatingHypohesis>>,
    >,
}

impl<'a> FloatingHypothesisIterator<'a> {
    pub fn new(top_header: &'a Header) -> FloatingHypothesisIterator<'a> {
        FloatingHypothesisIterator {
            inner: top_header.iter().filter_map(Box::new(|e| {
                if let DatabaseElement::Statement(s) = e {
                    if let FloatingHypohesisStatement(floating_hypothesis) = s {
                        return Some(floating_hypothesis);
                    }
                }
                None
            })),
        }
    }
}

impl<'a> Iterator for FloatingHypothesisIterator<'a> {
    type Item = &'a FloatingHypohesis;

    fn next(&mut self) -> Option<&'a FloatingHypohesis> {
        self.inner.next()
    }
}

pub struct TheoremIterator<'a> {
    inner: FilterMap<HeaderIterator<'a>, Box<dyn FnMut(DatabaseElement) -> Option<&Theorem>>>,
}

impl<'a> TheoremIterator<'a> {
    pub fn new(top_header: &'a Header) -> TheoremIterator<'a> {
        TheoremIterator {
            inner: top_header.iter().filter_map(Box::new(|e| {
                if let DatabaseElement::Statement(s) = e {
                    if let TheoremStatement(theorem) = s {
                        return Some(theorem);
                    }
                }
                None
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
