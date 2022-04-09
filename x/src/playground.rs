// Copyright (c) The nextest Contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Playground for arbitrary code.
//!
//! This lets users experiment with new lints and other throwaway code.
//! Add your code in the spots marked `// --- ADD PLAYGROUND CODE HERE ---`.
//!
//! This file should not have any production-related code checked into it.

#![allow(unused_variables)]

use nexlint::{prelude::*, NexLintContext};
use nexlint_lints::handle_lint_results;
use structopt::StructOpt;

#[derive(Copy, Clone, Debug)]
struct PlaygroundProject;

impl Linter for PlaygroundProject {
    fn name(&self) -> &'static str {
        "playground-project"
    }
}

impl ProjectLinter for PlaygroundProject {
    fn run<'l>(
        &self,
        ctx: &ProjectContext<'l>,
        out: &mut LintFormatter<'l, '_>,
    ) -> Result<RunStatus<'l>> {
        // --- ADD PLAYGROUND CODE HERE ---

        Ok(RunStatus::Executed)
    }
}

#[derive(Copy, Clone, Debug)]
struct PlaygroundPackage;

impl Linter for PlaygroundPackage {
    fn name(&self) -> &'static str {
        "playground-package"
    }
}

impl PackageLinter for PlaygroundPackage {
    fn run<'l>(
        &self,
        ctx: &PackageContext<'l>,
        out: &mut LintFormatter<'l, '_>,
    ) -> Result<RunStatus<'l>> {
        // --- ADD PLAYGROUND CODE HERE ---

        Ok(RunStatus::Executed)
    }
}

#[derive(Copy, Clone, Debug)]
struct PlaygroundFilePath;

impl Linter for PlaygroundFilePath {
    fn name(&self) -> &'static str {
        "playground-file-path"
    }
}

impl FilePathLinter for PlaygroundFilePath {
    fn run<'l>(
        &self,
        ctx: &FilePathContext<'l>,
        out: &mut LintFormatter<'l, '_>,
    ) -> Result<RunStatus<'l>> {
        // --- ADD PLAYGROUND CODE HERE ---

        Ok(RunStatus::Executed)
    }
}

#[derive(Copy, Clone, Debug)]
struct PlaygroundContent;

impl Linter for PlaygroundContent {
    fn name(&self) -> &'static str {
        "playground-content"
    }
}

impl ContentLinter for PlaygroundContent {
    fn run<'l>(
        &self,
        ctx: &ContentContext<'l>,
        out: &mut LintFormatter<'l, '_>,
    ) -> Result<RunStatus<'l>> {
        // --- ADD PLAYGROUND CODE HERE ---

        Ok(RunStatus::Executed)
    }
}

// ---

#[derive(Debug, StructOpt)]
pub struct Args {
    /// Dummy arg that doesn't do anything
    #[structopt(long)]
    #[allow(dead_code)]
    dummy: bool,
}

pub fn run(args: Args) -> crate::Result<()> {
    let nexlint_context = NexLintContext::from_current_dir()?;
    let engine = LintEngineConfig::new(&nexlint_context)
        .with_project_linters(&[&PlaygroundProject])
        .with_package_linters(&[&PlaygroundPackage])
        .with_file_path_linters(&[&PlaygroundFilePath])
        .with_content_linters(&[&PlaygroundContent])
        .build();

    let results = engine.run()?;

    handle_lint_results(results)
}
