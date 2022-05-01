// Copyright (c) The nextest Contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

use nexlint::{prelude::*, NexLintContext};
use nexlint_lints::{
    content::*,
    file_path::*,
    handle_lint_results,
    package::*,
    project::{DirectDepDups, DirectDepDupsConfig},
};
use structopt::StructOpt;

static LICENSE_HEADER: &str = "\
                               SPDX-License-Identifier: MIT OR Apache-2.0\n\
                               ";

#[derive(Debug, StructOpt)]
pub struct Args {
    #[structopt(long)]
    fail_fast: bool,
}

pub fn run(args: Args) -> crate::Result<()> {
    let direct_dups_config = DirectDepDupsConfig { allow: vec![] };
    let project_linters: &[&dyn ProjectLinter] = &[&DirectDepDups::new(&direct_dups_config)];

    let package_linters: &[&dyn PackageLinter] = &[
        &CrateNamesPaths,
        &IrrelevantBuildDeps,
        &UnpublishedPackagesOnlyUsePathDependencies::new(),
        &PublishedPackagesDontDependOnUnpublishedPackages,
        &OnlyPublishToCratesIo,
        //&CratesInCratesDirectory,
    ];

    let file_path_linters: &[&dyn FilePathLinter] =
        &[&AllowedPaths::new(DEFAULT_ALLOWED_PATHS_REGEX)?];

    let whitespace_exceptions = build_exceptions(&[])?;
    let content_linters: &[&dyn ContentLinter] = &[
        &LicenseHeader::new(LICENSE_HEADER),
        &RootToml,
        &EofNewline::new(&whitespace_exceptions),
        &TrailingWhitespace::new(&whitespace_exceptions),
    ];

    let nexlint_context = NexLintContext::from_current_dir()?;
    let engine = LintEngineConfig::new(&nexlint_context)
        .with_project_linters(project_linters)
        .with_package_linters(package_linters)
        .with_file_path_linters(file_path_linters)
        .with_content_linters(content_linters)
        .fail_fast(args.fail_fast)
        .build();

    let results = engine.run()?;

    handle_lint_results(results)
}
