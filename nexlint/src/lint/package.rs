// Copyright (c) The nextest Contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::{lint::LintContext, prelude::*};
use camino::Utf8Path;
use guppy::graph::{PackageGraph, PackageMetadata};

/// Represents a linter that runs once per package.
pub trait PackageLinter: Linter {
    fn run<'l>(
        &self,
        ctx: &PackageContext<'l>,
        out: &mut LintFormatter<'l, '_>,
    ) -> Result<RunStatus<'l>>;
}

/// Lint context for an individual package.
#[derive(Copy, Clone, Debug)]
pub struct PackageContext<'l> {
    project_ctx: &'l ProjectContext<'l>,
    // PackageContext requires the package graph to be computed and available, though ProjectContext
    // does not.
    package_graph: &'l PackageGraph,
    workspace_path: &'l Utf8Path,
    metadata: PackageMetadata<'l>,
}

impl<'l> PackageContext<'l> {
    pub fn new(
        project_ctx: &'l ProjectContext<'l>,
        package_graph: &'l PackageGraph,
        workspace_path: &'l Utf8Path,
        metadata: PackageMetadata<'l>,
    ) -> Result<Self> {
        Ok(Self {
            project_ctx,
            package_graph,
            workspace_path,
            metadata,
        })
    }

    /// Returns the project context.
    pub fn project_ctx(&self) -> &'l ProjectContext<'l> {
        self.project_ctx
    }

    /// Returns the package graph.
    pub fn package_graph(&self) -> &'l PackageGraph {
        self.package_graph
    }

    /// Returns the relative path for this package in the workspace.
    pub fn workspace_path(&self) -> &'l Utf8Path {
        self.workspace_path
    }

    /// Returns the metadata for this package.
    pub fn metadata(&self) -> &PackageMetadata<'l> {
        &self.metadata
    }
}

impl<'l> LintContext<'l> for PackageContext<'l> {
    fn kind(&self) -> LintKind<'l> {
        LintKind::Package {
            name: self.metadata.name(),
            workspace_path: self.workspace_path,
        }
    }
}
