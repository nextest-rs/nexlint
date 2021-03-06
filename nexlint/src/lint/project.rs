// Copyright (c) The nextest Contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::{lint::LintContext, prelude::*, NexLintContext};
use camino::{Utf8Path, Utf8PathBuf};
use guppy::graph::PackageGraph;

/// Represents a linter that checks some property for the overall project.
///
/// Linters that implement `ProjectLinter` will run once for the whole project.
pub trait ProjectLinter: Linter {
    // Since ProjectContext is only 1 word long, clippy complains about passing it by reference. Do
    // it that way for consistency reasons.
    #[allow(clippy::trivially_copy_pass_by_ref)]
    /// Executes the lint against the given project context.
    fn run<'l>(
        &self,
        ctx: &ProjectContext<'l>,
        out: &mut LintFormatter<'l, '_>,
    ) -> Result<RunStatus<'l>>;
}

/// Overall linter context for a project.
#[derive(Debug)]
pub struct ProjectContext<'l> {
    core: &'l NexLintContext,
}

impl<'l> ProjectContext<'l> {
    pub fn new(core: &'l NexLintContext) -> Self {
        Self { core }
    }

    /// Returns the core context.
    pub fn core(&self) -> &'l NexLintContext {
        self.core
    }

    /// Returns the project root.
    pub fn project_root(&self) -> &'l Utf8Path {
        self.core.project_root()
    }

    /// Returns the package graph, computing it for the first time if necessary.
    pub fn package_graph(&self) -> Result<&'l PackageGraph> {
        self.core.package_graph()
    }

    /// Returns the absolute path from the project root.
    pub fn full_path(&self, path: impl AsRef<Utf8Path>) -> Utf8PathBuf {
        self.core.project_root().join(path.as_ref())
    }

    /// Returns the name of the hakari workspace-hack package if it exists
    pub fn workspace_hack_name(&self) -> Option<&'l str> {
        self.core
            .hakari_config()
            .and_then(|config| config.builder.hakari_package.as_deref())
    }
}

impl<'l> LintContext<'l> for ProjectContext<'l> {
    fn kind(&self) -> LintKind<'l> {
        LintKind::Project
    }
}
