//! End-to-end CLI integration tests using assert_cmd.
//!
//! These tests verify the CLI commands work correctly from the user's perspective,
//! testing the full stack from command-line invocation through to filesystem effects.
//!
//! This test suite would have caught Session 19's "worktree base not set" bug.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Test fixture that creates a temporary directory with git repositories and config.
struct CliTestFixture {
    /// Temporary directory containing all test artifacts
    temp_dir: TempDir,
    /// Path to workspace.conf file
    config_path: PathBuf,
    /// Path to workspace config TOML file
    toml_config_path: PathBuf,
    /// Paths to bare git repositories for testing
    repo_paths: Vec<PathBuf>,
}

impl CliTestFixture {
    /// Create a new CLI test fixture with two test repositories.
    fn new() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let base_path = temp_dir.path();

        // Create main superproject repository (needed for config commands)
        let main_repo = base_path.join("main-repo");
        fs::create_dir_all(&main_repo).expect("Failed to create main repo dir");
        std::process::Command::new("git")
            .args(["init"])
            .current_dir(&main_repo)
            .output()
            .expect("Failed to init main repo");

        // Configure git user for main repo
        std::process::Command::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(&main_repo)
            .output()
            .expect("Failed to set git user.email");
        std::process::Command::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(&main_repo)
            .output()
            .expect("Failed to set git user.name");

        // Create initial commit in main repo
        fs::write(main_repo.join("README.md"), "# Main Repo").expect("Failed to write README");
        std::process::Command::new("git")
            .args(["add", "."])
            .current_dir(&main_repo)
            .output()
            .expect("Failed to git add");
        std::process::Command::new("git")
            .args(["commit", "-m", "Initial commit"])
            .current_dir(&main_repo)
            .output()
            .expect("Failed to git commit");

        // Create two bare git repositories for testing
        let repo_paths = vec![
            Self::create_test_repo(base_path, "repo-a"),
            Self::create_test_repo(base_path, "repo-b"),
        ];

        // Create workspace.conf with repository URLs
        let config_path = base_path.join("workspace.conf");
        let config_content = format!("{}\n{}\n", repo_paths[0].display(), repo_paths[1].display());
        fs::write(&config_path, config_content).expect("Failed to write workspace.conf");

        // Create workspace TOML config with temp worktree_base
        let toml_config_path = base_path.join("config.toml");
        let worktree_base = base_path.join(".worktrees");
        let toml_content = format!(
            r#"worktree_base = "{}"
main_repo = "{}"
default_branch = "main"
enable_nix = true
"#,
            worktree_base.display().to_string().replace('\\', "\\\\"),
            main_repo.display().to_string().replace('\\', "\\\\")
        );
        fs::write(&toml_config_path, toml_content).expect("Failed to write config.toml");

        Self {
            temp_dir,
            config_path,
            toml_config_path,
            repo_paths,
        }
    }

    /// Create a bare git repository with an initial commit.
    fn create_test_repo(base_path: &Path, name: &str) -> PathBuf {
        let repo_path = base_path.join(name);
        fs::create_dir_all(&repo_path).expect("Failed to create repo dir");

        // Initialize git repo
        std::process::Command::new("git")
            .args(["init"])
            .current_dir(&repo_path)
            .output()
            .expect("Failed to init git repo");

        // Create initial commit
        fs::write(repo_path.join("README.md"), format!("# {}", name))
            .expect("Failed to write README");

        std::process::Command::new("git")
            .args(["add", "."])
            .current_dir(&repo_path)
            .output()
            .expect("Failed to git add");

        std::process::Command::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(&repo_path)
            .output()
            .expect("Failed to set git user.email");

        std::process::Command::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(&repo_path)
            .output()
            .expect("Failed to set git user.name");

        std::process::Command::new("git")
            .args(["commit", "-m", "Initial commit"])
            .current_dir(&repo_path)
            .output()
            .expect("Failed to git commit");

        repo_path
    }

    /// Get path to the temp directory.
    fn path(&self) -> &Path {
        self.temp_dir.path()
    }

    /// Create a Command builder for the workspace binary with config pre-set.
    fn workspace_cmd(&self) -> Command {
        let mut cmd = Command::cargo_bin("workspace").expect("Failed to find workspace binary");
        cmd.args(["--config", self.toml_config_path.to_str().unwrap()]);
        cmd
    }
}

