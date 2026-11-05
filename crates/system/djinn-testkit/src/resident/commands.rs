// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

impl Resident {
    /// A guarded command for `program`, with this resident's environment.
    /// Refused before anything spawns when a wall would be crossed.
    pub fn command(&self, program: &Path, args: &[String]) -> Result<Command, Refusal> {
        self.guarded(SpawnKind::Command, program, args)
    }

    /// Run a guarded one-shot command in the run's job and collect its output.
    pub fn output(&self, program: &Path, args: &[String]) -> Result<Output, HarnessError> {
        let mut child = self.spawn(program, args)?;
        let mut stdout = child.stdout.take().unwrap();
        let mut stderr = child.stderr.take().unwrap();
        let out = std::thread::spawn(move || {
            let mut bytes = Vec::new();
            let _ = stdout.read_to_end(&mut bytes);
            bytes
        });
        let err = std::thread::spawn(move || {
            let mut bytes = Vec::new();
            let _ = stderr.read_to_end(&mut bytes);
            bytes
        });
        let deadline = Instant::now() + COMMAND_PATIENCE;
        let status = loop {
            if let Some(status) = child.try_wait()? {
                break status;
            }
            if Instant::now() >= deadline {
                let _ = child.kill();
                break child.wait()?;
            }
            std::thread::sleep(Duration::from_millis(20));
        };
        Ok(Output {
            status,
            stdout: out.join().unwrap_or_default(),
            stderr: err.join().unwrap_or_default(),
        })
    }

    /// Run a stock SSH client (`ssh-add`, `ssh-keygen`) with
    /// `SSH_AUTH_SOCK` naming this resident's own agent endpoint, which the
    /// guard checks like any endpoint; `stdin` is written and closed. The
    /// call and what it printed are recorded as a step.
    pub fn ssh_client(
        &self,
        program: &Path,
        args: &[&str],
        stdin: &[u8],
    ) -> Result<Output, HarnessError> {
        self.ssh_client_env(program, args, stdin, &[])
    }

    /// [`Self::ssh_client`] with more variables (an `SSH_ASKPASS`, say);
    /// still guarded.
    pub fn ssh_client_env(
        &self,
        program: &Path,
        args: &[&str],
        stdin: &[u8],
        extra: &[(&str, &str)],
    ) -> Result<Output, HarnessError> {
        let args: Vec<String> = args.iter().map(|arg| arg.to_string()).collect();
        let mut env = self.env_map();
        env.extend(extra.iter().map(|(k, v)| (k.to_string(), v.to_string())));
        env.insert("SSH_AUTH_SOCK".into(), self.endpoint("agent"));
        // Win32-OpenSSH exits 255, silently, without it. A machine root
        // (`C:\ProgramData`), not a user's.
        #[cfg(windows)]
        if let Ok(program_data) = std::env::var("ProgramData") {
            env.insert("ProgramData".into(), program_data);
        }
        let mut child = self
            .guarded_env(SpawnKind::Command, program, &args, env)?
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        let label = format!(
            "{} {}",
            program.file_name().unwrap_or_default().to_string_lossy(),
            args.first().map(String::as_str).unwrap_or("")
        );
        self.adopt(&child, label)?;
        {
            use std::io::Write;
            let mut input = child.stdin.take().unwrap();
            let _ = input.write_all(stdin);
        }
        let mut stdout = child.stdout.take().unwrap();
        let mut stderr = child.stderr.take().unwrap();
        let out = std::thread::spawn(move || {
            let mut bytes = Vec::new();
            let _ = stdout.read_to_end(&mut bytes);
            bytes
        });
        let err = std::thread::spawn(move || {
            let mut bytes = Vec::new();
            let _ = stderr.read_to_end(&mut bytes);
            bytes
        });
        let deadline = Instant::now() + COMMAND_PATIENCE;
        let status = loop {
            if let Some(status) = child.try_wait()? {
                break status;
            }
            if Instant::now() >= deadline {
                let _ = child.kill();
                break child.wait()?;
            }
            std::thread::sleep(Duration::from_millis(20));
        };
        let output = Output {
            status,
            stdout: out.join().unwrap_or_default(),
            stderr: err.join().unwrap_or_default(),
        };
        self.shared.step(
            "ssh client",
            json!({
                "resident": self.name,
                "program": program,
                "args": args,
                "exit": output.status.code(),
                "stdout": String::from_utf8_lossy(&output.stdout),
                "stderr": String::from_utf8_lossy(&output.stderr),
            }),
        );
        Ok(output)
    }

    pub(super) fn management_args(&self, args: &[&str]) -> Vec<String> {
        let mut all = vec!["--dir".to_string(), self.vault_dir().display().to_string()];
        if let Some(profile) = &self.profile {
            all.extend(["--profile".to_string(), profile.clone()]);
        }
        all.extend(args.iter().map(|arg| arg.to_string()));
        all
    }
}
