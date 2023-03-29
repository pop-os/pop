use std::{io, process};

pub fn check_output(output: process::Output) -> io::Result<process::Output> {
    check_status(output.status)?;
    Ok(output)
}

pub fn check_status(status: process::ExitStatus) -> io::Result<()> {
    if status.success() {
        Ok(())
    } else {
        Err(io::Error::new(io::ErrorKind::Other, format!("{}", status)))
    }
}