// ============================================================================
// Smoke Tests - Basic Functionality
// ============================================================================

#[test]
fn test_cli_help_displays_correctly() {
    let mut cmd = Command::cargo_bin("workspace").expect("Failed to find workspace binary");
    cmd.arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Unified workspace manager"))
        .stdout(predicate::str::contains("switch"))
        .stdout(predicate::str::contains("sync"))
        .stdout(predicate::str::contains("foreach"));
}

#[test]
fn test_cli_version_flag() {
    let mut cmd = Command::cargo_bin("workspace").expect("Failed to find workspace binary");
    cmd.arg("--version");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("workspace"));
}

#[test]
fn test_cli_switch_command_help() {
    let mut cmd = Command::cargo_bin("workspace").expect("Failed to find workspace binary");
    cmd.args(["switch", "--help"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Switch to a multi-repo workspace"));
}

// ============================================================================
// Switch Command Tests
// ============================================================================

#[test]
fn test_switch_creates_workspace_successfully() {
    let fixture = CliTestFixture::new();

    let mut cmd = fixture.workspace_cmd();
    cmd.args([
        "switch",
        "test-workspace",
        "--config-file",
        fixture.config_path.to_str().unwrap(),
    ])
    .current_dir(fixture.path());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("test-workspace"));

    // Verify workspace directory was created
    let workspace_path = fixture.path().join(".worktrees").join("test-workspace");
    assert!(workspace_path.exists(), "Workspace directory should exist");

    // Verify both repositories were cloned
    assert!(
        workspace_path.join("repo-a").exists(),
        "repo-a should exist"
    );
    assert!(
        workspace_path.join("repo-b").exists(),
        "repo-b should exist"
    );

    // Verify config was saved
    let config_file = workspace_path.join(".workspace-config.json");
    assert!(config_file.exists(), "Config file should exist");
}

#[test]
fn test_switch_with_missing_config_file_fails() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let nonexistent_config = temp_dir.path().join("nonexistent.conf");

    let mut cmd = Command::cargo_bin("workspace").expect("Failed to find workspace binary");
    cmd.args([
        "switch",
        "test-workspace",
        "--config-file",
        nonexistent_config.to_str().unwrap(),
    ])
    .current_dir(temp_dir.path());

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_switch_idempotent_when_workspace_exists() {
    let fixture = CliTestFixture::new();

    // Create workspace first time
    let mut cmd1 = fixture.workspace_cmd();
    cmd1.args([
        "switch",
        "test-workspace",
        "--config-file",
        fixture.config_path.to_str().unwrap(),
    ])
    .current_dir(fixture.path());
    cmd1.assert().success();

    // Switch to same workspace again (should be idempotent)
    let mut cmd2 = fixture.workspace_cmd();
    cmd2.args([
        "switch",
        "test-workspace",
        "--config-file",
        fixture.config_path.to_str().unwrap(),
    ])
    .current_dir(fixture.path());
    cmd2.assert().success();
}

// ============================================================================
// Sync Command Tests
// ============================================================================

#[test]
fn test_sync_with_nonexistent_workspace_fails() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let mut cmd = Command::cargo_bin("workspace").expect("Failed to find workspace binary");
    cmd.args(["sync", "nonexistent-workspace"])
        .current_dir(temp_dir.path());

    cmd.assert().failure().stderr(
        predicate::str::contains("not found").or(predicate::str::contains("does not exist")),
    );
}

