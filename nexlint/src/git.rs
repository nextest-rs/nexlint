// Copyright (c) The nextest Contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::errors::*;
use camino::{Utf8Path, Utf8PathBuf};
use determinator::Utf8Paths0;
use once_cell::sync::OnceCell;
use std::{
    borrow::Cow,
    ffi::{OsStr, OsString},
    fmt,
    process::{Command, Stdio},
};

/// Support for source control operations through running Git commands.
///
/// This assumes that the underlying Git repository doesn't change in the middle of an operation,
/// and caches data as a result. If mutation operations are added, the caches would need to be
/// invalidated.
#[derive(Clone, Debug)]
pub struct GitCli {
    root: Utf8PathBuf,
    // Caches.
    tracked_files: OnceCell<Utf8Paths0>,
}

impl GitCli {
    /// Creates a new instance of the Git CLI.
    pub fn new() -> Result<Self> {
        let root = Self::repository_root()?;
        Ok(Self {
            root,
            tracked_files: OnceCell::new(),
        })
    }

    /// Returns the root of the repository
    pub fn root(&self) -> &Utf8Path {
        &self.root
    }

    /// Returns the files tracked by Git in this working copy.
    ///
    /// The return value can be iterated on to get a list of paths.
    pub fn tracked_files(&self) -> Result<&Utf8Paths0> {
        self.tracked_files.get_or_try_init(|| {
            // TODO: abstract out SCM and command-running functionality.
            let output = self
                .git_command()
                // The -z causes files to not be quoted, and to be separated by \0.
                .args(["ls-files", "-z"])
                .output()
                .map_err(|err| SystemError::io("running git ls-files", err))?;
            if !output.status.success() {
                return Err(SystemError::Exec {
                    cmd: "git ls-files",
                    status: output.status,
                });
            }

            Utf8Paths0::from_bytes(output.stdout)
                .map_err(|(path, err)| SystemError::NonUtf8Path { path, err })
        })
    }

    /// Returns the merge base of the current commit (`HEAD`) with the specified commit.
    pub fn merge_base(&self, commit_ref: &str) -> Result<GitHash> {
        let output = self
            .git_command()
            .args(["merge-base", "HEAD", commit_ref])
            .output()
            .map_err(|err| {
                SystemError::io(format!("running git merge-base HEAD {}", commit_ref), err)
            })?;
        if !output.status.success() {
            return Err(SystemError::Exec {
                cmd: "git merge-base",
                status: output.status,
            });
        }

        // The output is a hex-encoded hash followed by a newline.
        let stdout = &output.stdout[..(output.stdout.len() - 1)];
        GitHash::from_hex(stdout)
    }

    /// Returns the files changed between the given commits, or the current directory if the new
    /// commit isn't specified.
    ///
    /// For more about the diff filter, see `man git-diff`'s help for `--diff-filter`.
    pub fn files_changed_between<'a>(
        &self,
        old: impl Into<Cow<'a, OsStr>>,
        new: impl Into<Option<Cow<'a, OsStr>>>,
        // TODO: make this more well-typed/express more of the diff model in Rust
        diff_filter: Option<&str>,
    ) -> Result<Utf8Paths0> {
        let mut command = self.git_command();
        command.args(["diff", "-z", "--name-only"]);
        if let Some(diff_filter) = diff_filter {
            command.arg(format!("--diff-filter={}", diff_filter));
        }
        command.arg(old.into());
        if let Some(new) = new.into() {
            command.arg(new);
        }

        let output = command
            .output()
            .map_err(|err| SystemError::io("running git diff", err))?;
        if !output.status.success() {
            return Err(SystemError::Exec {
                cmd: "git diff",
                status: output.status,
            });
        }

        Utf8Paths0::from_bytes(output.stdout)
            .map_err(|(path, err)| SystemError::NonUtf8Path { path, err })
    }

    // ---
    // Helper methods
    // ---

    // Attempt to query for the root of the repository
    fn repository_root() -> Result<Utf8PathBuf> {
        let output = Command::new("git")
            .args(["rev-parse", "--show-toplevel"])
            .stderr(Stdio::inherit())
            .output()
            .map_err(|err| SystemError::io("running git rev-parse --show-toplevel", err))?;
        if !output.status.success() {
            let msg = "unable to find a git repository; \
                nexlint must be run from inside of a git repository";
            return Err(SystemError::git_root(msg));
        }

        let mut git_root_bytes = output.stdout;
        // Pop the newline off the git root bytes.
        git_root_bytes.pop();
        String::from_utf8(git_root_bytes)
            .map(Into::into)
            .map_err(|_| {
                SystemError::git_root("git rev-parse --show-toplevel returned a non-Unicode path")
            })
    }

    // TODO: abstract out command running and error handling
    fn git_command(&self) -> Command {
        // TODO: add support for the GIT environment variable?
        let mut command = Command::new("git");
        command.current_dir(&self.root).stderr(Stdio::inherit());
        command
    }

    pub fn is_git_repo(&self, dir: &Utf8Path) -> Result<bool> {
        let output = self
            .git_command()
            .current_dir(dir)
            .args(["rev-parse", "--git-dir"])
            .output()
            .map_err(|err| SystemError::io("checking if a directory is a git repo", err))?;

        Ok(output.status.success())
    }
}

/// A Git hash.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct GitHash([u8; 20]);

impl GitHash {
    /// Creates a new Git hash from a hex-encoded string.
    pub fn from_hex(hex: impl AsRef<[u8]>) -> Result<Self> {
        let hex = hex.as_ref();
        Ok(GitHash(hex::FromHex::from_hex(hex).map_err(|err| {
            SystemError::from_hex(format!("parsing a Git hash: {:?}", hex), err)
        })?))
    }
}

impl<'a, 'b> From<&'a GitHash> for Cow<'b, OsStr> {
    fn from(git_hash: &'a GitHash) -> Cow<'b, OsStr> {
        OsString::from(format!("{:x}", git_hash)).into()
    }
}

impl fmt::LowerHex for GitHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}
