// Copyright (c) The nextest Contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::git::GitCli;
use camino::{Utf8Path, Utf8PathBuf};
use debug_ignore::DebugIgnore;
use guppy::{graph::PackageGraph, MetadataCommand};
use once_cell::sync::OnceCell;

mod errors;
mod git;
mod lint;

pub use errors::*;

pub mod prelude {
    pub use super::{
        errors::{Result, SystemError},
        lint::{
            content::{ContentContext, ContentLinter},
            file_path::{FilePathContext, FilePathLinter},
            package::{PackageContext, PackageLinter},
            project::{ProjectContext, ProjectLinter},
            runner::{LintEngine, LintEngineConfig, LintResults},
            LintFormatter, LintKind, LintLevel, LintMessage, LintSource, Linter, RunStatus,
            SkipReason,
        },
    };
}

/// Core context shared across all of x.
#[derive(Debug)]
pub struct NexLintContext {
    current_dir: Utf8PathBuf,
    current_rel_dir: Utf8PathBuf,
    git_cli: GitCli,
    package_graph: DebugIgnore<OnceCell<PackageGraph>>,
}

impl NexLintContext {
    /// Creates a new NexLintContext.
    pub fn new(current_dir: Utf8PathBuf) -> Result<Self> {
        let git_cli = GitCli::new()?;
        let current_rel_dir = match current_dir.strip_prefix(git_cli.root()) {
            Ok(rel_dir) => rel_dir.to_path_buf(),
            Err(_) => {
                return Err(SystemError::CwdNotInProjectRoot {
                    current_dir,
                    project_root: git_cli.root().to_owned(),
                })
            }
        };

        Ok(Self {
            current_dir,
            current_rel_dir,
            git_cli,
            package_graph: DebugIgnore(OnceCell::new()),
        })
    }

    pub fn from_current_dir() -> Result<Self> {
        let current_dir: Utf8PathBuf = std::env::current_dir()
            .map_err(|e| SystemError::io("error while fetching current dir", e))?
            .try_into()
            .map_err(|e| SystemError::camino("current dir is not valid UTF-8", e))?;
        Self::new(current_dir)
    }

    /// Returns the project root for this workspace.
    pub fn project_root(&self) -> &Utf8Path {
        self.git_cli.root()
    }

    /// Returns the current working directory for this process.
    pub fn current_dir(&self) -> &Utf8Path {
        &self.current_dir
    }

    /// Returns the current working directory for this process, relative to the project root.
    pub fn current_rel_dir(&self) -> &Utf8Path {
        &self.current_rel_dir
    }

    /// Returns true if x has been run from the project root.
    pub fn current_dir_is_root(&self) -> bool {
        self.current_rel_dir == ""
    }

    /// Returns the Git CLI for this workspace.
    pub fn git_cli(&self) -> &GitCli {
        &self.git_cli
    }

    /// Returns the package graph for this workspace.
    pub fn package_graph(&self) -> Result<&PackageGraph> {
        self.package_graph.get_or_try_init(|| {
            let mut cmd = MetadataCommand::new();
            // Run cargo metadata from the root of the workspace.
            let project_root = self.project_root();
            cmd.current_dir(project_root);
            cmd.build_graph()
                .map_err(|err| SystemError::guppy("building package graph", err))
        })
    }

    /// For a given list of workspace packages, returns a tuple of (known, unknown) packages.
    ///
    /// Initializes the package graph if it isn't already done so, and returns an error if the
    pub fn partition_workspace_names<'a, B>(
        &self,
        names: impl IntoIterator<Item = &'a str>,
    ) -> Result<(B, B)>
    where
        B: Default + Extend<&'a str>,
    {
        let workspace = self.package_graph()?.workspace();
        let (known, unknown) = names
            .into_iter()
            .partition(|name| workspace.contains_name(name));
        Ok((known, unknown))
    }
}
