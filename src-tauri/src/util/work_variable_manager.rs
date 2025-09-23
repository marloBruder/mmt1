use crate::{
    model::{ParseTree, ParseTreeNode, SymbolNumberMapping},
    util::earley_parser_optimized::WorkVariable,
    Error,
};

pub struct WorkVariableManager {
    next_vars: Vec<WorkVariable>,
}

impl WorkVariableManager {
    pub fn new(
        parse_trees: &Vec<&ParseTree>,
        symbol_number_mapping: &SymbolNumberMapping,
    ) -> Result<WorkVariableManager, Error> {
        let mut next_vars: Vec<WorkVariable> = symbol_number_mapping
            .typecode_default_vars
            .iter()
            .map(|(typecode_i, default_var_i)| WorkVariable {
                typecode_i: *typecode_i,
                variable_i: *default_var_i,
                number: 0,
            })
            .collect();

        let mut nodes: Vec<&ParseTreeNode> = parse_trees.iter().map(|pt| &pt.top_node).collect();

        while let Some(node) = nodes.pop() {
            if let ParseTreeNode::WorkVariable(work_var_in_pt) = node {
                if work_var_in_pt.variable_i
                    == symbol_number_mapping
                        .get_typecode_default_variable_i(work_var_in_pt.typecode_i)
                        .ok_or(Error::InternalLogicError)?
                {
                    let Some(work_var) = next_vars
                        .iter_mut()
                        .find(|work_var| work_var.typecode_i == work_var_in_pt.typecode_i)
                    else {
                        return Err(Error::InternalLogicError);
                    };

                    if work_var.number < work_var_in_pt.number + 1 {
                        work_var.number = work_var_in_pt.number + 1;
                    }
                }
            }
        }

        Ok(WorkVariableManager { next_vars })
    }

    pub fn next_var(&mut self, typecode_i: u32) -> Option<WorkVariable> {
        self.next_vars.iter_mut().find_map(|work_var| {
            if work_var.typecode_i == typecode_i {
                let return_var = work_var.clone();
                work_var.number += 1;
                Some(return_var)
            } else {
                None
            }
        })
    }
}
