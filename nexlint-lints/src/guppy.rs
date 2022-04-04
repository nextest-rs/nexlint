//! Project and package linters that run queries on guppy.

use guppy::{graph::PackagePublish, Version, VersionReq};
use nexlint::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

/// Ban certain crates from being used as direct dependencies or in the default build.
#[derive(Debug)]
pub struct BannedDeps<'cfg> {
    config: &'cfg BannedDepsConfig,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct BannedDepsConfig {
    /// Banned direct dependencies
    pub direct: HashMap<String, String>,
}

impl<'cfg> BannedDeps<'cfg> {
    pub fn new(config: &'cfg BannedDepsConfig) -> Self {
        Self { config }
    }
}

impl<'cfg> Linter for BannedDeps<'cfg> {
    fn name(&self) -> &'static str {
        "banned-deps"
    }
}

impl<'cfg> ProjectLinter for BannedDeps<'cfg> {
    fn run<'l>(
        &self,
        ctx: &ProjectContext<'l>,
        out: &mut LintFormatter<'l, '_>,
    ) -> Result<RunStatus<'l>> {
        let package_graph = ctx.package_graph()?;

        let filter_ban = |banned: &'cfg HashMap<String, String>| {
            package_graph.packages().filter_map(move |package| {
                banned
                    .get(package.name())
                    .map(move |message| (package, message))
            })
        };

        let banned_direct = &self.config.direct;
        for (package, message) in filter_ban(banned_direct) {
            // Look at the reverse direct dependencies of this package.
            for link in package.reverse_direct_links() {
                let from = link.from();
                if let Some(workspace_path) = from.source().workspace_path() {
                    out.write_kind(
                        LintKind::Package {
                            name: from.name(),
                            workspace_path,
                        },
                        LintLevel::Error,
                        format!("banned direct dependency '{}': {}", package.name(), message),
                    );
                }
            }
        }

        Ok(RunStatus::Executed)
    }
}

/// Enforce attributes on workspace crates.
#[derive(Debug)]
pub struct EnforcedAttributes<'cfg> {
    config: &'cfg EnforcedAttributesConfig,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct EnforcedAttributesConfig {
    /// Ensure the authors of every workspace crate are set to this.
    pub authors: Option<Vec<String>>,
    /// Ensure the `license` field of every workspace crate is set to this.
    pub license: Option<String>,
}

impl<'cfg> EnforcedAttributes<'cfg> {
    pub fn new(config: &'cfg EnforcedAttributesConfig) -> Self {
        Self { config }
    }
}

impl<'cfg> Linter for EnforcedAttributes<'cfg> {
    fn name(&self) -> &'static str {
        "enforced-attributes"
    }
}

impl<'cfg> PackageLinter for EnforcedAttributes<'cfg> {
    fn run<'l>(
        &self,
        ctx: &PackageContext<'l>,
        out: &mut LintFormatter<'l, '_>,
    ) -> Result<RunStatus<'l>> {
        let metadata = ctx.metadata();
        if let Some(authors) = &self.config.authors {
            if metadata.authors() != authors.as_slice() {
                out.write(
                    LintLevel::Error,
                    format!("invalid authors (expected {:?})", authors.join(", "),),
                );
            }
        }
        if let Some(license) = &self.config.license {
            if metadata.license() != Some(license.as_str()) {
                out.write(
                    LintLevel::Error,
                    format!("invalid license (expected {})", license),
                )
            }
        }

        Ok(RunStatus::Executed)
    }
}

/// Check conventions in crate names and paths.
#[derive(Debug)]
pub struct CrateNamesPaths;

impl Linter for CrateNamesPaths {
    fn name(&self) -> &'static str {
        "crate-names-paths"
    }
}

impl PackageLinter for CrateNamesPaths {
    fn run<'l>(
        &self,
        ctx: &PackageContext<'l>,
        out: &mut LintFormatter<'l, '_>,
    ) -> Result<RunStatus<'l>> {
        let name = ctx.metadata().name();
        if name.contains('_') {
            out.write(
                LintLevel::Error,
                "crate name contains '_' (use '-' instead)",
            );
        }

        let workspace_path = ctx.workspace_path();
        if workspace_path.as_str().contains('_') {
            out.write(
                LintLevel::Error,
                "workspace path contains '_' (use '-' instead)",
            );
        }

        for build_target in ctx.metadata().build_targets() {
            let target_name = build_target.name();
            if target_name.contains('_') {
                // If the path is implicitly specified by the name, don't warn about it.
                let file_stem = build_target.path().file_stem();
                if file_stem != Some(target_name) {
                    out.write(
                        LintLevel::Error,
                        format!(
                            "build target '{}' contains '_' (use '-' instead)",
                            target_name
                        ),
                    );
                }
            }
        }

        Ok(RunStatus::Executed)
    }
}

