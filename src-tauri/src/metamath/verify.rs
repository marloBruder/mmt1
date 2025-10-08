use std::collections::{HashMap, HashSet};

use crate::{
    metamath::mmp_parser::LocateAfterRef,
    model::{self, MetamathData, Theorem},
    util::StrIterToSpaceSeperatedString,
    Error,
};

pub struct Verifier {
    proof_steps: Vec<ProofStep>,
    step_numbers: Vec<ProofNumber>,
    next_step_number_i: usize,
    stack: Vec<StackLine>,
    proof_lines_returned: usize,
    previous_proof_line_mapping: HashMap<String, usize>,
    show_all: bool,
}

#[derive(Debug)]
struct ProofStep {
    pub label: String,
    pub label_theorem_number: Option<u32>,
    pub hypotheses: Vec<ProofStepHypothesis>,
    pub statement: String,
}

#[derive(Debug)]
pub struct ProofStepHypothesis {
    pub statement: String,
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

pub enum VerifierCreationResult {
    Verifier(Verifier),
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

impl Verifier {
    pub fn new(
        theorem: &Theorem,
        metamath_data: &MetamathData,
        show_all: bool,
    ) -> Result<VerifierCreationResult, Error> {
        let opt_proof_steps_and_numbers = if let Some(proof) = theorem.proof.as_ref() {
            if proof.starts_with("( ") {
                Verifier::calc_proof_steps_and_numbers_compressed(theorem, metamath_data)?
            } else {
                Verifier::calc_proof_steps_and_numbers_uncompressed(theorem, metamath_data)?
            }
        } else {
            return Ok(VerifierCreationResult::IsAxiom);
        };

        let Some((proof_steps, step_numbers)) = opt_proof_steps_and_numbers else {
            return Ok(VerifierCreationResult::IsIncomplete);
        };

        Ok(VerifierCreationResult::Verifier(Verifier {
            proof_steps,
            step_numbers,
            next_step_number_i: 0,
            stack: Vec::new(),
            proof_lines_returned: 0,
            previous_proof_line_mapping: HashMap::new(),
            show_all,
        }))
    }

