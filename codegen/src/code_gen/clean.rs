//! Removing a previously generated output tree.
//!
//! Generation writes `<output>/<package>/<Type>.<ext>` and overwrites in place.
//! It never removes anything, so a type that is renamed or moves package leaves
//! its old file behind — stale code that still compiles and that a
//! `git diff`-based drift check can never flag, because a file nothing
//! regenerates never changes. Cleaning before generating makes the output tree
//! a pure function of the schema again.

use std::path::{Component, Path};

use anyhow::{bail, Result};

use super::fs::FileSystem;

/// Recursively delete `output_dir`. A missing directory is a no-op, so this is
/// safe to run before the first generate.
///
/// Refuses paths whose blast radius is obviously wrong — the filesystem root,
/// a bare relative `.`, or anything reachable by climbing out with `..`. This
/// deletes a whole tree, so the mistakes worth catching are the catastrophic
/// ones, not the merely surprising.
pub fn clean_output_dir(fs: &dyn FileSystem, output_dir: &str) -> Result<()> {
    guard_output_dir(output_dir)?;
    fs.remove_dir_all(output_dir)
}

fn guard_output_dir(output_dir: &str) -> Result<()> {
    let trimmed = output_dir.trim();
    if trimmed.is_empty() {
        bail!("refusing to clean an empty output path");
    }

    let path = Path::new(trimmed);
    let mut components = path.components().peekable();

    // A path made only of roots and `.` names is the root or the current
    // directory — never a generated tree.
    let names = path
        .components()
        .filter(|c| matches!(c, Component::Normal(_)))
        .count();
    if names == 0 {
        bail!(
            "refusing to clean {trimmed:?}: it is the filesystem root or the \
             current directory, not a generated output tree"
        );
    }

    if components.any(|c| matches!(c, Component::ParentDir)) {
        bail!("refusing to clean {trimmed:?}: `..` escapes the output tree");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::code_gen::fs::MemoryFileSystem;

    #[test]
    fn removes_the_tree_and_nothing_beside_it() {
        let fs = Arc::new(MemoryFileSystem::new());
        fs.write_file("/out/pkg/A.ts", b"a").unwrap();
        fs.write_file("/out/pkg/nested/B.ts", b"b").unwrap();
        fs.write_file("/out-other/C.ts", b"c").unwrap();
        fs.write_file("/keep.ts", b"k").unwrap();

        clean_output_dir(fs.as_ref(), "/out").unwrap();

        let remaining: Vec<String> = fs.files().keys().cloned().collect();
        assert_eq!(remaining.len(), 2, "remaining: {remaining:?}");
        assert!(fs.exists("/out-other/C.ts"), "sibling prefix must survive");
        assert!(fs.exists("/keep.ts"));
    }

    #[test]
    fn a_missing_directory_is_not_an_error() {
        let fs = MemoryFileSystem::new();
        clean_output_dir(&fs, "/never/existed").unwrap();
    }

    #[test]
    fn refuses_paths_with_a_catastrophic_blast_radius() {
        for path in ["", "   ", "/", ".", "./", "../generated", "src/../.."] {
            assert!(
                guard_output_dir(path).is_err(),
                "should have refused {path:?}"
            );
        }
    }

    #[test]
    fn accepts_ordinary_output_paths() {
        for path in ["src/generated", "./src/generated", "/abs/out", "out"] {
            guard_output_dir(path).unwrap_or_else(|e| panic!("rejected {path:?}: {e}"));
        }
    }
}