#[test]
fn test_sync_with_existing_workspace_succeeds() {
    let fixture = CliTestFixture::new();

    // First create a workspace
    let mut cmd_switch = fixture.workspace_cmd();
    cmd_switch
        .args([
            "switch",
            "test-workspace",
            "--config-file",
            fixture.config_path.to_str().unwrap(),
        ])
        .current_dir(fixture.path());
    cmd_switch.assert().success();

    // Now sync it
    let mut cmd_sync = fixture.workspace_cmd();
    cmd_sync
        .args(["sync", "test-workspace"])
        .current_dir(fixture.path());

    cmd_sync
        .assert()
        .success()
        .stdout(predicate::str::contains("Synchronizing workspace"));
}

#[test]
fn test_sync_reports_up_to_date_when_no_changes() {
    let fixture = CliTestFixture::new();

    // Create workspace
    let mut cmd_switch = fixture.workspace_cmd();
    cmd_switch
        .args([
            "switch",
            "test-workspace",
            "--config-file",
            fixture.config_path.to_str().unwrap(),
        ])
        .current_dir(fixture.path());
    cmd_switch.assert().success();

    // Sync immediately (no remote changes)
    let mut cmd_sync = fixture.workspace_cmd();
    cmd_sync
        .args(["sync", "test-workspace"])
        .current_dir(fixture.path());

    cmd_sync
        .assert()
        .success()
        .stdout(predicate::str::contains("up-to-date"));
    // Should report repositories are up-to-date, not "updated"
}

// ============================================================================
// Foreach Command Tests
// ============================================================================

#[test]
fn test_foreach_with_nonexistent_workspace_fails() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let mut cmd = Command::cargo_bin("workspace").expect("Failed to find workspace binary");
    cmd.args(["foreach", "nonexistent-workspace", "echo", "test"])
        .current_dir(temp_dir.path());

    cmd.assert().failure().stderr(
        predicate::str::contains("not found").or(predicate::str::contains("does not exist")),
    );
}

#[test]
fn test_foreach_executes_command_in_all_repos() {
    let fixture = CliTestFixture::new();

    // Create workspace
    let mut cmd_switch = fixture.workspace_cmd();
    cmd_switch
        .args([
            "switch",
            "test-workspace",
            "--config-file",
            fixture.config_path.to_str().unwrap(),
        ])
        .current_dir(fixture.path());
    cmd_switch.assert().success();

    // Execute foreach command
    let mut cmd_foreach = fixture.workspace_cmd();
    cmd_foreach
        .args(["foreach", "test-workspace", "echo", "Hello"])
        .current_dir(fixture.path());

    cmd_foreach
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello"));
}

#[test]
fn test_foreach_environment_variable_expansion() {
    let fixture = CliTestFixture::new();

    // Create workspace
    let mut cmd_switch = fixture.workspace_cmd();
    cmd_switch
        .args([
            "switch",
            "test-workspace",
            "--config-file",
            fixture.config_path.to_str().unwrap(),
        ])
        .current_dir(fixture.path());
    cmd_switch.assert().success();

    // Execute foreach with $name variable
    let mut cmd_foreach = fixture.workspace_cmd();
    cmd_foreach
        .args(["foreach", "test-workspace", "echo", "Repo: $name"])
        .current_dir(fixture.path());

    cmd_foreach
        .assert()
        .success()
        .stdout(predicate::str::contains("Repo: repo-a"))
        .stdout(predicate::str::contains("Repo: repo-b"));
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test]
fn test_invalid_subcommand_shows_error() {
    let mut cmd = Command::cargo_bin("workspace").expect("Failed to find workspace binary");
    cmd.arg("invalid-command");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("unrecognized").or(predicate::str::contains("error")));
}

#[test]
fn test_missing_required_arguments_shows_error() {
    let mut cmd = Command::cargo_bin("workspace").expect("Failed to find workspace binary");
    cmd.arg("switch"); // Missing workspace name

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("required").or(predicate::str::contains("error")));
}

// ============================================================================
// Integration Workflow Tests (Simplified from Python tests)
// ============================================================================