/// Ensure that any workspace packages with build dependencies also have a build script.
#[derive(Debug)]
pub struct IrrelevantBuildDeps;

impl Linter for IrrelevantBuildDeps {
    fn name(&self) -> &'static str {
        "irrelevant-build-deps"
    }
}

impl PackageLinter for IrrelevantBuildDeps {
    fn run<'l>(
        &self,
        ctx: &PackageContext<'l>,
        out: &mut LintFormatter<'l, '_>,
    ) -> Result<RunStatus<'l>> {
        let metadata = ctx.metadata();

        let has_build_dep = metadata
            .direct_links()
            .any(|link| link.build().is_present());

        if !metadata.has_build_script() && has_build_dep {
            out.write(LintLevel::Error, "build dependencies but no build script");
        }

        Ok(RunStatus::Executed)
    }
}

/// Ensure that packages within the workspace only depend on one version of a third-party crate.
#[derive(Debug)]
pub struct DirectDepDups<'cfg> {
    config: &'cfg DirectDepDupsConfig,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct DirectDepDupsConfig {
    pub allow: Vec<String>,
}

impl<'cfg> DirectDepDups<'cfg> {
    pub fn new(config: &'cfg DirectDepDupsConfig) -> crate::Result<Self> {
        Ok(Self { config })
    }
}

impl<'cfg> Linter for DirectDepDups<'cfg> {
    fn name(&self) -> &'static str {
        "direct-dep-dups"
    }
}

impl<'cfg> ProjectLinter for DirectDepDups<'cfg> {
    fn run<'l>(
        &self,
        ctx: &ProjectContext<'l>,
        out: &mut LintFormatter<'l, '_>,
    ) -> Result<RunStatus<'l>> {
        let package_graph = ctx.package_graph()?;

        // This is a map of direct deps by name -> version -> packages that depend on it.
        let mut direct_deps: BTreeMap<&str, BTreeMap<&Version, Vec<&str>>> = BTreeMap::new();
        package_graph.query_workspace().resolve_with_fn(|_, link| {
            // Collect direct dependencies of workspace packages.
            let (from, to) = link.endpoints();
            if from.in_workspace() && !to.in_workspace() {
                direct_deps
                    .entry(to.name())
                    .or_default()
                    .entry(to.version())
                    .or_default()
                    .push(from.name());
            }
            // query_workspace + preventing further traversals will mean that only direct
            // dependencies are considered.
            false
        });
        for (direct_dep, versions) in direct_deps
            .iter()
            .filter(|(d, _)| !self.config.allow.contains(&d.to_string()))
        {
            if versions.len() > 1 {
                let mut msg = format!("duplicate direct dependency '{}':\n", direct_dep);
                for (version, packages) in versions {
                    msg.push_str(&format!("  * {} (", version));
                    msg.push_str(&packages.join(", "));
                    msg.push_str(")\n");
                }
                out.write(LintLevel::Error, msg);
            }
        }

        Ok(RunStatus::Executed)
    }
}

/// Ensure that all unpublished packages only use path dependencies for workspace dependencies
#[derive(Debug)]
pub struct UnpublishedPackagesOnlyUsePathDependencies {
    no_version_req: VersionReq,
}

impl UnpublishedPackagesOnlyUsePathDependencies {
    pub fn new() -> Self {
        Self {
            no_version_req: VersionReq::parse("*").expect("* should be a valid req"),
        }
    }
}

impl Linter for UnpublishedPackagesOnlyUsePathDependencies {
    fn name(&self) -> &'static str {
        "unpublished-packages-only-use-path-dependencies"
    }
}

