use crate::{editor::on_edit::DetailedError, Error};

mod stage_1;
mod stage_2;

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
    pub number_of_lines_before_first_statement: u32,
    pub statements: Vec<&'a str>,
}

pub struct MmpParserStage1Fail {
    pub error: DetailedError,
}

impl<'a> MmpParserStage1Success<'a> {
    pub fn next_stage(self) -> Result<MmpParserStage2<'a>, Error> {
        stage_2::stage_2(self)
    }
}

pub enum MmpParserStage2<'a> {
    Success(MmpParserStage2Success<'a>),
    Fail(MmpParserStage2Fail),
}

pub struct MmpParserStage2Success<'a> {
    pub constants: Option<&'a str>,
    pub variables: Vec<&'a str>, // Each str may contain mulitple vars
    pub floating_hypotheses: Vec<&'a str>,
    pub label: Option<MmpLabel<'a>>,
    pub allow_discouraged: bool,
    pub locate_after: Option<LocateAfterRef<'a>>,
    pub distinct_vars: Vec<&'a str>,
    pub proof_lines: Vec<ProofLine<'a>>,
    pub comments: Vec<&'a str>,
    pub statements: Vec<MmpStatement>,
}

pub enum MmpLabel<'a> {
    Header { header_pos: &'a str, title: &'a str },
    Comment(&'a str),
    Axiom(&'a str),
    Theorem(&'a str),
}

#[derive(Debug, Clone, Copy)]
pub enum LocateAfterRef<'a> {
    LocateAfter(&'a str),
    LocateAfterConst(&'a str),
    LocateAfterVar(&'a str),
}

#[derive(Debug)]
pub struct ProofLine<'a> {
    pub advanced_unification: bool,
    pub is_hypothesis: bool,
    pub step_name: &'a str,
    pub hypotheses: &'a str,
    pub hypotheses_parsed: Vec<Option<usize>>, // None if the hypothesis is "?"
    pub step_ref: &'a str,
    pub expression: &'a str,
}

#[derive(Debug)]
pub enum MmpStatement {
    MmpLabel,
    DistinctVar,
    AllowDiscouraged,
    LocateAfter,
    Constant,
    Variable,
    FloatingHypohesis,
    ProofLine,
    Comment,
}

pub struct MmpParserStage2Fail {
    pub errors: Vec<DetailedError>,
}