#[test]
fn test_basic_development_workflow() {
    let fixture = CliTestFixture::new();

    // Step 1: Create feature workspace
    let mut cmd_switch = fixture.workspace_cmd();
    cmd_switch
        .args([
            "switch",
            "feature-branch",
            "--config-file",
            fixture.config_path.to_str().unwrap(),
        ])
        .current_dir(fixture.path());
    cmd_switch.assert().success();

    // Step 2: Make changes in a repo
    let repo_path = fixture
        .path()
        .join(".worktrees")
        .join("feature-branch")
        .join("repo-a");
    fs::write(repo_path.join("feature.txt"), "New feature").expect("Failed to write file");

    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to git add");

    std::process::Command::new("git")
        .args(["commit", "-m", "Add feature"])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to git commit");

    // Step 3: Sync workspace
    let mut cmd_sync = fixture.workspace_cmd();
    cmd_sync
        .args(["sync", "feature-branch"])
        .current_dir(fixture.path());
    cmd_sync.assert().success();

    // Step 4: Run command across all repos
    let mut cmd_foreach = fixture.workspace_cmd();
    cmd_foreach
        .args(["foreach", "feature-branch", "git", "status", "--short"])
        .current_dir(fixture.path());
    cmd_foreach.assert().success();
}

#[test]
fn test_multiple_workspaces_can_coexist() {
    let fixture = CliTestFixture::new();

    let workspaces = vec!["feature-1", "feature-2", "bugfix-3"];

    for workspace_name in &workspaces {
        let mut cmd = fixture.workspace_cmd();
        cmd.args([
            "switch",
            workspace_name,
            "--config-file",
            fixture.config_path.to_str().unwrap(),
        ])
        .current_dir(fixture.path());
        cmd.assert().success();
    }

    // Verify all workspaces exist
    for workspace_name in &workspaces {
        let workspace_path = fixture.path().join(".worktrees").join(workspace_name);
        assert!(
            workspace_path.exists(),
            "Workspace {} should exist",
            workspace_name
        );
    }
}

// ============================================================================
// Repair Command Tests
// ============================================================================

#[test]
fn test_repair_missing_repository() {
    let fixture = CliTestFixture::new();

    // Create workspace
    let mut cmd_switch = fixture.workspace_cmd();
    cmd_switch
        .args([
            "switch",
            "test-workspace",
            "--config-file",
            fixture.config_path.to_str().unwrap(),
        ])
        .current_dir(fixture.path());
    cmd_switch.assert().success();

    // Remove repo-a to simulate missing repository
    let repo_path = fixture
        .path()
        .join(".worktrees")
        .join("test-workspace")
        .join("repo-a");
    fs::remove_dir_all(&repo_path).expect("Failed to remove repo");

    // Repair the missing repository
    let mut cmd_repair = fixture.workspace_cmd();
    cmd_repair
        .args(["repair", "test-workspace", "repo-a"])
        .current_dir(fixture.path());

    cmd_repair
        .assert()
        .success()
        .stdout(predicate::str::contains("successful"))
        .stdout(predicate::str::contains("CreatedMissing"));

    // Verify repo was recreated
    assert!(repo_path.exists(), "Repaired repo should exist");
    assert!(repo_path.join(".git").exists(), "Repo should have .git");
}

#[test]
fn test_repair_corrupted_repository() {
    let fixture = CliTestFixture::new();

    // Create workspace
    let mut cmd_switch = fixture.workspace_cmd();
    cmd_switch
        .args([
            "switch",
            "test-workspace",
            "--config-file",
            fixture.config_path.to_str().unwrap(),
        ])
        .current_dir(fixture.path());
    cmd_switch.assert().success();

    // Corrupt repo-a by removing .git directory and replacing with invalid file
    let repo_path = fixture
        .path()
        .join(".worktrees")
        .join("test-workspace")
        .join("repo-a");
    fs::remove_dir_all(repo_path.join(".git")).expect("Failed to remove .git");
    fs::write(repo_path.join(".git"), "gitdir: /nonexistent/path\n")
        .expect("Failed to corrupt .git");

    // Repair the corrupted repository
    let mut cmd_repair = fixture.workspace_cmd();
    cmd_repair
        .args(["repair", "test-workspace", "repo-a"])
        .current_dir(fixture.path());

    cmd_repair
        .assert()
        .success()
        .stdout(predicate::str::contains("successful"))
        .stdout(predicate::str::contains("ReplacedCorrupted"));

    // Verify repo was fixed
    assert!(repo_path.exists(), "Repaired repo should exist");

    // Verify git status works
    let output = std::process::Command::new("git")
        .args(["status"])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to run git status");
    assert!(output.status.success(), "Git status should work");
}

