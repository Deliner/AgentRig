use super::Python;
use crate::shared::testing::write;
use crate::structural_lint::architecture::extraction::{self, Target};
use anyhow::{Result, ensure};
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
    process::Command,
};

fn files(paths: &[&str]) -> BTreeSet<PathBuf> {
    paths.iter().map(PathBuf::from).collect()
}

fn resolve(paths: &[&str], source: &str, code: &str) -> Result<BTreeSet<PathBuf>> {
    let inventory = files(paths);
    let external = BTreeSet::new();
    let resolver = Python::new(Path::new("src"), &inventory, &external)?;
    let references = extraction::extract(Path::new(source), code)?;
    ensure!(references.issues.is_empty(), "unexpected syntax issue");
    let mut targets = BTreeSet::new();
    for reference in references.items {
        let resolved = resolver.resolve(Path::new(source), &reference.target)?;
        targets.extend(resolved.files);
        targets.extend(resolved.initializers);
    }
    Ok(targets)
}

#[test]
fn imports_include_every_regular_package_initializer() {
    let paths = [
        "src/main.py",
        "src/pkg/__init__.py",
        "src/pkg/api/__init__.py",
        "src/pkg/api/model.py",
    ];
    let result = resolve(&paths, paths[0], "import pkg.api.model as model\n").unwrap();
    assert_eq!(result, files(&paths[1..]));
    let result = resolve(&paths, paths[0], "from pkg.api.model import Value as V\n").unwrap();
    assert_eq!(result, files(&paths[1..]));
}

// DECISION: D227
#[test]
fn the_packages_an_import_passes_are_initializers_and_the_imported_module_is_its_target() {
    let paths = [
        "src/main.py",
        "src/pkg/__init__.py",
        "src/pkg/api/__init__.py",
        "src/pkg/api/model.py",
    ];
    let inventory = files(&paths);
    let external = BTreeSet::new();
    let resolver = Python::new(Path::new("src"), &inventory, &external).unwrap();
    let source = Path::new(paths[0]);
    let module = Target::PythonModule("pkg.api.model".into());
    let resolved = resolver.resolve(source, &module).unwrap();
    assert_eq!(resolved.files, files(&paths[3..]));
    assert_eq!(resolved.initializers, files(&paths[1..3]));
    let names = vec!["Value".to_owned()];
    let attribute = Target::PythonFrom {
        module: "pkg.api".into(),
        names,
    };
    let resolved = resolver.resolve(source, &attribute).unwrap();
    assert_eq!(resolved.files, files(&paths[2..3]));
    assert_eq!(resolved.initializers, files(&paths[1..2]));
}

#[test]
fn relative_imports_resolve_from_module_or_initializer_package() {
    let paths = [
        "src/pkg/service/main.py",
        "src/pkg/service/__init__.py",
        "src/pkg/__init__.py",
        "src/pkg/api.py",
    ];
    let expected = files(&[paths[2], paths[3]]);
    assert_eq!(
        resolve(&paths, paths[0], "from ..api import Value\n").unwrap(),
        expected
    );
    assert_eq!(
        resolve(&paths, paths[1], "from ..api import Value\n").unwrap(),
        expected
    );
    assert!(
        resolve(&paths, paths[0], "from ...api import Value\n")
            .unwrap_err()
            .to_string()
            .contains("escapes")
    );
}

#[test]
fn namespace_packages_allow_concrete_submodules_but_not_empty_proof() {
    let paths = ["src/main.py", "src/team/api.py", "src/team/nested/model.py"];
    assert_eq!(
        resolve(&paths, paths[0], "from team import api\n").unwrap(),
        files(&[paths[1]])
    );
    assert_eq!(
        resolve(&paths, paths[0], "import team.nested.model\n").unwrap(),
        files(&[paths[2]])
    );
    assert!(
        resolve(&paths, paths[0], "import team\n")
            .unwrap_err()
            .to_string()
            .contains("namespace-only")
    );
    assert!(
        resolve(&paths, paths[0], "from team import absent\n")
            .unwrap_err()
            .to_string()
            .contains("unresolved namespace member")
    );
}

#[test]
fn package_precedes_module_and_module_precedes_namespace() {
    let paths = [
        "src/main.py",
        "src/pkg.py",
        "src/pkg/__init__.py",
        "src/pkg/api.py",
    ];
    assert_eq!(
        resolve(&paths, paths[0], "import pkg\n").unwrap(),
        files(&[paths[2]])
    );
    let paths = ["src/main.py", "src/pkg.py", "src/pkg/api.py"];
    assert_eq!(
        resolve(&paths, paths[0], "import pkg\n").unwrap(),
        files(&[paths[1]])
    );
    assert!(
        resolve(&paths, paths[0], "import pkg.api\n")
            .unwrap_err()
            .to_string()
            .contains("non-package")
    );
}

