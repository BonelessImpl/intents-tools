use anyhow::Context;

pub fn format_command(args: &[&str]) -> String {
    args.iter()
        .map(|arg| {
            if arg.chars().any(|c| c.is_whitespace()) {
                format!("'{}'", arg) // Wrap JSON args in single quotes
            } else {
                arg.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn call_cmd<'a>(
    cmd_and_args: impl IntoIterator<Item = &'a str>,
) -> anyhow::Result<std::process::Output> {
    let cmd_and_args = cmd_and_args.into_iter().collect::<Vec<_>>();

    if cmd_and_args.is_empty() {
        return Err(anyhow::anyhow!(
            "Attempted to make a system call to an empty command"
        ));
    }

    let full_cmd = format_command(&cmd_and_args);
    println!("ℹ - Calling command: {full_cmd}");

    let mut cmd = std::process::Command::new(cmd_and_args[0]);
    for arg in cmd_and_args.into_iter().skip(1) {
        cmd.arg(arg);
    }

    let output = cmd
        .output()
        .context(format!("While calling command: {}", full_cmd))?;

    if output.status.success() {
        Ok(output)
    } else {
        Err(anyhow::anyhow!(
            "Command `{}` exited with error.\nStdout: {}\n\nStderr: {}",
            full_cmd,
            String::from_utf8(output.stdout)
                .unwrap_or("<Stderr to string conversion failed>".to_string()),
            String::from_utf8(output.stderr)
                .unwrap_or("<Stderr to string conversion failed>".to_string())
        ))
    }
}
