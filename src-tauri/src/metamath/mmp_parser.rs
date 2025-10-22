use std::{collections::HashSet, sync::Arc};

use crate::{
    editor::on_edit::DetailedError,
    model::{Comment, Constant, FloatingHypothesis, HeaderPath, MetamathData, ParseTree, Variable},
    Error, Settings,
};

pub mod calc_indention;
mod stage_1;
mod stage_2;
mod stage_3;
mod stage_4;
mod stage_5;
pub mod stage_6;

pub fn new<'a>(text: &'a str) -> MmpParserStage0<'a> {
    MmpParserStage0 { text }
}

pub struct MmpParserStage0<'a> {
    pub text: &'a str,
}

impl<'a> MmpParserStage0<'a> {
    pub fn next_stage(&self) -> Result<MmpParserStage1<'a>, Error> {
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
    pub fn next_stage(&self) -> Result<MmpParserStage2<'a>, Error> {
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
    pub allow_incomplete: bool,
    pub locate_after: Option<LocateAfterRef<'a>>,
    pub distinct_vars: Vec<&'a str>,
    pub proof_lines: Vec<ProofLine<'a>>,
    pub comments: Vec<&'a str>,
    pub statements: Vec<(MmpStatement, u32)>,
}

pub enum MmpLabel<'a> {
    Header {
        header_path: &'a str,
        title: &'a str,
    },
    Axiom(&'a str),
    Theorem(&'a str),
}

#[derive(Debug, Clone, Copy)]
pub enum LocateAfterRef<'a> {
    LocateAfterStart,
    LocateAfterHeader(&'a str),
    LocateAfterComment(&'a str),
    LocateAfterConst(&'a str),
    LocateAfterVar(&'a str),
    LocateAfter(&'a str),
}

#[derive(Debug)]
pub struct ProofLine<'a> {
    pub advanced_unification: bool,
    pub is_hypothesis: bool,
    pub step_name: &'a str,
    pub hypotheses: &'a str,
    pub step_ref: &'a str,
    pub expression: &'a str,
}

#[derive(Debug, PartialEq, Eq)]
pub enum MmpStatement {
    MmpLabel,
    DistinctVar,
    AllowDiscouraged,
    AllowIncomplete,
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

impl<'a> MmpParserStage2Success<'a> {
    pub fn next_stage(
        &self,
        stage_1: &MmpParserStage1Success<'a>,
        mm_data: &MetamathData,
    ) -> Result<MmpParserStage3<'a>, Error> {
        stage_3::stage_3(stage_1, self, mm_data)
    }
}

pub enum MmpParserStage3<'a> {
    Success(MmpParserStage3Success<'a>),
    Fail(MmpParserStage3Fail),
}

pub enum MmpParserStage3Success<'a> {
    Header(MmpParserStage3Header),
    Comment(MmpParserStage3Comment),
    Constants(Vec<Constant>),
    Variables(Vec<Variable>),
    FloatingHypohesis(FloatingHypothesis),
    Theorem(MmpParserStage3Theorem<'a>),
    Empty,
}

pub struct MmpParserStage3Header {
    pub parent_header_path: HeaderPath,
    pub header_i: usize,
    pub title: String,
    pub description: String,
}

pub struct MmpParserStage3Comment {
    pub parent_header_path: HeaderPath,
    pub comment_i: usize,
    pub comment: Comment,
}

pub struct MmpParserStage3Theorem<'a> {
    pub is_axiom: bool,
    // pub allow_discouraged: bool,
    pub label: &'a str,
    pub temp_variable_statements: Vec<Vec<Variable>>,
    pub temp_floating_hypotheses: Vec<FloatingHypothesis>,
    // pub distinct_vars: Vec<&'a str>,
    // pub proof_lines: Vec<ProofLineParsed<'a>>,
    // pub locate_after: Option<LocateAfterRef<'a>>,
    // pub description: &'a str,
    // pub indention: Vec<u32>,
    pub axiom_dependencies: Vec<(String, u32)>,
    pub definition_dependencies: Vec<(String, u32)>,
}

pub struct MmpParserStage3Fail {
    pub errors: Vec<DetailedError>,
}

impl<'a> MmpParserStage3Theorem<'a> {
    pub fn next_stage(
        &self,
        stage_1: &MmpParserStage1Success,
        stage_2: &MmpParserStage2Success,
        mm_data: &MetamathData,
    ) -> Result<MmpParserStage4, Error> {
        stage_4::stage_4(stage_1, stage_2, self, mm_data)
    }
}

pub enum MmpParserStage4 {
    Success(MmpParserStage4Success),
    Fail(MmpParserStage4Fail),
}

pub struct MmpParserStage4Success {
    pub distinct_variable_pairs: HashSet<(String, String)>,
    pub proof_lines_parsed: Vec<ProofLineParsed>,
    pub reference_numbers: Vec<Option<u32>>,
    pub proof_line_statuses: Vec<ProofLineStatus>,
}

#[derive(Debug)]
pub struct ProofLineParsed {
    pub hypotheses_parsed: Vec<Option<usize>>, // None if the hypothesis is "?"
    pub parse_tree: Option<ParseTree>,
}

#[derive(Debug, Clone, Copy)]
pub enum ProofLineStatus {
    None,
    Err((bool, bool, bool, bool)),
    Correct,
    CorrectRecursively,
    Unified((bool, bool, bool, bool), bool),
}

pub struct MmpParserStage4Fail {
    pub errors: Vec<DetailedError>,
    pub reference_numbers: Vec<Option<u32>>,
    pub proof_line_statuses: Vec<ProofLineStatus>,
}

impl MmpParserStage4Success {
    pub fn next_stage(
        &self,
        stage_2: &MmpParserStage2Success,
        stage_3: &MmpParserStage3Theorem,
        mm_data: &MetamathData,
        stop: Option<Arc<std::sync::Mutex<bool>>>,
    ) -> Result<MmpParserStage5, Error> {
        stage_5::stage_5(stage_2, stage_3, self, mm_data, stop)
    }
}

#[derive(Debug)]
pub struct MmpParserStage5 {
    pub unify_result: Vec<UnifyLine>,
    pub unify_reference_numbers: Vec<Option<u32>>,
}

#[derive(Debug)]
pub struct UnifyLine {
    pub new_line: bool,
    pub deleted_line: bool,
    pub advanced_unification: bool,
    pub is_hypothesis: bool,
    pub step_name: String,
    pub hypotheses: Vec<String>,
    pub step_ref: String,
    pub parse_tree: Option<ParseTree>,
    pub old_assertion: Option<String>,
    pub status: ProofLineStatus,
}

impl MmpParserStage5 {
    pub fn next_stage(
        &self,
        stage_3: &MmpParserStage3Theorem,
        stage_4: &MmpParserStage4Success,
        mm_data: &MetamathData,
        settings: &Settings,
    ) -> Result<MmpParserStage6, Error> {
        stage_6::stage_6(stage_3, stage_4, self, mm_data, settings)
    }
}

pub struct MmpParserStage6 {
    pub proof: Option<String>,
}
