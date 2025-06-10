use crate::{editor::on_edit::DetailedError, Error};

mod stage_1;

pub fn new(text: &str) -> MmpParserStage0 {
    MmpParserStage0 { text }
}

pub struct MmpParserStage0<'a> {
    pub text: &'a str,
}

impl<'a> MmpParserStage0<'a> {
    pub fn next_stage(self) -> Result<MmpParserStage1<'a>, Error> {
        stage_1::stage_1(self)
    }
}

pub enum MmpParserStage1<'a> {
    Success(MmpParserStage1Success<'a>),
    Fail(MmpParserStage1Fail),
}

pub struct MmpParserStage1Success<'a> {
    pub number_of_lines_before_first: u32,
    pub lines: Vec<&'a str>,
}

pub struct MmpParserStage1Fail {
    pub error: DetailedError,
}
