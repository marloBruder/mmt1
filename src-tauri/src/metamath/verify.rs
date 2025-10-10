use std::collections::{HashMap, HashSet};

use crate::{
    metamath::mmp_parser::LocateAfterRef,
    model::{self, FloatingHypothesis, MetamathData, Theorem},
    util::StrIterToSpaceSeperatedString,
    Error,
};

pub struct Verifier<'a> {
    proof_steps: Vec<ProofStep<'a>>,
    step_numbers: Vec<ProofNumber>,
    next_step_number_i: usize,
    stack: Vec<StackLine>,
    theorem_distinct_var_conditions: &'a HashSet<(String, String)>,
    proof_lines_returned: usize,
    previous_proof_line_mapping: HashMap<String, usize>,
    show_all: bool,
    only_show_last: bool,
}

#[derive(Debug, Clone)]
pub struct ProofStep<'a> {
    pub label: Result<&'a str, String>,
    pub label_theorem_number: Option<u32>,
    pub hypotheses: Vec<ProofStepHypothesis<'a>>,
    pub statement: Result<&'a str, String>,
    pub distinct_var_conditions: Option<&'a HashSet<(String, String)>>,
}

#[derive(Debug, Clone)]
pub struct ProofStepHypothesis<'a> {
    pub statement: Result<&'a str, String>,
    pub is_floating_hypothesis: bool,
}

struct ProofNumber {
    number: u32,
    save: bool,
}

#[derive(Debug)]
struct StackLine {
    pub statement: String,
    pub display_step_number: Option<usize>,
}

pub enum VerifierCreationResult<'a> {
    Verifier(Verifier<'a>),
    IsAxiom,
    IsIncomplete,
}

pub enum StepResult {
    VerifierFinished,
    NoProofLine,
    ProofLine(model::ProofLine),
}

pub enum VerificationResult {
    Correct,
    Incorrect,
    Incomplete,
}

impl<'a> Verifier<'a> {
    pub fn new(
        theorem: &'a Theorem,
        metamath_data: &'a MetamathData,
        show_all: bool,
        only_show_last: bool,
        already_calculated_proof_steps: Option<&'a HashMap<&'a str, ProofStep<'a>>>,
        prev_flaoting_hypotheses: Option<&'a Vec<FloatingHypothesis>>,
        compressed_infered_proof_steps: Option<Vec<ProofStep<'a>>>,
    ) -> Result<VerifierCreationResult<'a>, Error> {
        let opt_proof_steps_and_numbers = if let Some(proof) = theorem.proof.as_ref() {
            if proof.starts_with("( ") {
                Verifier::calc_proof_steps_and_numbers_compressed(
                    theorem,
                    metamath_data,
                    already_calculated_proof_steps,
                    prev_flaoting_hypotheses,
                    compressed_infered_proof_steps,
                )?
            } else {
                Verifier::calc_proof_steps_and_numbers_uncompressed(
                    theorem,
                    metamath_data,
                    already_calculated_proof_steps,
                    prev_flaoting_hypotheses,
                )?
            }
        } else {
            return Ok(VerifierCreationResult::IsAxiom);
        };

        let Some((proof_steps, step_numbers)) = opt_proof_steps_and_numbers else {
            return Ok(VerifierCreationResult::IsIncomplete);
        };

        let theorem_distinct_var_conditions = &metamath_data
            .optimized_data
            .theorem_data
            .get(&theorem.label)
            .ok_or(Error::InternalLogicError)?
            .distinct_variable_pairs;

