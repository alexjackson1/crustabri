use super::{
    buffered_sat_solver::{BufferedSatSolver, DimacsInstanceRead},
    sat_solver::SolvingResult,
    Literal, SatSolver,
};
use std::{
    io::{Read, Write},
    process::{Command, Stdio},
};

/// A SAT solver which execution is made by a system command.
///
/// The SAT solver must be able to read from Stdin. This behavior can be triggered by a CLI option.
/// The input and output formats must follow the ones from the SAT competition.
pub struct ExternalSatSolver {
    buffered_sat_solver: BufferedSatSolver,
}

impl ExternalSatSolver {
    /// Builds a new external SAT solver.
    ///
    /// The `program` argument is the path from a directory in PATH to the software to execute.
    /// The `options` parameter is the CLI options to provide to the software under execution.
    pub fn new(program: String, options: Vec<String>) -> Self {
        Self {
            buffered_sat_solver: BufferedSatSolver::new(Box::new(move |r| {
                exec_solver(r, &program, &options)
            })),
        }
    }
}

impl SatSolver for ExternalSatSolver {
    fn add_clause(&mut self, cl: Vec<Literal>) {
        self.buffered_sat_solver.add_clause(cl)
    }

    fn solve(&mut self) -> SolvingResult {
        self.buffered_sat_solver.solve()
    }

    fn solve_under_assumptions(&mut self, assumptions: &[Literal]) -> SolvingResult {
        self.buffered_sat_solver
            .solve_under_assumptions(assumptions)
    }

    fn n_vars(&self) -> usize {
        self.buffered_sat_solver.n_vars()
    }
}

fn exec_solver(mut reader: DimacsInstanceRead, program: &str, options: &[String]) -> Box<dyn Read> {
    let mut child = Command::new(program)
        .args(options)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn child process");
    let mut stdin = child.stdin.take().expect("Failed to open stdin");
    std::thread::spawn(move || {
        let mut buffer = String::new();
        loop {
            match reader.read_to_string(&mut buffer) {
                Ok(0) | Err(_) => break,
                Ok(_) => stdin
                    .write_all(buffer.as_bytes())
                    .expect("Failed to write to stdin"),
            }
        }
        stdin.flush()
    });
    let stdout = child.stdout.take().expect("Failed to open stdout");
    child.wait().expect("failed to wait on child");
    Box::new(stdout)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clause;

    fn get_echo_command(content: &str) -> Option<(String, Vec<String>)> {
        if cfg!(target_family = "unix") {
            Some(("echo".to_string(), vec![content.to_string()]))
        } else {
            None
        }
    }

    #[test]
    fn test_solve_output() {
        let (program, options) = match get_echo_command("s SATISFIABLE\nv 1 2 0\n") {
            Some(cmd) => cmd,
            None => return,
        };
        let mut s = ExternalSatSolver::new(program, options);
        s.add_clause(clause![1, 2]);
        let model = s.solve().unwrap_model().unwrap();
        assert!(model.value_of(1).unwrap());
        assert!(model.value_of(2).unwrap());
        assert_eq!(2, s.n_vars());
    }

    #[test]
    fn test_solve_under_assumptions_output() {
        let (program, options) = match get_echo_command("s UNSATISFIABLE\n") {
            Some(cmd) => cmd,
            None => return,
        };
        let mut s = ExternalSatSolver::new(program, options);
        s.add_clause(clause![1, 2]);
        let model = s
            .solve_under_assumptions(&[Literal::from(-1), Literal::from(-2)])
            .unwrap_model();
        assert!(model.is_none());
        assert_eq!(2, s.n_vars());
    }
}
