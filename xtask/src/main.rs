mod documentation;

use std::{
    env, fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

const REQUIRED_FILES: &[&str] = &[
    ".cargo/config.toml",
    ".github/workflows/validation.yml",
    ".gitignore",
    "AGENTS.md",
    "ARCHITECTURE.md",
    "Cargo.lock",
    "Cargo.toml",
    "LICENSE",
    "LICENSING.md",
    "README.md",
    "ROADMAP.md",
    "STATUS.md",
    "TESTING.md",
    "rust-toolchain.toml",
    "spec/README.md",
    "spec/semantic-model.md",
    "src/lib.rs",
    "xtask/Cargo.toml",
    "xtask/src/documentation.rs",
    "xtask/src/main.rs",
];

const ACTIVE_IDENTITY_FILES: &[&str] = &[
    ".github/workflows/validation.yml",
    "AGENTS.md",
    "ARCHITECTURE.md",
    "Cargo.toml",
    "README.md",
    "ROADMAP.md",
    "STATUS.md",
    "TESTING.md",
    "spec/README.md",
    "spec/semantic-model.md",
    "src/lib.rs",
];

const GPL3_LICENSE_BLOB_SHA: &str = "f288702d2fa16d3cdf0035b15a9fcbc552cd88e7";

fn main() {
    let mut arguments = env::args().skip(1);
    let result = match (arguments.next().as_deref(), arguments.next()) {
        (Some("validate"), None) => validate(),
        _ => Err("usage: cargo validate".to_owned()),
    };

    if let Err(error) = result {
        eprintln!("validation failed: {error}");
        std::process::exit(1);
    }
}

fn validate() -> Result<(), String> {
    let root = repository_root()?;
    let initial_state = git_status(&root)?;

    validate_required_files(&root)?;
    validate_removed_template_authority(&root)?;
    validate_repository_identity(&root)?;
    validate_license_representation(&root)?;
    validate_workflow_pin(&root)?;
    documentation::validate(&root)?;

    run(
        &root,
        "cargo",
        &["metadata", "--format-version", "1", "--locked", "--no-deps"],
    )?;
    run(&root, "cargo", &["fmt", "--all", "--", "--check"])?;
    run(
        &root,
        "cargo",
        &["test", "--workspace", "--all-targets", "--locked"],
    )?;
    run(
        &root,
        "cargo",
        &[
            "clippy",
            "--workspace",
            "--all-targets",
            "--locked",
            "--",
            "-D",
            "warnings",
        ],
    )?;
    run_with_env(
        &root,
        "cargo",
        &["doc", "--workspace", "--no-deps", "--locked"],
        &[("RUSTDOCFLAGS", "-D warnings")],
    )?;
    run(&root, "git", &["diff", "--check"])?;
    run(&root, "git", &["diff", "--cached", "--check"])?;

    let final_state = git_status(&root)?;
    if final_state != initial_state {
        return Err(format!(
            "validation changed repository state:\nbefore:\n{initial_state}after:\n{final_state}"
        ));
    }

    println!("RunenShader repository validation passed");
    Ok(())
}

fn repository_root() -> Result<PathBuf, String> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| "xtask manifest must live at <repository>/xtask".to_owned())
}

fn validate_required_files(root: &Path) -> Result<(), String> {
    for relative_path in REQUIRED_FILES {
        let path = root.join(relative_path);
        if !path.is_file() {
            return Err(format!("required file is missing: {relative_path}"));
        }
    }
    Ok(())
}

fn validate_removed_template_authority(root: &Path) -> Result<(), String> {
    if root.join("BOOTSTRAP.md").exists() {
        return Err("retired template authority must not exist: BOOTSTRAP.md".to_owned());
    }
    Ok(())
}

