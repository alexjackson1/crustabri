use super::ConstraintsEncoder;
use crate::{
    aa::{AAFramework, Argument},
    sat::{clause, Assignment, Literal, SatSolver},
    utils::{Label, LabelType},
};

/// An encoder for the complete semantics adding auxiliary variables to make it polynomial.
#[derive(Default)]
pub struct AuxVarCompleteConstraintsEncoder;

impl AuxVarCompleteConstraintsEncoder {
    pub(crate) fn encode_disjunction_var<T>(
        af: &AAFramework<T>,
        solver: &mut dyn SatSolver,
        arg: &Label<T>,
    ) where
        T: LabelType,
    {
        let attacked_id = arg.id();
        let attacked_disjunction_solver_var =
            Self::arg_id_to_solver_disjunction_var(attacked_id) as isize;
        Self::encode_disjunction_var_with(
            af,
            solver,
            arg,
            attacked_disjunction_solver_var,
            &Self::arg_id_to_solver_var,
        )
    }

    pub(crate) fn encode_disjunction_var_with<T>(
        af: &AAFramework<T>,
        solver: &mut dyn SatSolver,
        arg: &Label<T>,
        disjunction_var: isize,
        arg_id_to_solver_var: &dyn Fn(usize) -> usize,
    ) where
        T: LabelType,
    {
        let arg_id = arg.id();
        let arg_var = arg_id_to_solver_var(arg_id) as isize;
        solver.add_clause(clause![-arg_var, -disjunction_var]);
        let mut full_cl = clause![-disjunction_var];
        af.iter_attacks_to(arg).for_each(|att| {
            let attacker_id = att.attacker().id();
            let attacker_solver_var = arg_id_to_solver_var(attacker_id) as isize;
            solver.add_clause(clause![disjunction_var, -attacker_solver_var]);
            full_cl.push(attacker_solver_var.into());
        });
        solver.add_clause(full_cl);
    }

    pub(crate) fn encode_attack_constraints_for_arg<T>(
        af: &AAFramework<T>,
        solver: &mut dyn SatSolver,
        arg: &Label<T>,
        arg_id_to_solver_var: &dyn Fn(usize) -> usize,
        arg_id_to_solver_disjunction_var: &dyn Fn(usize) -> usize,
    ) where
        T: LabelType,
    {
        let attacked_id = arg.id();
        let attacked_solver_var = arg_id_to_solver_var(attacked_id) as isize;
        let mut full_cl = clause![attacked_solver_var];
        af.iter_attacks_to(arg).for_each(|att| {
            let attacker_id = att.attacker().id();
            let attacker_disjunction_solver_var =
                arg_id_to_solver_disjunction_var(attacker_id) as isize;
            solver.add_clause(clause![
                -attacked_solver_var,
                attacker_disjunction_solver_var
            ]);
            full_cl.push((-attacker_disjunction_solver_var).into());
        });
        solver.add_clause(full_cl);
    }

    pub(crate) fn encode_range_constraint<T>(
        solver: &mut dyn SatSolver,
        arg: &Label<T>,
        n_args: usize,
    ) where
        T: LabelType,
    {
        let range_var = Self::arg_id_to_range_var(n_args, arg.id()) as isize;
        let arg_var = Self::arg_id_to_solver_var(arg.id()) as isize;
        let att_disj_var = Self::arg_id_to_solver_disjunction_var(arg.id()) as isize;
        solver.add_clause(clause!(-arg_var, range_var));
        solver.add_clause(clause!(-att_disj_var, range_var));
        solver.add_clause(clause!(-range_var, arg_var, att_disj_var));
    }

    fn arg_id_to_solver_disjunction_var(id: usize) -> usize {
        Self::arg_id_to_solver_var(id) - 1
    }

    pub(crate) fn arg_id_to_solver_var(id: usize) -> usize {
        (id + 1) << 1
    }

    pub(crate) fn arg_id_from_solver_var(v: usize) -> Option<usize> {
        if v & 1 == 1 {
            None
        } else {
            Some((v >> 1) - 1)
        }
    }

    pub(crate) fn arg_id_to_range_var(n_args: usize, id: usize) -> usize {
        (n_args << 1) + id + 1
    }
}

impl<T> ConstraintsEncoder<T> for AuxVarCompleteConstraintsEncoder
where
    T: LabelType,
{
    fn encode_constraints(&self, af: &AAFramework<T>, solver: &mut dyn SatSolver) {
        solver.reserve(af.n_arguments() << 1);
        af.argument_set().iter().for_each(|arg| {
            Self::encode_attack_constraints_for_arg(
                af,
                solver,
                arg,
                &Self::arg_id_to_solver_var,
                &Self::arg_id_to_solver_disjunction_var,
            );
            Self::encode_disjunction_var(af, solver, arg);
        });
    }

    fn encode_constraints_and_range(&self, af: &AAFramework<T>, solver: &mut dyn SatSolver) {
        solver.reserve(af.n_arguments() * 3);
        af.argument_set().iter().for_each(|arg| {
            Self::encode_attack_constraints_for_arg(
                af,
                solver,
                arg,
                &Self::arg_id_to_solver_var,
                &Self::arg_id_to_solver_disjunction_var,
            );
            Self::encode_disjunction_var(af, solver, arg);
            Self::encode_range_constraint(solver, arg, af.n_arguments());
        });
    }

    fn first_range_var(&self, n_args: usize) -> usize {
        Self::arg_id_to_range_var(n_args, 0)
    }

    fn assignment_to_extension<'a>(
        &self,
        assignment: &Assignment,
        af: &'a AAFramework<T>,
    ) -> Vec<&'a Argument<T>> {
        assignment
            .iter()
            .filter_map(|(var, opt_v)| match opt_v {
                Some(true) => Self::arg_id_from_solver_var(var)
                    .and_then(|id| {
                        if id < af.n_arguments() {
                            Some(id)
                        } else {
                            None
                        }
                    })
                    .map(|id| af.argument_set().get_argument_by_id(id)),
                _ => None,
            })
            .collect()
    }

    fn arg_to_lit(&self, arg: &Argument<T>) -> Literal {
        Literal::from(Self::arg_id_to_solver_var(arg.id()) as isize)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        aa::{AAFramework, ArgumentSet},
        encodings::{AuxVarCompleteConstraintsEncoder, ConstraintsEncoder},
        sat::default_solver,
    };

    #[test]
    fn test_no_attacks() {
        let af = AAFramework::new_with_argument_set(ArgumentSet::new_with_labels(&["a0"]));
        let encoder = AuxVarCompleteConstraintsEncoder::default();
        let mut solver = default_solver();
        encoder.encode_constraints(&af, solver.as_mut());
        assert_ne!(solver.n_vars(), 0);
    }
}
