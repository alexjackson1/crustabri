use crate::{Argument, ResponseWriter};
use anyhow::{Context, Result};
use std::io::Write;

/// A writer for the output format used in the ICCMA 2023 competition.
#[derive(Default)]
pub struct Iccma23Writer;

impl ResponseWriter<usize> for Iccma23Writer {
    fn write_no_extension(&self, writer: &mut dyn Write) -> Result<()> {
        super::specs::write_no_extension(writer)
    }

    fn write_single_extension(
        &self,
        writer: &mut dyn Write,
        extension: &[&Argument<usize>],
    ) -> Result<()> {
        let context = "while writing an extension";
        write!(writer, "w").context(context)?;
        extension
            .iter()
            .try_for_each(|arg| write!(writer, " {}", arg).context(context))?;
        writeln!(writer).context(context)?;
        writer.flush().context(context)
    }

    fn write_acceptance_status(
        &self,
        writer: &mut dyn Write,
        acceptance_status: bool,
    ) -> Result<()> {
        super::specs::write_acceptance_status(writer, acceptance_status)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ArgumentSet;
    use std::io::BufWriter;

    #[test]
    fn test_write_single_extension() {
        let arg_names = vec![0, 1, 2];
        let args = ArgumentSet::new_with_labels(&arg_names);
        let writer = Iccma23Writer::default();
        let mut buffer = BufWriter::new(Vec::new());
        writer
            .write_single_extension(&mut buffer, &args.iter().collect::<Vec<&Argument<usize>>>())
            .unwrap();
        assert_eq!(
            "w 0 1 2\n",
            String::from_utf8(buffer.into_inner().unwrap()).unwrap()
        );
    }

    #[test]
    fn test_write_empty_extension() {
        let writer = Iccma23Writer::default();
        let mut buffer = BufWriter::new(Vec::new());
        writer
            .write_single_extension(&mut buffer, &[] as &[&Argument<usize>])
            .unwrap();
        assert_eq!(
            "w\n",
            String::from_utf8(buffer.into_inner().unwrap()).unwrap()
        );
    }

    #[test]
    fn test_write_no_extension() {
        let writer = Iccma23Writer::default();
        let mut buffer = BufWriter::new(Vec::new());
        writer.write_no_extension(&mut buffer).unwrap();
        assert_eq!(
            "NO\n",
            String::from_utf8(buffer.into_inner().unwrap()).unwrap()
        );
    }

    #[test]
    fn test_write_acceptance_status_yes() {
        let writer = Iccma23Writer::default();
        let mut buffer = BufWriter::new(Vec::new());
        writer.write_acceptance_status(&mut buffer, true).unwrap();
        assert_eq!(
            "YES\n",
            String::from_utf8(buffer.into_inner().unwrap()).unwrap()
        );
    }

    #[test]
    fn test_write_acceptance_status_no() {
        let writer = Iccma23Writer::default();
        let mut buffer = BufWriter::new(Vec::new());
        writer.write_acceptance_status(&mut buffer, false).unwrap();
        assert_eq!(
            "NO\n",
            String::from_utf8(buffer.into_inner().unwrap()).unwrap()
        );
    }
}
