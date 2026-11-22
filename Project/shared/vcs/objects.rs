use super::git;
use super::repository::Repository;
use anyhow::{Context, Result, bail, ensure};
use std::{
    io::{BufRead, BufReader, Read, Write},
    process::Stdio,
};

impl Repository<'_> {
    pub fn blobs(
        &self,
        objects: &[&str],
        mut each: impl FnMut(Vec<u8>) -> Result<()>,
    ) -> Result<()> {
        let nothing_requested = objects.is_empty();
        if nothing_requested {
            return Ok(());
        }
        let mut child = git::command(self.root, &["cat-file", "--batch"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        let input = child.stdin.take().context("git cat-file has no input")?;
        let output = child.stdout.take().context("git cat-file has no output")?;
        let exchanged = exchange(input, BufReader::new(output), objects, &mut each);
        let mut stderr = String::new();
        let mut errors = child.stderr.take().context("git cat-file has no errors")?;
        errors.read_to_string(&mut stderr)?;
        let status = child.wait()?;
        let reason = stderr.trim();
        let explained = status.success() || reason.is_empty();
        ensure!(explained, "git failed: {reason}");
        exchanged?;
        ensure!(status.success(), "git failed: {reason}");
        Ok(())
    }
}

fn exchange(
    mut input: impl Write,
    mut output: impl BufRead,
    objects: &[&str],
    each: &mut impl FnMut(Vec<u8>) -> Result<()>,
) -> Result<()> {
    for object in objects {
        input.write_all(format!("{object}\n").as_bytes())?;
        let mut header = String::new();
        output.read_line(&mut header)?;
        let header = header.trim_end();
        let fields: Vec<&str> = header.split(' ').collect();
        let &[_, "blob", size] = fields.as_slice() else {
            bail!("{object} is not a readable blob: {header}");
        };
        let mut content = vec![0; size.parse()?];
        output.read_exact(&mut content)?;
        let mut separator = [0];
        output.read_exact(&mut separator)?;
        let terminated = separator == *b"\n";
        ensure!(terminated, "git cat-file output breaks after {object}");
        each(content)?;
    }
    Ok(())
}