impl PackageLinter for UnpublishedPackagesOnlyUsePathDependencies {
    fn run<'l>(
        &self,
        ctx: &PackageContext<'l>,
        out: &mut LintFormatter<'l, '_>,
    ) -> Result<RunStatus<'l>> {
        let metadata = ctx.metadata();

        // Skip all packages which aren't 'publish = false'
        if !metadata.publish().is_never() {
            return Ok(RunStatus::Executed);
        }

        for direct_dep in metadata.direct_links() {
            if direct_dep.version_req() != &self.no_version_req {
                let msg = format!(
                    "unpublished package specifies a version of first-party dependency '{}' ({}); \
                    unpublished packages should only use path dependencies for first-party packages.",
                    direct_dep.dep_name(),
                    direct_dep.version_req(),
                );
                out.write(LintLevel::Error, msg);
            }
        }

        Ok(RunStatus::Executed)
    }
}

/// Ensure that all published packages only depend on other, published packages
#[derive(Debug)]
pub struct PublishedPackagesDontDependOnUnpublishedPackages;

impl Linter for PublishedPackagesDontDependOnUnpublishedPackages {
    fn name(&self) -> &'static str {
        "published-packages-dont-depend-on-unpublished-packages"
    }
}

impl PackageLinter for PublishedPackagesDontDependOnUnpublishedPackages {
    fn run<'l>(
        &self,
        ctx: &PackageContext<'l>,
        out: &mut LintFormatter<'l, '_>,
    ) -> Result<RunStatus<'l>> {
        let metadata = ctx.metadata();

        // Skip all packages which aren't publishable
        if metadata.publish().is_never() {
            return Ok(RunStatus::Executed);
        }

        for direct_dep in metadata.direct_links() {
            // If the direct dependency isn't publishable
            if direct_dep.to().publish().is_never() {
                out.write(
                    LintLevel::Error,
                    format!(
                        "published package can't depend on unpublished package '{}'",
                        direct_dep.dep_name()
                    ),
                );
            }
        }

        Ok(RunStatus::Executed)
    }
}

/// Only allow crates to be published to crates.io
#[derive(Debug)]
pub struct OnlyPublishToCratesIo;

impl Linter for OnlyPublishToCratesIo {
    fn name(&self) -> &'static str {
        "only-publish-to-crates-io"
    }
}

impl PackageLinter for OnlyPublishToCratesIo {
    fn run<'l>(
        &self,
        ctx: &PackageContext<'l>,
        out: &mut LintFormatter<'l, '_>,
    ) -> Result<RunStatus<'l>> {
        let metadata = ctx.metadata();

        let is_ok = match metadata.publish() {
            PackagePublish::Unrestricted => false,
            PackagePublish::Registries(&[ref registry]) => registry == PackagePublish::CRATES_IO,
            // Unpublished package.
            PackagePublish::Registries(&[]) => true,
            // Multiple registries or something else.
            _ => false,
        };

        if !is_ok {
            out.write(
                LintLevel::Error,
                "published package should only be publishable to crates.io. \
                    If you intend to publish this package, ensure the 'publish' \
                    field in the package's Cargo.toml is 'publish = [\"crates-io\"]. \
                    Otherwise set the 'publish' field to 'publish = false'.",
            );
        }

        Ok(RunStatus::Executed)
    }
}

/// Crates in the `/crates` directory have a flatten structure and their directory name is the same
/// as the crate name
#[derive(Debug)]
pub struct CratesInCratesDirectory;

impl Linter for CratesInCratesDirectory {
    fn name(&self) -> &'static str {
        "only-publish-to-crates-io"
    }
}

impl PackageLinter for CratesInCratesDirectory {
    fn run<'l>(
        &self,
        ctx: &PackageContext<'l>,
        out: &mut LintFormatter<'l, '_>,
    ) -> Result<RunStatus<'l>> {
        let mut path_components = ctx.workspace_path().components();
        match path_components.next().map(|p| p.as_str()) {
            Some("crates") => {}
            _ => return Ok(RunStatus::Executed),
        }

        match path_components.next().map(|p| p.as_str()) {
            Some(directory) if directory == ctx.metadata().name() => {}
            _ => {
                out.write(
                    LintLevel::Error,
                    "crates in the `crates/` directory must be in a directory with the same name as the crate",
                );
            }
        }

        if path_components.next().is_some() {
            out.write(
                    LintLevel::Error,
                    "crates in the `crates/` directory must be in a flat directory structure, no nesting",
                );
        }

        Ok(RunStatus::Executed)
    }
}
