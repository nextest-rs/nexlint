// Copyright (c) The nextest Contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

use anyhow::anyhow;
use nexlint::prelude::LintResults;

pub use anyhow::Result;

mod allowed_paths;
mod guppy;
mod license;
mod toml;
mod whitespace;

pub mod project {
    pub use super::guppy::{BannedDeps, BannedDepsConfig, DirectDepDups, DirectDepDupsConfig};
}

pub mod package {
    pub use super::guppy::{
        CrateNamesPaths, CratesInCratesDirectory, CratesOnlyInCratesDirectory, EnforcedAttributes,
        IrrelevantBuildDeps, OnlyPublishToCratesIo,
        PublishedPackagesDontDependOnUnpublishedPackages,
        UnpublishedPackagesOnlyUsePathDependencies,
    };
}

pub mod file_path {
    pub use super::allowed_paths::{AllowedPaths, DEFAULT_ALLOWED_PATHS_REGEX};
}

pub mod content {
    pub use super::{
        license::LicenseHeader,
        toml::RootToml,
        whitespace::{build_exceptions, EofNewline, TrailingWhitespace},
    };
}

pub fn handle_lint_results(results: LintResults) -> crate::Result<()> {
    // TODO: handle skipped results

    for (source, message) in &results.messages {
        println!(
            "[{}] [{}] [{}]: {}\n",
            message.level(),
            source.name(),
            source.kind(),
            message.message()
        );
    }

    if !results.messages.is_empty() {
        Err(anyhow!("there were lint errors"))
    } else {
        Ok(())
    }
}