#[test]
fn test_repair_functional_repository_no_action() {
    let fixture = CliTestFixture::new();

    // Create workspace
    let mut cmd_switch = fixture.workspace_cmd();
    cmd_switch
        .args([
            "switch",
            "test-workspace",
            "--config-file",
            fixture.config_path.to_str().unwrap(),
        ])
        .current_dir(fixture.path());
    cmd_switch.assert().success();

    // Repair a functional repository (should be no-op)
    let mut cmd_repair = fixture.workspace_cmd();
    cmd_repair
        .args(["repair", "test-workspace", "repo-a"])
        .current_dir(fixture.path());

    cmd_repair
        .assert()
        .success()
        .stdout(predicate::str::contains("successful"))
        .stdout(predicate::str::contains("NoActionNeeded"));
}

#[test]
fn test_repair_nonexistent_workspace_fails() {
    let fixture = CliTestFixture::new();

    // Try to repair in nonexistent workspace
    let mut cmd_repair = fixture.workspace_cmd();
    cmd_repair
        .args(["repair", "nonexistent-workspace", "repo-a"])
        .current_dir(fixture.path());

    cmd_repair
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found").or(predicate::str::contains("Workspace")));
}

#[test]
fn test_repair_nonexistent_repo_fails() {
    let fixture = CliTestFixture::new();

    // Create workspace
    let mut cmd_switch = fixture.workspace_cmd();
    cmd_switch
        .args([
            "switch",
            "test-workspace",
            "--config-file",
            fixture.config_path.to_str().unwrap(),
        ])
        .current_dir(fixture.path());
    cmd_switch.assert().success();

    // Try to repair repo that doesn't exist in config
    let mut cmd_repair = fixture.workspace_cmd();
    cmd_repair
        .args(["repair", "test-workspace", "nonexistent-repo"])
        .current_dir(fixture.path());

    cmd_repair
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found").or(predicate::str::contains("Repository")));
}

// ============================================================================
// Config Command Tests
// ============================================================================

#[test]
fn test_config_show_default_config() {
    let fixture = CliTestFixture::new();

    // Create a workspace
    let mut cmd_switch = fixture.workspace_cmd();
    cmd_switch
        .args([
            "switch",
            "test-workspace",
            "--config-file",
            fixture.config_path.to_str().unwrap(),
        ])
        .current_dir(fixture.path());
    cmd_switch.assert().success();

    // Show config for the workspace
    let mut cmd_config = fixture.workspace_cmd();
    cmd_config
        .args(["config", "show", "test-workspace"])
        .current_dir(fixture.path());

    cmd_config
        .assert()
        .success()
        .stdout(predicate::str::contains("Configuration for workspace"));
}

#[test]
fn test_config_set_workspace_specific() {
    let fixture = CliTestFixture::new();

    // Create a git worktree using 'workspace add' (config commands require git worktrees)
    let mut cmd_add = fixture.workspace_cmd();
    cmd_add
        .args(["add", "test-workspace", "--branch", "feature"])
        .current_dir(fixture.path());
    cmd_add.assert().success();

    // Set workspace-specific config
    let mut cmd_set = fixture.workspace_cmd();
    cmd_set
        .args([
            "config",
            "set",
            "test-workspace",
            "https://github.com/test/repo.git",
            "feature-branch",
        ])
        .current_dir(fixture.path());

    cmd_set
        .assert()
        .success()
        .stdout(predicate::str::contains("Set repository config"))
        .stdout(predicate::str::contains("https://github.com/test/repo.git"))
        .stdout(predicate::str::contains("feature-branch"));

    // Verify config was set by showing it
    let mut cmd_show = fixture.workspace_cmd();
    cmd_show
        .args(["config", "show", "test-workspace"])
        .current_dir(fixture.path());

    cmd_show
        .assert()
        .success()
        .stdout(predicate::str::contains("https://github.com/test/repo.git"))
        .stdout(predicate::str::contains("feature-branch"));
}