    fn calc_proof_steps_and_numbers_compressed(
        theorem: &Theorem,
        metamath_data: &MetamathData,
    ) -> Result<Option<(Vec<ProofStep>, Vec<ProofNumber>)>, Error> {
        let Some(proof) = theorem.proof.as_ref() else {
            return Err(Error::InternalLogicError);
        };

        let mut steps = Vec::new();

        let hypotheses = Verifier::calc_all_hypotheses_of_theorem(theorem, metamath_data);

        for (hypothesis, label) in hypotheses {
            steps.push(ProofStep {
                label,
                label_theorem_number: None,
                hypotheses: Vec::new(),
                statement: hypothesis.statement,
            })
        }

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

            if label == theorem.label {
                return Err(Error::InvalidProofError);
            }

            if let Some((theorem_i, theorem)) = metamath_data
                .database_header
                .theorem_locate_after_iter(Some(LocateAfterRef::LocateAfter(&theorem.label)))
                .enumerate()
                .find(|(_, t)| t.label == label)
            {
                let theorem_hypotheses =
                    Verifier::calc_all_hypotheses_of_theorem(theorem, metamath_data);
                steps.push(ProofStep {
                    label: label.to_string(),
                    label_theorem_number: Some((theorem_i + 1) as u32),
                    hypotheses: theorem_hypotheses
                        .into_iter()
                        .map(|(hyp, _label)| hyp)
                        .collect(),
                    statement: theorem.assertion.clone(),
                });
            } else {
                let floating_hypothesis = metamath_data
                    .database_header
                    .floating_hypohesis_locate_after_iter(Some(LocateAfterRef::LocateAfter(
                        &theorem.label,
                    )))
                    .find(|fh| fh.label == label)
                    .ok_or(Error::NotFoundError)?;

                steps.push(ProofStep {
                    label: label.to_string(),
                    label_theorem_number: None,
                    hypotheses: Vec::new(),
                    statement: floating_hypothesis.to_assertions_string(),
                });
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

    fn calc_all_hypotheses_of_theorem(
        theorem: &Theorem,
        metamath_data: &MetamathData,
    ) -> Vec<(ProofStepHypothesis, String)> {
        let mut hypotheses = Vec::new();

        // Calculate variables occuring in assertion and hypotheses
        let variables = Verifier::calc_variables_of_theorem(theorem, metamath_data);

        // Calculate proof steps of floating hypotheses
        for floating_hypothesis in &metamath_data.optimized_data.floating_hypotheses {
            if variables.contains(&*floating_hypothesis.variable) {
                hypotheses.push((
                    ProofStepHypothesis {
                        statement: format!(
                            "{} {}",
                            floating_hypothesis.typecode, floating_hypothesis.variable
                        ),
                        is_floating_hypothesis: true,
                    },
                    floating_hypothesis.label.clone(),
                ));
            }
        }

        // Calculate proof steps of essential hypotheses
        for hypothesis in &theorem.hypotheses {
            hypotheses.push((
                ProofStepHypothesis {
                    statement: hypothesis.expression.clone(),
                    is_floating_hypothesis: false,
                },
                hypothesis.label.clone(),
            ));
        }

        hypotheses
    }

    fn calc_variables_of_theorem<'a>(
        theorem: &'a Theorem,
        metamath_data: &MetamathData,
    ) -> HashSet<&'a str> {
        let mut variables = HashSet::new();

        Verifier::add_variables_to_hashset_from_statement(
            &mut variables,
            &theorem.assertion,
            metamath_data,
        );

        for hypothesis in &theorem.hypotheses {
            Verifier::add_variables_to_hashset_from_statement(
                &mut variables,
                &hypothesis.expression,
                metamath_data,
            );
        }

        variables
    }

    fn add_variables_to_hashset_from_statement<'a>(
        hashset: &mut HashSet<&'a str>,
        statement: &'a str,
        metamath_data: &MetamathData,
    ) {
        for token in statement.split_whitespace() {
            if metamath_data.is_variable(token) {
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
                _ => return Err(Error::InvalidFormatError),
            }
        }
        if num == 0 {
            return Err(Error::InvalidFormatError);
        }
        Ok(num)
    }