fn validate_repository_identity(root: &Path) -> Result<(), String> {
    const FORBIDDEN: &[&str] = &["rust-framework-template", "Rust Framework Template"];

    for relative_path in ACTIVE_IDENTITY_FILES {
        let content = read_utf8(root, relative_path)?;
        for marker in FORBIDDEN {
            if content.contains(marker) {
                return Err(format!(
                    "active RunenShader surface retains template identity {marker:?}: {relative_path}"
                ));
            }
        }
        if content.contains("Apache-2.0") {
            return Err(format!(
                "active RunenShader surface retains Apache product-license claim: {relative_path}"
            ));
        }
    }

    let cargo = read_utf8(root, "Cargo.toml")?;
    for required in [
        "name = \"runen-shader\"",
        "repository = \"https://github.com/dornglut/runen-shader\"",
        "license = \"GPL-3.0-only\"",
    ] {
        if !cargo.contains(required) {
            return Err(format!("Cargo.toml is missing accepted identity field: {required}"));
        }
    }
    if cargo.contains("rust-version") {
        return Err(
            "Cargo.toml must not claim an MSRV until concrete RunenShader evidence accepts one"
                .to_owned(),
        );
    }

    let lock = read_utf8(root, "Cargo.lock")?;
    if !lock.contains("name = \"runen-shader\"") || lock.contains("rust-framework-template") {
        return Err("Cargo.lock does not match the RunenShader package identity".to_owned());
    }

    Ok(())
}

fn validate_license_representation(root: &Path) -> Result<(), String> {
    let license_sha = output(root, "git", &["hash-object", "LICENSE"])?;
    if license_sha.trim() != GPL3_LICENSE_BLOB_SHA {
        return Err("LICENSE does not match the accepted complete GPLv3 text".to_owned());
    }

    let licensing = read_utf8(root, "LICENSING.md")?;
    if !licensing.contains("GPL-3.0-only")
        || !licensing.contains("commercial license")
        || !licensing.contains("Historical template grant")
    {
        return Err("LICENSING.md is missing required current/historical licensing context".to_owned());
    }

    let readme = read_utf8(root, "README.md")?;
    for required in ["[GPL-3.0-only](LICENSE)", "[LICENSING.md](LICENSING.md)"] {
        if !readme.contains(required) {
            return Err(format!("README.md is missing license link: {required}"));
        }
    }

    Ok(())
}

fn validate_workflow_pin(root: &Path) -> Result<(), String> {
    const PREFIX: &str =
        "uses: dornglut/github-workflows/.github/workflows/reusable-rust-cargo-validate.yml@";
    let workflow = read_utf8(root, ".github/workflows/validation.yml")?;
    let revision = workflow
        .lines()
        .map(str::trim)
        .find_map(|line| line.strip_prefix(PREFIX))
        .ok_or_else(|| "validation workflow does not call the accepted reusable workflow".to_owned())?;

    if revision.len() != 40 || !revision.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err("validation workflow must pin the reusable workflow to a full commit SHA".to_owned());
    }

    Ok(())
}

fn read_utf8(root: &Path, relative_path: &str) -> Result<String, String> {
    fs::read_to_string(root.join(relative_path))
        .map_err(|error| format!("failed to read {relative_path} as UTF-8: {error}"))
}

fn git_status(root: &Path) -> Result<String, String> {
    output(
        root,
        "git",
        &["status", "--porcelain", "--untracked-files=all"],
    )
}

fn run(root: &Path, program: &str, args: &[&str]) -> Result<(), String> {
    run_with_env(root, program, args, &[])
}

fn run_with_env(
    root: &Path,
    program: &str,
    args: &[&str],
    environment: &[(&str, &str)],
) -> Result<(), String> {
    let mut command = Command::new(program);
    command
        .args(args)
        .current_dir(root)
        .envs(environment.iter().copied())
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    let status = command
        .status()
        .map_err(|error| format!("failed to execute {program}: {error}"))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("{program} {} exited with {status}", args.join(" ")))
    }
}

fn output(root: &Path, program: &str, args: &[&str]) -> Result<String, String> {
    let result = Command::new(program)
        .args(args)
        .current_dir(root)
        .output()
        .map_err(|error| format!("failed to execute {program}: {error}"))?;

    if !result.status.success() {
        return Err(format!(
            "{program} {} exited with {}:\n{}",
            args.join(" "),
            result.status,
            String::from_utf8_lossy(&result.stderr)
        ));
    }

    String::from_utf8(result.stdout)
        .map_err(|error| format!("{program} produced invalid UTF-8: {error}"))
}