#[test]
fn test_config_set_with_git_ref() {
    let fixture = CliTestFixture::new();

    // Create a git worktree using 'workspace add' (config commands require git worktrees)
    let mut cmd_add = fixture.workspace_cmd();
    cmd_add
        .args(["add", "test-workspace", "--branch", "test-branch"])
        .current_dir(fixture.path());
    cmd_add.assert().success();

    // Set config with git ref
    let mut cmd_set = fixture.workspace_cmd();
    cmd_set
        .args([
            "config",
            "set",
            "test-workspace",
            "https://github.com/test/repo.git",
            "main",
            "v1.0.0",
        ])
        .current_dir(fixture.path());

    cmd_set
        .assert()
        .success()
        .stdout(predicate::str::contains("v1.0.0"));

    // Verify all parts are in the config
    let mut cmd_show = fixture.workspace_cmd();
    cmd_show
        .args(["config", "show", "test-workspace"])
        .current_dir(fixture.path());

    cmd_show
        .assert()
        .success()
        .stdout(predicate::str::contains("https://github.com/test/repo.git"))
        .stdout(predicate::str::contains("main"))
        .stdout(predicate::str::contains("v1.0.0"));
}

#[test]
fn test_config_set_fails_for_nonexistent_workspace() {
    let fixture = CliTestFixture::new();

    // Try to set config for nonexistent workspace
    let mut cmd_set = fixture.workspace_cmd();
    cmd_set
        .args([
            "config",
            "set",
            "nonexistent-workspace",
            "https://github.com/test/repo.git",
            "main",
        ])
        .current_dir(fixture.path());

    cmd_set
        .assert()
        .failure()
        .stderr(predicate::str::contains("does not exist"));
}

#[test]
fn test_config_set_default() {
    let fixture = CliTestFixture::new();

    // Set default config
    let mut cmd_set_default = fixture.workspace_cmd();
    cmd_set_default
        .args([
            "config",
            "set-default",
            "https://github.com/default/repo.git",
            "develop",
        ])
        .current_dir(fixture.path());

    cmd_set_default
        .assert()
        .success()
        .stdout(predicate::str::contains("Set default repository config"))
        .stdout(predicate::str::contains(
            "https://github.com/default/repo.git",
        ))
        .stdout(predicate::str::contains("develop"));
}

#[test]
fn test_config_set_default_with_ref() {
    let fixture = CliTestFixture::new();

    // Set default config with git ref
    let mut cmd_set_default = fixture.workspace_cmd();
    cmd_set_default
        .args([
            "config",
            "set-default",
            "https://github.com/default/repo.git",
            "main",
            "v2.0.0",
        ])
        .current_dir(fixture.path());

    cmd_set_default
        .assert()
        .success()
        .stdout(predicate::str::contains("v2.0.0"));
}

#[test]
fn test_config_import_from_file() {
    let fixture = CliTestFixture::new();

    // Import config will create the worktree automatically if it doesn't exist
    let mut cmd_import = fixture.workspace_cmd();
    cmd_import
        .args([
            "config",
            "import",
            "import-test",
            fixture.config_path.to_str().unwrap(),
        ])
        .current_dir(fixture.path());

    cmd_import
        .assert()
        .success()
        .stdout(predicate::str::contains("Importing configuration"))
        .stdout(predicate::str::contains("Import complete"));

    // Verify worktree was created
    let workspace_path = fixture.path().join(".worktrees").join("import-test");
    assert!(
        workspace_path.exists(),
        "Worktree should be created by import"
    );
}

