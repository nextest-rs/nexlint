// Copyright (c) The nextest Contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

use nexlint::prelude::*;
use std::collections::HashSet;

#[derive(Copy, Clone, Debug)]
pub struct LicenseHeader<'a>(&'a str);

impl<'a> LicenseHeader<'a> {
    pub fn new(header: &'a str) -> Self {
        Self(header)
    }
}

impl<'a> Linter for LicenseHeader<'a> {
    fn name(&self) -> &'static str {
        "license-header"
    }
}

impl<'a> ContentLinter for LicenseHeader<'a> {
    fn pre_run<'l>(&self, file_ctx: &FilePathContext<'l>) -> Result<RunStatus<'l>> {
        // TODO: Add a way to pass around state between pre_run and run, so that this computation
        // only needs to be done once.
        match FileType::new(file_ctx) {
            Some(_) => Ok(RunStatus::Executed),
            None => Ok(RunStatus::Skipped(SkipReason::UnsupportedExtension(
                file_ctx.extension(),
            ))),
        }
    }

    fn run<'l>(
        &self,
        ctx: &ContentContext<'l>,
        out: &mut LintFormatter<'l, '_>,
    ) -> Result<RunStatus<'l>> {
        let content = match ctx.content() {
            Some(content) => content,
            None => {
                // This is not a UTF-8 file -- don't analyze it.
                return Ok(RunStatus::Skipped(SkipReason::NonUtf8Content));
            }
        };

        let file_type = FileType::new(ctx.file_ctx()).expect("None filtered out in pre_run");
        // Determine if the file is missing the license header
        let missing_header = match file_type {
            FileType::Rust
            | FileType::Proto
            | FileType::JavaScript
            | FileType::TypeScript
            | FileType::Move => {
                let maybe_license: HashSet<_> = content
                    .lines()
                    .skip_while(|line| line.is_empty())
                    .take(4)
                    .map(|s| s.trim_start_matches("// "))
                    .collect();
                !self
                    .0
                    .lines()
                    .collect::<HashSet<_>>()
                    .is_subset(&maybe_license)
            }
            FileType::Shell | FileType::Python => {
                let maybe_license = content
                    .lines()
                    .skip_while(|line| line.starts_with("#!"))
                    .skip_while(|line| line.is_empty())
                    .take(4)
                    .map(|s| s.trim_start_matches("# "))
                    .collect();
                !self
                    .0
                    .lines()
                    .collect::<HashSet<_>>()
                    .is_subset(&maybe_license)
            }
        };

        if missing_header {
            out.write(LintLevel::Error, "missing license header");
        }

        Ok(RunStatus::Executed)
    }
}

enum FileType {
    Rust,
    Shell,
    Proto,
    JavaScript,
    TypeScript,
    Move,
    Python,
}

impl FileType {
    fn new(ctx: &FilePathContext<'_>) -> Option<Self> {
        match ctx.extension() {
            Some("rs") => Some(FileType::Rust),
            Some("sh") => Some(FileType::Shell),
            Some("proto") => Some(FileType::Proto),
            Some("js") => Some(FileType::JavaScript),
            Some("jsx") => Some(FileType::JavaScript),
            Some("cjs") => Some(FileType::JavaScript),
            Some("mjs") => Some(FileType::JavaScript),
            Some("ts") => Some(FileType::TypeScript),
            Some("tsx") => Some(FileType::TypeScript),
            Some("mts") => Some(FileType::TypeScript),
            Some("cts") => Some(FileType::TypeScript),
            Some("move") => Some(FileType::Move),
            Some("py") => Some(FileType::Python),
            _ => None,
        }
    }
}
