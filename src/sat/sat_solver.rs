use super::cadical_solver::CadicalSolver;
use std::num::{NonZeroIsize, NonZeroUsize};

/// A variable in a SAT solver.
///
/// A variable is represented by a non-null positive integer.
/// It can be obtained through the [From] trait from an integer type.
pub struct Variable(NonZeroUsize);

macro_rules! impl_var_from {
    ($t: ty) => {
        impl From<$t> for Variable {
            fn from(v: $t) -> Self {
                Self(NonZeroUsize::try_from(v as usize).unwrap())
            }
        }
    };
}
impl_var_from!(usize);
impl_var_from!(u128);
impl_var_from!(u64);
impl_var_from!(u32);
impl_var_from!(u16);
impl_var_from!(u8);

macro_rules! impl_var_from_neg {
    ($t: ty) => {
        impl From<$t> for Variable {
            fn from(v: $t) -> Self {
                if v < 0 {
                    panic!("cannot build a variable from a negative integer")
                }
                Self(NonZeroUsize::try_from(v as usize).unwrap())
            }
        }
    };
}
impl_var_from_neg!(isize);
impl_var_from_neg!(i128);
impl_var_from_neg!(i64);
impl_var_from_neg!(i32);
impl_var_from_neg!(i16);
impl_var_from_neg!(i8);

impl From<Variable> for usize {
    fn from(v: Variable) -> Self {
        v.0.into()
    }
}

/// A literal in a SAT solver.
///
/// A literal is represented by a non-null integer.
/// It can be obtained through the [From] trait from a signed integer type.
pub struct Literal(NonZeroIsize);

macro_rules! impl_lit_from {
    ($t: ty) => {
        impl From<$t> for Literal {
            fn from(l: $t) -> Self {
                Self(NonZeroIsize::try_from(l as isize).unwrap())
            }
        }
    };
}
impl_lit_from!(isize);
impl_lit_from!(i128);
impl_lit_from!(i64);
impl_lit_from!(i32);
impl_lit_from!(i16);
impl_lit_from!(i8);

impl From<Literal> for isize {
    fn from(l: Literal) -> Self {
        l.0.into()
    }
}

/// Builds a clause from a list of integers.
#[macro_export]
macro_rules! clause {
    () => (
        vec![] as Vec<Literal>
    );
    ($($x:expr),+ $(,)?) => (
        [$($x),+].into_iter().map(Literal::from).collect::<Vec<Literal>>()
    );
}

/// An assignment of a set of variables.
///
/// Inside the set of variables involved in the assignment, some may be unassigned.
/// This is the reason why accessors to assigned value returns an [Option<bool>].
#[derive(Debug, PartialEq, Eq)]
pub struct Assignment(Vec<Option<bool>>);

impl Assignment {
    pub(crate) fn new(assignment: Vec<Option<bool>>) -> Self {
        Self(assignment)
    }

    /// Returns the value potentially assigned to the variable.
    ///
    /// The result in an [Option].
    /// In case the variable is not assigned, [Option::None] is returned.
    /// Else, [Option::Some] is returned and contains the assigned value.
    pub(crate) fn value_of<T>(&self, v: T) -> Option<bool>
    where
        T: Into<Variable>,
    {
        self.0[usize::from(v.into()) - 1]
    }

    pub(crate) fn iter(&self) -> AssignmentIterator {
        AssignmentIterator {
            assignment: self,
            next: 0,
        }
    }
}

pub(crate) struct AssignmentIterator<'a> {
    assignment: &'a Assignment,
    next: usize,
}

impl Iterator for AssignmentIterator<'_> {
    type Item = (usize, Option<bool>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.next == self.assignment.0.len() {
            None
        } else {
            self.next += 1;
            Some((self.next, self.assignment.0[self.next - 1]))
        }
    }
}

pub enum SolvingResult {
    Satisfiable(Assignment),
    Unsatisfiable,
    Unknown,
}

impl SolvingResult {
    /// Returns the underlying model if it exists, or [Option::None].
    ///
    /// # Panics
    ///
    /// If the solving result is set [SolvingResult::Unknown], this function panics.
    pub fn unwrap_model(self) -> Option<Assignment> {
        match self {
            SolvingResult::Satisfiable(assignment) => Some(assignment),
            SolvingResult::Unsatisfiable => None,
            SolvingResult::Unknown => {
                panic!(r#"cannot unwrap solving result when the solver returned "Unknown""#)
            }
        }
    }
}

pub trait SatSolver {
    fn add_clause(&mut self, cl: Vec<Literal>);

    fn solve(&mut self) -> SolvingResult;
}

pub(crate) fn default_solver() -> Box<dyn SatSolver> {
    Box::new(CadicalSolver::default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_var_from_pos() {
        let v = Variable::from(1);
        assert_eq!(1, usize::from(v))
    }

    #[test]
    #[allow(unused_must_use)]
    #[should_panic]
    fn test_var_from_null() {
        Variable::from(0);
    }

    #[test]
    #[allow(unused_must_use)]
    #[should_panic]
    fn test_var_from_neg() {
        Variable::from(-1);
    }

    #[test]
    fn test_lit_from_pos() {
        let l = Literal::from(1);
        assert_eq!(1, isize::from(l))
    }

    #[test]
    #[allow(unused_must_use)]
    #[should_panic]
    fn test_lit_from_null() {
        Literal::from(0);
    }

    #[test]
    fn test_lit_from_neg() {
        let l = Literal::from(-1);
        assert_eq!(-1, isize::from(l))
    }

    #[test]
    fn test_solving_result_unwrap_model_some() {
        assert_eq!(
            Some(Assignment::new(vec![])),
            SolvingResult::Satisfiable(Assignment::new(vec![])).unwrap_model()
        );
    }

    #[test]
    fn test_solving_result_unwrap_model_none() {
        assert_eq!(None, SolvingResult::Unsatisfiable.unwrap_model());
    }

    #[test]
    #[should_panic]
    fn test_solving_result_unwrap_model_unknown() {
        SolvingResult::Unknown.unwrap_model();
    }
}