    fn calc_proof_steps_and_numbers_uncompressed(
        theorem: &Theorem,
        metamath_data: &MetamathData,
    ) -> Result<Option<(Vec<ProofStep>, Vec<ProofNumber>)>, Error> {
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
                .find(|(_, ps)| ps.label == token)
            {
                proof_step_numbers.push(ProofNumber {
                    number: (i + 1) as u32,
                    save: false,
                });
            } else {
                proof_steps.push(Verifier::calc_proof_step_from_label(
                    token,
                    theorem,
                    metamath_data,
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

    fn calc_proof_step_from_label(
        label: &str,
        theorem: &Theorem,
        metamath_data: &MetamathData,
    ) -> Result<ProofStep, Error> {
        if let Some(hyp) = theorem.hypotheses.iter().find(|h| h.label == label) {
            return Ok(ProofStep {
                label: label.to_string(),
                label_theorem_number: None,
                hypotheses: Vec::new(),
                statement: hyp.expression.clone(),
            });
        }

        if let Some(floating_hypothesis) = metamath_data
            .database_header
            .floating_hypohesis_locate_after_iter(Some(LocateAfterRef::LocateAfter(&theorem.label)))
            .find(|fh| fh.label == label)
        {
            return Ok(ProofStep {
                label: label.to_string(),
                label_theorem_number: None,
                hypotheses: Vec::new(),
                statement: floating_hypothesis.to_assertions_string(),
            });
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
            let label_theorem_hypotheses =
                Verifier::calc_all_hypotheses_of_theorem(theorem, metamath_data);
            return Ok(ProofStep {
                label: label.to_string(),
                label_theorem_number: Some((theorem_i + 1) as u32),
                hypotheses: label_theorem_hypotheses
                    .into_iter()
                    .map(|(hyp, _label)| hyp)
                    .collect(),
                statement: theorem.assertion.clone(),
            });
        }

        Err(Error::NotFoundError)
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
            step.statement.clone()
        } else {
            let (next_step, new_hypotheses_nums) =
                Verifier::calc_step_application(step, &self.stack)?;
            hypotheses_nums = new_hypotheses_nums;
            for _ in 0..step.hypotheses.len() {
                self.stack.pop();
            }
            next_step
        };

        let mut proof_line: Option<model::ProofLine> = None;

        let mut display_step_number: Option<usize> = None;

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
            || self.step_numbers.get(self.next_step_number_i).is_none()
        {
            match self.previous_proof_line_mapping.get(&next_stack_statement) {
                Some(i) => {
                    display_step_number = Some(*i);
                }
                None => {
                    self.proof_lines_returned += 1;
                    proof_line = Some(model::ProofLine {
                        step_name: self.proof_lines_returned.to_string(),
                        hypotheses: hypotheses_nums.iter().map(|&i| i.to_string()).collect(),
                        reference: step.label.clone(),
                        reference_number: step.label_theorem_number,
                        indention: 1,
                        assertion: next_stack_statement.clone(),
                        old_assertion: None,
                    });
                    display_step_number = Some(self.proof_lines_returned);
                    self.previous_proof_line_mapping
                        .insert(next_stack_statement.clone(), self.proof_lines_returned);
                }
            }
        }

        if next_step_number.save {
            self.proof_steps.push(ProofStep {
                label: String::new(),
                label_theorem_number: None,
                hypotheses: Vec::new(),
                statement: next_stack_statement.clone(),
            });
        }

        self.stack.push(StackLine {
            statement: next_stack_statement,
            display_step_number,
        });

        // println!("\nStack:");
        // for stack_line in &self.stack {
        //     println!(
        //         "{}: {}",
        //         stack_line.display_step_number, stack_line.statement
        //     )
        // }

        Ok(match proof_line {
            Some(pl) => StepResult::ProofLine(pl),
            None => StepResult::NoProofLine,
        })
    }

    fn calc_step_application<'a>(
        step: &'a ProofStep,
        stack: &Vec<StackLine>,
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
                let mut hypothesis_token_iter = hypothesis.statement.split_ascii_whitespace();
                let hypothesis_typecode = hypothesis_token_iter
                    .next()
                    .ok_or(Error::InternalLogicError)?;
                let hypothesis_variable = hypothesis_token_iter
                    .next()
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
                if stack_line.statement
                    != Verifier::calc_substitution(&hypothesis.statement, &var_map)
                {
                    return Err(Error::InvalidProofError);
                }
            }
        }
        Ok((
            Verifier::calc_substitution(&step.statement, &var_map),
            hypotheses_nums,
        ))
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
    ) -> Result<VerificationResult, Error> {
        let mut verifier = match Verifier::new(theorem, metamath_data, false) {
            Ok(VerifierCreationResult::Verifier(v)) => v,
            Ok(VerifierCreationResult::IsAxiom) => return Ok(VerificationResult::Correct),
            Ok(VerifierCreationResult::IsIncomplete) => return Ok(VerificationResult::Incomplete),
            Err(Error::InvalidProofError) => return Ok(VerificationResult::Incorrect),
            Err(err) => return Err(err),
        };

        loop {
            match verifier.proccess_next_step(metamath_data) {
                Ok(StepResult::VerifierFinished) => return Ok(VerificationResult::Correct),
                Ok(_) => {}
                Err(Error::InvalidProofError) => return Ok(VerificationResult::Incorrect),
                Err(err) => return Err(err),
            }
        }
    }
}
