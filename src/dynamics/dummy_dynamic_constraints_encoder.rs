use super::DynamicSolver;
use crate::{
    aa::{AAFramework, Argument, ArgumentSet},
    solvers::{CredulousAcceptanceComputer, SkepticalAcceptanceComputer},
    utils::LabelType,
};

type CredulousAcceptanceComputerFactory<T> =
    dyn for<'a> Fn(&'a AAFramework<T>) -> Box<dyn CredulousAcceptanceComputer<T> + 'a>;
type SkepticalAcceptanceComputerFactory<T> =
    dyn for<'a> Fn(&'a AAFramework<T>) -> Box<dyn SkepticalAcceptanceComputer<T> + 'a>;

/// A dynamic solver that just recomputes from scratch every time it is called.
pub struct DummyDynamicConstraintsEncoder<T>
where
    T: LabelType,
{
    af: AAFramework<T>,
    credulous_acceptance_computer_factory: Option<Box<CredulousAcceptanceComputerFactory<T>>>,
    skeptical_acceptance_computer_factory: Option<Box<SkepticalAcceptanceComputerFactory<T>>>,
}

impl<T> DummyDynamicConstraintsEncoder<T>
where
    T: LabelType,
{
    /// Builds a new dummy solver given factories to build its underlying solvers.
    pub fn new(
        credulous_acceptance_computer_factory: Option<Box<CredulousAcceptanceComputerFactory<T>>>,
        skeptical_acceptance_computer_factory: Option<Box<SkepticalAcceptanceComputerFactory<T>>>,
    ) -> Self {
        Self {
            af: AAFramework::new_with_argument_set(ArgumentSet::new_with_labels(&[])),
            credulous_acceptance_computer_factory,
            skeptical_acceptance_computer_factory,
        }
    }
}

impl<T> DynamicSolver<T> for DummyDynamicConstraintsEncoder<T>
where
    T: LabelType,
{
    fn new_argument(&mut self, label: T) {
        self.af.new_argument(label)
    }

    fn remove_argument(&mut self, label: &T) -> anyhow::Result<()> {
        self.af.remove_argument(label)
    }

    fn new_attack(&mut self, from: &T, to: &T) -> anyhow::Result<()> {
        self.af.new_attack(from, to)
    }

    fn remove_attack(&mut self, from: &T, to: &T) -> anyhow::Result<()> {
        self.af.remove_attack(from, to)
    }
}

impl<T> CredulousAcceptanceComputer<T> for DummyDynamicConstraintsEncoder<T>
where
    T: LabelType,
{
    fn are_credulously_accepted(&mut self, args: &[&T]) -> bool {
        let mut acceptance_computer =
            (self.credulous_acceptance_computer_factory.as_ref().unwrap())(&self.af);
        acceptance_computer.are_credulously_accepted(args)
    }

    fn are_credulously_accepted_with_certificate(
        &mut self,
        args: &[&T],
    ) -> (bool, Option<Vec<&Argument<T>>>) {
        let mut acceptance_computer =
            (self.credulous_acceptance_computer_factory.as_ref().unwrap())(&self.af);
        let (status, ext) = acceptance_computer.are_credulously_accepted_with_certificate(args);
        let extension = ext.map(|e| {
            e.iter()
                .map(|l| self.af.argument_set().get_argument_by_id(l.id()))
                .collect()
        });
        (status, extension)
    }
}

impl<T> SkepticalAcceptanceComputer<T> for DummyDynamicConstraintsEncoder<T>
where
    T: LabelType,
{
    fn are_skeptically_accepted(&mut self, args: &[&T]) -> bool {
        let mut acceptance_computer =
            (self.skeptical_acceptance_computer_factory.as_ref().unwrap())(&self.af);
        acceptance_computer.are_skeptically_accepted(args)
    }

    fn are_skeptically_accepted_with_certificate(
        &mut self,
        args: &[&T],
    ) -> (bool, Option<Vec<&Argument<T>>>) {
        let mut acceptance_computer =
            (self.skeptical_acceptance_computer_factory.as_ref().unwrap())(&self.af);
        let (status, ext) = acceptance_computer.are_skeptically_accepted_with_certificate(args);
        let extension = ext.map(|e| {
            e.iter()
                .map(|l| self.af.argument_set().get_argument_by_id(l.id()))
                .collect()
        });
        (status, extension)
    }
}