#[test]
fn unresolved_modules_are_not_assumed_external_and_local_wins() {
    let inventory = files(&["src/main.py", "src/vendor/local.py"]);
    let external = BTreeSet::from(["vendor".into(), "typing".into()]);
    let resolver = Python::new(Path::new("src"), &inventory, &external).unwrap();
    let source = Path::new("src/main.py");
    let resolved = resolver
        .resolve(source, &Target::PythonModule("typing".into()))
        .unwrap();
    assert_eq!(resolved.external.as_deref(), Some("typing"));
    assert!(resolved.files.is_empty());
    assert!(
        resolver
            .resolve(source, &Target::PythonModule("unknown".into()))
            .is_err()
    );
    assert!(
        resolver
            .resolve(source, &Target::PythonModule("vendor.missing".into()))
            .is_err()
    );
    let local = resolver
        .resolve(source, &Target::PythonModule("vendor.local".into()))
        .unwrap();
    assert_eq!(local.files, files(&["src/vendor/local.py"]));
    assert!(local.external.is_none());
}

#[test]
fn package_member_ambiguity_and_wildcards_require_export_analysis() {
    let paths = ["src/main.py", "src/pkg/__init__.py", "src/pkg/api.py"];
    let ambiguous = resolve(&paths, paths[0], "from pkg import api\n").unwrap_err();
    assert!(
        ambiguous
            .to_string()
            .contains("initializer exports require analysis")
    );
    let wildcard = resolve(&paths, paths[0], "from pkg import *\n").unwrap_err();
    assert!(wildcard.to_string().contains("__all__"));
    assert_eq!(
        resolve(&paths, paths[0], "from pkg import Value\n").unwrap(),
        files(&[paths[1]])
    );
}

#[test]
fn roots_names_missing_sources_and_stub_only_targets_are_explicit() {
    let inventory = files(&["main.py", "src/main.py", "src/api.pyi"]);
    let external = BTreeSet::new();
    assert!(Python::new(Path::new("../src"), &inventory, &external).is_err());
    assert!(Python::new(Path::new("/src"), &inventory, &external).is_err());
    let resolver = Python::new(Path::new("src"), &inventory, &external).unwrap();
    assert!(
        resolver
            .resolve(Path::new("main.py"), &Target::PythonModule("api".into()))
            .is_err()
    );
    assert!(
        resolver
            .resolve(
                Path::new("src/missing.py"),
                &Target::PythonModule("api".into())
            )
            .is_err()
    );
    assert!(
        resolver
            .resolve(
                Path::new("src/main.py"),
                &Target::PythonModule("api".into())
            )
            .is_err()
    );
    let resolver = Python::new(Path::new("."), &inventory, &external).unwrap();
    assert_eq!(
        resolver
            .resolve(Path::new("main.py"), &Target::PythonModule("main".into()))
            .unwrap()
            .files,
        files(&["main.py"])
    );
}

#[test]
fn invalid_module_names_are_rejected() {
    let inventory = files(&["main.py"]);
    for name in ["", "../api", "a..b", "a/b", "a-b", "a\\b"] {
        let invalid = BTreeSet::from([name.to_owned()]);
        assert!(
            Python::new(Path::new("."), &inventory, &invalid).is_err(),
            "{name}"
        );
    }
}

#[test]
fn resolved_files_agree_with_real_python_imports() {
    let directory = tempfile::tempdir().unwrap();
    let paths = [
        "src/pkg/__init__.py",
        "src/pkg/api/__init__.py",
        "src/pkg/api/model.py",
        "src/main.py",
    ];
    for path in paths {
        write(directory.path(), path, "");
    }
    let expected = resolve(&paths, "src/main.py", "import pkg.api.model\n").unwrap();
    let script = "import sys,json,pathlib; root=pathlib.Path(sys.argv[1]); sys.path.insert(0,str(root/'src')); import pkg.api.model; print(json.dumps(sorted(str(pathlib.Path(m.__file__).relative_to(root)) for n,m in sys.modules.items() if n=='pkg' or n.startswith('pkg.'))))";
    let output = Command::new("python3")
        .args(["-I", "-B", "-c", script])
        .arg(directory.path())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let actual: BTreeSet<PathBuf> = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(actual, expected);
    assert!(!directory.path().join("src/pkg/__pycache__").exists());
}