        Ok(VerifierCreationResult::Verifier(Verifier {
            proof_steps,
            step_numbers,
            next_step_number_i: 0,
            stack: Vec::new(),
            theorem_distinct_var_conditions,
            proof_lines_returned: 0,
            previous_proof_line_mapping: HashMap::new(),
            show_all,
            only_show_last,
        }))
    }

    fn calc_proof_steps_and_numbers_compressed<'b>(
        theorem: &'b Theorem,
        metamath_data: &'b MetamathData,
        already_calculated_proof_steps: Option<&'b HashMap<&str, ProofStep>>,
        prev_flaoting_hypotheses: Option<&'b Vec<FloatingHypothesis>>,
        compressed_infered_proof_steps: Option<Vec<ProofStep<'b>>>,
    ) -> Result<Option<(Vec<ProofStep<'b>>, Vec<ProofNumber>)>, Error> {
        let Some(proof) = theorem.proof.as_ref() else {
            return Err(Error::InternalLogicError);
        };

        let mut steps = match compressed_infered_proof_steps {
            None => {
                let mut steps = Vec::new();

                let hypotheses = Verifier::calc_all_hypotheses_of_theorem(
                    theorem,
                    metamath_data,
                    prev_flaoting_hypotheses,
                )?;

                for (hypothesis, label) in hypotheses {
                    steps.push(ProofStep {
                        label,
                        label_theorem_number: None,
                        hypotheses: Vec::new(),
                        statement: hypothesis.statement,
                        distinct_var_conditions: None,
                    })
                }

                steps
            }
            Some(steps) => steps,
        };

        let mut incomplete = false;

        let mut token_iter = proof.split_whitespace().skip(1);

        while let Some(label) = token_iter.next() {
            if label == ")" {
                break;
            }

            if label == "?" {
                incomplete = true;
                continue;
            }

            if let Some(proof_steps) = already_calculated_proof_steps {
                if let Some(proof_step) = proof_steps.get(label) {
                    steps.push(proof_step.clone());
                } else {
                    let floating_hypothesis = theorem
                        .temp_floating_hypotheses
                        .iter()
                        .find(|fh| fh.label == label)
                        .ok_or(Error::InvalidProofError)?;

                    steps.push(ProofStep {
                        label: Ok(&floating_hypothesis.label),
                        label_theorem_number: None,
                        hypotheses: Vec::new(),
                        statement: Err(floating_hypothesis.to_assertions_string()),
                        distinct_var_conditions: None,
                    });
                }
            } else {
                if label == theorem.label {
                    return Err(Error::InvalidProofError);
                }

                if let Some((step_theorem_i, step_theorem)) = metamath_data
                    .database_header
                    .theorem_locate_after_iter(Some(LocateAfterRef::LocateAfter(&theorem.label)))
                    .enumerate()
                    .find(|(_, t)| t.label == label)
                {
                    let theorem_hypotheses = Verifier::calc_all_hypotheses_of_theorem(
                        step_theorem,
                        metamath_data,
                        prev_flaoting_hypotheses,
                    )?;

                    let theorem_data = metamath_data
                        .optimized_data
                        .theorem_data
                        .get(&step_theorem.label)
                        .ok_or(Error::InternalLogicError)?;

                    steps.push(ProofStep {
                        label: Ok(&step_theorem.label),
                        label_theorem_number: Some((step_theorem_i + 1) as u32),
                        hypotheses: theorem_hypotheses
                            .into_iter()
                            .map(|(hyp, _label)| hyp)
                            .collect(),
                        statement: Ok(&step_theorem.assertion),
                        distinct_var_conditions: Some(&theorem_data.distinct_variable_pairs),
                    });
                } else {
                    let floating_hypothesis = match prev_flaoting_hypotheses {
                        None => metamath_data
                            .database_header
                            .floating_hypohesis_locate_after_iter(Some(
                                LocateAfterRef::LocateAfter(&theorem.label),
                            ))
                            .chain(theorem.temp_floating_hypotheses.iter())
                            .find(|fh| fh.label == label)
                            .ok_or(Error::InvalidProofError)?,
                        Some(floating_hypotheses) => floating_hypotheses
                            .iter()
                            .chain(theorem.temp_floating_hypotheses.iter())
                            .find(|fh| fh.label == label)
                            .ok_or(Error::InvalidProofError)?,
                    };

                    steps.push(ProofStep {
                        label: Ok(&floating_hypothesis.label),
                        label_theorem_number: None,
                        hypotheses: Vec::new(),
                        statement: Err(floating_hypothesis.to_assertions_string()),
                        distinct_var_conditions: None,
                    });
                }
            }
        }

        let mut step_numbers: Vec<ProofNumber> = Vec::new();

        let mut char_iter = token_iter.map(|str| str.chars()).flatten();

        let mut current_compressed_num = String::new();

        while let Some(character) = char_iter.next() {
            match character {
                c @ 'A'..='T' => {
                    current_compressed_num.push(c);
                    step_numbers.push(ProofNumber {
                        number: Verifier::compressed_num_to_num(&current_compressed_num)?,
                        save: false,
                    });
                    current_compressed_num = String::new();
                }
                c @ 'U'..='Y' => current_compressed_num.push(c),
                'Z' => {
                    step_numbers
                        .last_mut()
                        .ok_or(Error::InvalidProofError)?
                        .save = true;
                }
                _ => return Err(Error::InvalidProofError),
            }
        }

        Ok(if !incomplete {
            Some((steps, step_numbers))
        } else {
            None
        })
    }

    pub fn calc_all_hypotheses_of_theorem<'b>(
        theorem: &'b Theorem,
        metamath_data: &'b MetamathData,
        prev_flaoting_hypotheses: Option<&Vec<FloatingHypothesis>>,
    ) -> Result<Vec<(ProofStepHypothesis<'b>, Result<&'b str, String>)>, Error> {
        let mut hypotheses: Vec<(ProofStepHypothesis, Result<&str, String>)> = Vec::new();

        // Calculate variables occuring in assertion and hypotheses
        let variables = Verifier::calc_variables_of_theorem(theorem, metamath_data);

        // Calculate proof steps of floating hypotheses
        match prev_flaoting_hypotheses {
            None => {
                for floating_hypothesis in metamath_data
                    .database_header
                    .floating_hypohesis_locate_after_iter(Some(LocateAfterRef::LocateAfter(
                        &theorem.label,
                    )))
                    .chain(theorem.temp_floating_hypotheses.iter())
                {
                    if variables.contains(&*floating_hypothesis.variable) {
                        hypotheses.push((
                            ProofStepHypothesis {
                                statement: Err(floating_hypothesis.to_assertions_string()),
                                is_floating_hypothesis: true,
                            },
                            Ok(&floating_hypothesis.label),
                        ));
                    }
                }
            }
            Some(floating_hyptheses) => {
                for floating_hypothesis in floating_hyptheses
                    .iter()
                    .chain(theorem.temp_floating_hypotheses.iter())
                {
                    if variables.contains(&*floating_hypothesis.variable) {
                        hypotheses.push((
                            ProofStepHypothesis {
                                statement: Err(floating_hypothesis.to_assertions_string()),
                                is_floating_hypothesis: true,
                            },
                            Err(floating_hypothesis.label.clone()),
                        ));
                    }
                }
            }
        }

        if hypotheses.len() != variables.len() {
            return Err(Error::InvalidProofError);
        }

        // Calculate proof steps of essential hypotheses
        for hypothesis in &theorem.hypotheses {
            hypotheses.push((
                ProofStepHypothesis {
                    statement: Ok(&hypothesis.expression),
                    is_floating_hypothesis: false,
                },
                Ok(&hypothesis.label),
            ));
        }

        Ok(hypotheses)
    }

    fn calc_variables_of_theorem<'b>(
        theorem: &'b Theorem,
        metamath_data: &MetamathData,
    ) -> HashSet<&'b str> {
        let mut variables = HashSet::new();
        let theorem_variables: HashSet<&str> = theorem
            .temp_variables
            .iter()
            .flatten()
            .map(|v| &*v.symbol)
            .collect();

        Verifier::add_variables_to_hashset_from_statement(
            &mut variables,
            &theorem.assertion,
            metamath_data,
            &theorem_variables,
        );

        for hypothesis in &theorem.hypotheses {
            Verifier::add_variables_to_hashset_from_statement(
                &mut variables,
                &hypothesis.expression,
                metamath_data,
                &theorem_variables,
            );
        }

        variables
    }

    fn add_variables_to_hashset_from_statement<'b>(
        hashset: &mut HashSet<&'b str>,
        statement: &'b str,
        metamath_data: &MetamathData,
        theorem_variables: &HashSet<&str>,
    ) {
        for token in statement.split_whitespace() {
            if metamath_data.is_variable(token) || theorem_variables.contains(token) {
                hashset.insert(token);
            }
        }
    }

    fn compressed_num_to_num(compressed_num: &str) -> Result<u32, Error> {
        let mut first = true;
        let mut num = 0;
        let mut multiplier = 20;
        for ch in compressed_num.chars().rev() {
            match ch {
                ch @ 'A'..='T' if first => {
                    num = (ch as u32) - 64;
                    first = false;
                }
                ch @ 'U'..='Y' if !first => {
                    num += ((ch as u32) - 84) * multiplier;
                    multiplier *= 5;
                }
                _ => return Err(Error::InvalidProofError),
            }
        }
        if num == 0 {
            return Err(Error::InvalidProofError);
        }
        Ok(num)
    }

    fn calc_proof_steps_and_numbers_uncompressed<'b>(
        theorem: &'b Theorem,
        metamath_data: &'b MetamathData,
        already_calculated_proof_steps: Option<&'b HashMap<&str, ProofStep>>,
        prev_flaoting_hypotheses: Option<&'b Vec<FloatingHypothesis>>,
    ) -> Result<Option<(Vec<ProofStep<'b>>, Vec<ProofNumber>)>, Error> {
        let Some(proof) = theorem.proof.as_ref() else {
            return Err(Error::InternalLogicError);
        };

        let mut proof_steps: Vec<ProofStep> = Vec::new();
        let mut proof_step_numbers: Vec<ProofNumber> = Vec::new();

        let mut incomplete = false;

        for token in proof.split_ascii_whitespace() {
            if token == "?" {
                incomplete = true;
                continue;
            }

            if let Some((i, _)) = proof_steps
                .iter()
                .enumerate()
                .find(|(_, ps)| ps.label.as_ref().map(|s| *s).unwrap_or_else(|s| s) == token)
            {
                proof_step_numbers.push(ProofNumber {
                    number: (i + 1) as u32,
                    save: false,
                });
            } else {
                if let Some(computed_proof_steps) = already_calculated_proof_steps {
                    if let Some(proof_step) = computed_proof_steps.get(token) {
                        proof_steps.push(proof_step.clone());
                        continue;
                    }
                }

                proof_steps.push(Verifier::calc_proof_step_from_label(
                    token,
                    theorem,
                    metamath_data,
                    prev_flaoting_hypotheses,
                )?);
                proof_step_numbers.push(ProofNumber {
                    number: proof_steps.len() as u32,
                    save: false,
                })
            }
        }

        Ok(if !incomplete {
            Some((proof_steps, proof_step_numbers))
        } else {
            None
        })
    }

    fn calc_proof_step_from_label<'b>(
        label: &str,
        theorem: &'b Theorem,
        metamath_data: &'b MetamathData,
        prev_flaoting_hypotheses: Option<&'b Vec<FloatingHypothesis>>,
    ) -> Result<ProofStep<'b>, Error> {
        if let Some(hyp) = theorem.hypotheses.iter().find(|h| h.label == label) {
            return Ok(ProofStep {
                label: Ok(&hyp.label),
                label_theorem_number: None,
                hypotheses: Vec::new(),
                statement: Ok(&hyp.expression),
                distinct_var_conditions: None,
            });
        }

        match prev_flaoting_hypotheses {
            None => {
                if let Some(floating_hypothesis) = metamath_data
                    .database_header
                    .floating_hypohesis_locate_after_iter(Some(LocateAfterRef::LocateAfter(
                        &theorem.label,
                    )))
                    .chain(theorem.temp_floating_hypotheses.iter())
                    .find(|fh| fh.label == label)
                {
                    return Ok(ProofStep {
                        label: Ok(&floating_hypothesis.label),
                        label_theorem_number: None,
                        hypotheses: Vec::new(),
                        statement: Err(floating_hypothesis.to_assertions_string()),
                        distinct_var_conditions: None,
                    });
                }
            }
            Some(floating_hypotheses) => {
                if let Some(floating_hypothesis) = floating_hypotheses
                    .iter()
                    .chain(theorem.temp_floating_hypotheses.iter())
                    .find(|fh| fh.label == label)
                {
                    return Ok(ProofStep {
                        label: Ok(&floating_hypothesis.label),
                        label_theorem_number: None,
                        hypotheses: Vec::new(),
                        statement: Err(floating_hypothesis.to_assertions_string()),
                        distinct_var_conditions: None,
                    });
                }
            }
        }

        if theorem.label == label {
            return Err(Error::InvalidProofError);
        }

        if let Some((theorem_i, theorem)) = metamath_data
            .database_header
            .theorem_locate_after_iter(Some(LocateAfterRef::LocateAfter(&theorem.label)))
            .enumerate()
            .find(|(_, t)| t.label == label)
        {
            let label_theorem_hypotheses = Verifier::calc_all_hypotheses_of_theorem(
                theorem,
                metamath_data,
                prev_flaoting_hypotheses,
            )?;

            let theorem_data = metamath_data
                .optimized_data
                .theorem_data
                .get(&theorem.label)
                .ok_or(Error::InternalLogicError)?;

            return Ok(ProofStep {
                label: Ok(&theorem.label),
                label_theorem_number: Some((theorem_i + 1) as u32),
                hypotheses: label_theorem_hypotheses
                    .into_iter()
                    .map(|(hyp, _label)| hyp)
                    .collect(),
                statement: Ok(&theorem.assertion),
                distinct_var_conditions: Some(&theorem_data.distinct_variable_pairs),
            });
        }

        Err(Error::InvalidProofError)
    }

    pub fn proccess_next_step(
        &mut self,
        metamath_data: &MetamathData,
    ) -> Result<StepResult, Error> {
        let Some(next_step_number) = self.step_numbers.get(self.next_step_number_i) else {
            return Ok(StepResult::VerifierFinished);
        };
        self.next_step_number_i += 1;

        let step = self
            .proof_steps
            .get((next_step_number.number - 1) as usize)
            .ok_or(Error::InvalidProofError)?;
        let mut hypotheses_nums: Vec<usize> = Vec::new();

        let next_stack_statement = if step.hypotheses.len() == 0 {
            step.statement
                .clone()
                .map(|s| s.to_string())
                .unwrap_or_else(|s| s)
        } else {
            let (next_step, new_hypotheses_nums) = Verifier::calc_step_application(
                step,
                &self.stack,
                self.theorem_distinct_var_conditions,
                metamath_data,
            )?;
            hypotheses_nums = new_hypotheses_nums;

            for _ in 0..step.hypotheses.len() {
                self.stack.pop();
            }

            next_step
        };

        let mut proof_line: Option<model::ProofLine> = None;

        let mut display_step_number: Option<usize> = None;

        let is_last_step = self.step_numbers.get(self.next_step_number_i).is_none();

        if self.show_all
            || next_stack_statement
                .split_whitespace()
                .next()
                .is_some_and(|t| {
                    metamath_data
                        .logical_typecodes
                        .iter()
                        .any(|lt| lt.typecode == t)
                })
            || is_last_step
        {
            match self.previous_proof_line_mapping.get(&next_stack_statement) {
                Some(i) if !is_last_step => {
                    display_step_number = Some(*i);
                }
                _ if !self.only_show_last || is_last_step => {
                    self.proof_lines_returned += 1;
                    proof_line = Some(model::ProofLine {
                        step_name: self.proof_lines_returned.to_string(),
                        hypotheses: hypotheses_nums.iter().map(|&i| i.to_string()).collect(),
                        reference: step
                            .label
                            .as_ref()
                            .map(|s| *s)
                            .unwrap_or_else(|s| s)
                            .to_string(),
                        reference_number: step.label_theorem_number,
                        indention: 1,
                        assertion: next_stack_statement.clone(),
                        old_assertion: None,
                    });
                    display_step_number = Some(self.proof_lines_returned);
                    self.previous_proof_line_mapping
                        .insert(next_stack_statement.clone(), self.proof_lines_returned);
                }
                _ => {}
            }
        }

        if next_step_number.save {
            self.proof_steps.push(ProofStep {
                label: Ok(""),
                label_theorem_number: None,
                hypotheses: Vec::new(),
                statement: Err(next_stack_statement.clone()),
                distinct_var_conditions: None,
            });
        }

        self.stack.push(StackLine {
            statement: next_stack_statement,
            display_step_number,
        });

        // println!("\nStack:");
        // for stack_line in &self.stack {
        //     println!(
        //         "{:?}: {}",
        //         stack_line.display_step_number, stack_line.statement
        //     )
        // }

        Ok(match proof_line {
            Some(pl) => StepResult::ProofLine(pl),
            None => StepResult::NoProofLine,
        })
    }

    fn calc_step_application(
        step: &ProofStep,
        stack: &Vec<StackLine>,
        distinct_var_conditions: &HashSet<(String, String)>,
        metamath_data: &MetamathData,
    ) -> Result<(String, Vec<usize>), Error> {
        if stack.len() < step.hypotheses.len() {
            return Err(Error::InvalidProofError);
        }

        let mut var_map: HashMap<&str, &str> = HashMap::new();

        let mut hypotheses_nums: Vec<usize> = Vec::new();

        for (index, hypothesis) in step.hypotheses.iter().enumerate() {
            let stack_line = stack
                .get(stack.len() - step.hypotheses.len() + index)
                .ok_or(Error::InternalLogicError)?;

            if let Some(num) = stack_line.display_step_number {
                hypotheses_nums.push(num);
            }

            if hypothesis.is_floating_hypothesis {
                let (hypothesis_typecode, hypothesis_variable) = hypothesis
                    .statement
                    .as_ref()
                    .map(|s| *s)
                    .unwrap_or_else(|s| s)
                    .split_once(' ')
                    .ok_or(Error::InternalLogicError)?;

                if hypothesis_typecode
                    != stack_line
                        .statement
                        .split_ascii_whitespace()
                        .next()
                        .ok_or(Error::InternalLogicError)?
                {
                    return Err(Error::InvalidProofError);
                }

                let mapped = stack_line
                    .statement
                    .split_once(' ')
                    .ok_or(Error::InvalidProofError)?
                    .1;

                var_map.insert(hypothesis_variable, mapped);
            } else {
                if !Verifier::check_substitution(
                    &stack_line.statement,
                    hypothesis
                        .statement
                        .as_ref()
                        .map(|s| *s)
                        .unwrap_or_else(|s| s),
                    &var_map,
                ) {
                    return Err(Error::InvalidProofError);
                }
            }
        }

        let empty_hash_set: HashSet<(String, String)> = HashSet::new();

        let step_distinct_var_conditions = step.distinct_var_conditions.unwrap_or(&empty_hash_set);

        if !step_distinct_var_conditions.is_empty() {
            let substitutions_variables: HashMap<&str, HashSet<&str>> = var_map
                .iter()
                .map(|(var, sub)| {
                    (
                        *var,
                        sub.split_ascii_whitespace()
                            .filter(|symbol| metamath_data.is_variable(symbol))
                            .collect(),
                    )
                })
                .collect();

            for (var_1, var_2) in step_distinct_var_conditions {
                if let Some(var_1_sub_vars) = substitutions_variables.get(&**var_1) {
                    if let Some(var_2_sub_vars) = substitutions_variables.get(&**var_2) {
                        for &var_1_var in var_1_sub_vars.iter() {
                            for &var_2_var in var_2_sub_vars.iter() {
                                if var_1_var == var_2_var
                                    || !distinct_var_conditions
                                        .contains(&(var_1_var.to_string(), var_2_var.to_string()))
                                {
                                    return Err(Error::InvalidProofError);
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok((
            Verifier::calc_substitution(
                step.statement.as_ref().map(|s| *s).unwrap_or_else(|s| &s),
                &var_map,
            ),
            hypotheses_nums,
        ))
    }

    fn check_substitution(
        stack_statement: &str,
        theorem_statement: &str,
        var_mapping: &HashMap<&str, &str>,
    ) -> bool {
        let mut stack_statement_iter = stack_statement.split_ascii_whitespace();

        let mut theorem_statement_iter = theorem_statement
            .split_ascii_whitespace()
            .map(|t| {
                if let Some(sub) = var_mapping.get(&t) {
                    sub.split_ascii_whitespace()
                } else {
                    t.split_ascii_whitespace()
                }
            })
            .flatten();

        loop {
            match (stack_statement_iter.next(), theorem_statement_iter.next()) {
                (Some(stack_t), Some(theorem_t)) => {
                    if stack_t != theorem_t {
                        return false;
                    }
                }
                (None, Some(_)) | (Some(_), None) => return false,
                (None, None) => return true,
            }
        }
    }

    fn calc_substitution(statement: &str, var_mapping: &HashMap<&str, &str>) -> String {
        statement
            .split_ascii_whitespace()
            .map(|t| {
                if let Some(sub) = var_mapping.get(&t) {
                    *sub
                } else {
                    t
                }
            })
            .fold_to_space_seperated_string()
    }

    pub fn verify_proof(
        theorem: &Theorem,
        metamath_data: &MetamathData,
        already_calculated_proof_steps: Option<&HashMap<&str, ProofStep>>,
        prev_flaoting_hypotheses: Option<&Vec<FloatingHypothesis>>,
        compressed_infered_proof_steps: Option<Vec<ProofStep>>,
    ) -> Result<VerificationResult, Error> {
        let mut verifier = match Verifier::new(
            theorem,
            metamath_data,
            false,
            true,
            already_calculated_proof_steps,
            prev_flaoting_hypotheses,
            compressed_infered_proof_steps,
        ) {
            Ok(VerifierCreationResult::Verifier(v)) => v,
            Ok(VerifierCreationResult::IsAxiom) => return Ok(VerificationResult::Correct),
            Ok(VerifierCreationResult::IsIncomplete) => return Ok(VerificationResult::Incomplete),
            Err(Error::InvalidProofError) => return Ok(VerificationResult::Incorrect),
            Err(err) => return Err(err),
        };

        let mut last_proof_line: Option<model::ProofLine> = None;

        loop {
            match verifier.proccess_next_step(metamath_data) {
                Ok(StepResult::VerifierFinished) => break,
                Ok(StepResult::ProofLine(pl)) => last_proof_line = Some(pl),
                Ok(StepResult::NoProofLine) => {}
                Err(Error::InvalidProofError) => return Ok(VerificationResult::Incorrect),
                Err(err) => return Err(err),
            }
        }

        Ok(if let Some(pl) = last_proof_line {
            if pl.assertion == theorem.assertion {
                VerificationResult::Correct
            } else {
                VerificationResult::Incorrect
            }
        } else {
            VerificationResult::Incorrect
        })
    }
}
