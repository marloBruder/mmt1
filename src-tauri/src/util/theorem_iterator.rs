use crate::model::{Header, Theorem};

pub struct TheoremIterator<'a> {
    curr_header: &'a Header,
    next_theorem_index: usize,
    next_header_index: usize,
    past: Vec<(&'a Header, usize)>,
}

impl<'a> TheoremIterator<'a> {
    pub fn new(top_header: &'a Header) -> TheoremIterator<'a> {
        TheoremIterator {
            curr_header: &top_header,
            next_theorem_index: 0,
            next_header_index: 0,
            past: Vec::new(),
        }
    }
}

impl<'a> Iterator for TheoremIterator<'a> {
    type Item = &'a Theorem;

    fn next(&mut self) -> Option<&'a Theorem> {
        if self.next_theorem_index < self.curr_header.theorems.len() {
            self.next_theorem_index += 1;
            return self.curr_header.theorems.get(self.next_theorem_index - 1);
        }

        if self.next_header_index < self.curr_header.sub_headers.len() {
            self.past.push((self.curr_header, self.next_header_index));
            self.curr_header = self
                .curr_header
                .sub_headers
                .get(self.next_header_index)
                .unwrap();
            self.next_theorem_index = 0;
            self.next_header_index = 0;
            return self.next();
        }

        if self.past.len() != 0 {
            let (past_header, past_header_index) = self.past.pop().unwrap();
            self.curr_header = past_header;
            self.next_theorem_index = past_header.theorems.len();
            self.next_header_index = past_header_index + 1;
            return self.next();
        }

        None
    }
}
