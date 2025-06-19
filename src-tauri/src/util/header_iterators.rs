use std::iter::FilterMap;

use crate::{
    metamath::mmp_parser::LocateAfterRef,
    model::{
        Constant, DatabaseElement, FloatingHypothesis, Header,
        Statement::{self, *},
        Theorem, Variable,
    },
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

        self.next_const_i = 0;
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

        self.next_var_i = 0;
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
        Box<dyn FnMut(DatabaseElement) -> Option<&FloatingHypothesis>>,
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
    type Item = &'a FloatingHypothesis;

    fn next(&mut self) -> Option<&'a FloatingHypothesis> {
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

pub struct HeaderLocateAfterIterator<'a, 'b> {
    inner: HeaderIterator<'a>,
    locate_after: Option<LocateAfterRef<'b>>,
    found: bool,
}

impl<'a, 'b> HeaderLocateAfterIterator<'a, 'b> {
    pub fn new(
        top_header: &'a Header,
        locate_after: Option<LocateAfterRef<'b>>,
    ) -> HeaderLocateAfterIterator<'a, 'b> {
        HeaderLocateAfterIterator {
            inner: top_header.iter(),
            locate_after,
            found: false,
        }
    }
}

impl<'a, 'b> Iterator for HeaderLocateAfterIterator<'a, 'b> {
    type Item = DatabaseElement<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.found {
            return None;
        }

        let next_element = self.inner.next()?;

        match (&next_element, &self.locate_after) {
            (DatabaseElement::Header(_header, _), Some(_la)) if false => {}
            (DatabaseElement::Statement(statement), Some(la)) => match (statement, la) {
                (Statement::CommentStatement(_comment), _) if false => {}
                (
                    Statement::ConstantStatement(constants),
                    LocateAfterRef::LocateAfterConst(la_const),
                ) => {
                    if constants.iter().any(|c| c.symbol == *la_const) {
                        self.found = true;
                    }
                }
                (
                    Statement::VariableStatement(variables),
                    LocateAfterRef::LocateAfterVar(la_var),
                ) => {
                    if variables.iter().any(|v| v.symbol == *la_var) {
                        self.found = true;
                    }
                }
                (
                    Statement::FloatingHypohesisStatement(floating_hypothesis),
                    LocateAfterRef::LocateAfter(la_label),
                ) => {
                    if floating_hypothesis.label == *la_label {
                        self.found = true;
                    }
                }
                (Statement::TheoremStatement(theorem), LocateAfterRef::LocateAfter(la_label)) => {
                    if theorem.label == *la_label {
                        self.found = true;
                    }
                }
                (_, _) => {}
            },
            (_, _) => {}
        }

        Some(next_element)
    }
}

pub struct TheoremLocateAfterIterator<'a, 'b> {
    inner: FilterMap<
        HeaderLocateAfterIterator<'a, 'b>,
        Box<dyn FnMut(DatabaseElement) -> Option<&Theorem>>,
    >,
}

impl<'a, 'b> TheoremLocateAfterIterator<'a, 'b> {
    pub fn new(
        top_header: &'a Header,
        locate_after: Option<LocateAfterRef<'b>>,
    ) -> TheoremLocateAfterIterator<'a, 'b> {
        TheoremLocateAfterIterator {
            inner: top_header
                .locate_after_iter(locate_after)
                .filter_map(Box::new(|e| {
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

impl<'a, 'b> Iterator for TheoremLocateAfterIterator<'a, 'b> {
    type Item = &'a Theorem;

    fn next(&mut self) -> Option<&'a Theorem> {
        self.inner.next()
    }
}

pub struct MathSymbolLocateAfterIterator<'a, 'b> {
    inner: FilterMap<
        HeaderLocateAfterIterator<'a, 'b>,
        Box<dyn FnMut(DatabaseElement) -> Option<Vec<&str>>>,
    >,
    curr_math_symbol_vec: Option<Vec<&'a str>>,
    next_symbol_i: usize,
}

impl<'a, 'b> MathSymbolLocateAfterIterator<'a, 'b> {
    pub fn new(
        top_header: &'a Header,
        locate_after: Option<LocateAfterRef<'b>>,
    ) -> MathSymbolLocateAfterIterator<'a, 'b> {
        MathSymbolLocateAfterIterator {
            inner: top_header
                .locate_after_iter(locate_after)
                .filter_map(Box::new(|e| {
                    if let DatabaseElement::Statement(s) = e {
                        if let ConstantStatement(constants) = s {
                            return Some(constants.iter().map(|c| &*c.symbol).collect());
                        } else if let VariableStatement(variables) = s {
                            return Some(variables.iter().map(|v| &*v.symbol).collect());
                        }
                    }
                    None
                })),
            curr_math_symbol_vec: None,
            next_symbol_i: 0,
        }
    }
}

impl<'a, 'b> Iterator for MathSymbolLocateAfterIterator<'a, 'b> {
    type Item = &'a str;

    fn next(&mut self) -> Option<&'a str> {
        if self.curr_math_symbol_vec.is_none() {
            self.curr_math_symbol_vec = Some(self.inner.next()?);
        }

        if self.next_symbol_i < self.curr_math_symbol_vec.as_ref().unwrap().len() {
            self.next_symbol_i += 1;
            return self
                .curr_math_symbol_vec
                .as_ref()
                .unwrap()
                .get(self.next_symbol_i - 1)
                .map(|s| *s);
        }

        self.next_symbol_i = 0;
        self.curr_math_symbol_vec = Some(self.inner.next()?);
        while self.curr_math_symbol_vec.as_ref().unwrap().is_empty() {
            self.curr_math_symbol_vec = Some(self.inner.next()?);
        }
        self.curr_math_symbol_vec
            .as_ref()
            .unwrap()
            .get(0)
            .map(|s| *s)
    }
}