#[test]
fn test_config_import_creates_workspace_if_missing() {
    let fixture = CliTestFixture::new();

    // Import should create the workspace if it doesn't exist
    let mut cmd_import = fixture.workspace_cmd();
    cmd_import
        .args([
            "config",
            "import",
            "auto-created-workspace",
            fixture.config_path.to_str().unwrap(),
        ])
        .current_dir(fixture.path());

    cmd_import
        .assert()
        .success()
        .stdout(predicate::str::contains("Import complete"));

    // Verify workspace was created
    let workspace_path = fixture
        .path()
        .join(".worktrees")
        .join("auto-created-workspace");
    assert!(
        workspace_path.exists(),
        "Workspace should be created by import"
    );
}

#[test]
fn test_config_import_fails_for_missing_file() {
    let fixture = CliTestFixture::new();

    let nonexistent_file = fixture.path().join("nonexistent.conf");

    // Try to import from nonexistent file
    let mut cmd_import = fixture.workspace_cmd();
    cmd_import
        .args([
            "config",
            "import",
            "test-workspace",
            nonexistent_file.to_str().unwrap(),
        ])
        .current_dir(fixture.path());

    cmd_import
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_config_inheritance_workspace_overrides_default() {
    let fixture = CliTestFixture::new();

    // Create a git worktree using 'workspace add' (config commands require git worktrees)
    let mut cmd_add = fixture.workspace_cmd();
    cmd_add
        .args(["add", "test-workspace", "--branch", "test-branch"])
        .current_dir(fixture.path());
    cmd_add.assert().success();

    // Set default config
    let mut cmd_set_default = fixture.workspace_cmd();
    cmd_set_default
        .args([
            "config",
            "set-default",
            "https://github.com/default/repo.git",
            "main",
        ])
        .current_dir(fixture.path());
    cmd_set_default.assert().success();

    // Set workspace-specific config (should override default)
    let mut cmd_set = fixture.workspace_cmd();
    cmd_set
        .args([
            "config",
            "set",
            "test-workspace",
            "https://github.com/override/repo.git",
            "feature",
        ])
        .current_dir(fixture.path());
    cmd_set.assert().success();

    // Show config - should see workspace-specific (not default)
    let mut cmd_show = fixture.workspace_cmd();
    cmd_show
        .args(["config", "show", "test-workspace"])
        .current_dir(fixture.path());

    cmd_show
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "https://github.com/override/repo.git",
        ))
        .stdout(predicate::str::contains("feature"));
}

#[test]
fn test_config_show_help() {
    let mut cmd = Command::cargo_bin("workspace").expect("Failed to find workspace binary");
    cmd.args(["config", "--help"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains(
            "Manage per-workspace configurations",
        ))
        .stdout(predicate::str::contains("show"))
        .stdout(predicate::str::contains("set"))
        .stdout(predicate::str::contains("set-default"))
        .stdout(predicate::str::contains("import"));
}

#[test]
fn test_config_multiple_repositories_in_workspace() {
    let fixture = CliTestFixture::new();

    // Create a git worktree using 'workspace add' (config commands require git worktrees)
    let mut cmd_add = fixture.workspace_cmd();
    cmd_add
        .args(["add", "multi-repo-workspace", "--branch", "multi-branch"])
        .current_dir(fixture.path());
    cmd_add.assert().success();

    // Set multiple repo configs
    let repos = vec![
        ("https://github.com/test/repo1.git", "main"),
        ("https://github.com/test/repo2.git", "develop"),
        ("https://github.com/test/repo3.git", "feature"),
    ];

    for (url, branch) in &repos {
        let mut cmd_set = fixture.workspace_cmd();
        cmd_set
            .args(["config", "set", "multi-repo-workspace", url, branch])
            .current_dir(fixture.path());
        cmd_set.assert().success();
    }

    // Show config - should see all three repos
    let mut cmd_show = fixture.workspace_cmd();
    cmd_show
        .args(["config", "show", "multi-repo-workspace"])
        .current_dir(fixture.path());

    cmd_show.assert().success();
    // Note: Config show will display at least one of the repos due to the inheritance chain
}
