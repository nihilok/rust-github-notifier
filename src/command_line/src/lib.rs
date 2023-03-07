use std::process::Command;

pub fn execute_command(command_line: &str, print_stderr: bool) -> bool {
    let output = Command::new("sh")
        .arg("-c")
        .arg(format!("{command_line}"))
        .output();
    return match output {
        Ok(output) => {
            if !output.status.success() {
                if output.stderr.len() > 0 && print_stderr {
                    println!(
                        "{}",
                        String::from_utf8(output.stderr)
                            .expect("Could not decode stderr")
                    );
                }
                return false;
            }
            true
        }
        Err(err) => {
            dbg!("{}", err);
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = execute_command("echo hello", false);
        assert_eq!(result, true);
    }

    #[test]
    fn it_fails() {
        let result = execute_command("invalid-command-xxxxxxxxxxxx", false);
        assert_eq!(result, false);
    }
}
