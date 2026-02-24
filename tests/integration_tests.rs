use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn code_weather() -> Command {
    Command::cargo_bin("code-weather").unwrap()
}

// Test 1: Help shows all commands
#[test]
fn test_help_shows_commands() {
    code_weather()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("forecast"))
        .stdout(predicate::str::contains("init"))
        .stdout(predicate::str::contains("explain"));
}

// Test 2: Version flag works
#[test]
fn test_version_flag() {
    code_weather()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("code-weather"));
}

// Test 3: Forecast on non-existent path fails
#[test]
fn test_forecast_nonexistent_path() {
    code_weather()
        .args(["forecast", "/definitely/not/a/real/path"])
        .assert()
        .failure();
}

// Test 4: Forecast with JSON output
#[test]
fn test_forecast_json_output() {
    let dir = TempDir::new().unwrap();
    
    // Create a simple Rust file
    fs::write(
        dir.path().join("main.rs"),
        "fn main() { println!(\"hello\"); }\n",
    ).unwrap();
    
    code_weather()
        .args(["forecast", "--format", "json"])
        .arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("\"condition\""))
        .stdout(predicate::str::contains("\"temperature\""));
}

// Test 5: Forecast with markdown output
#[test]
fn test_forecast_markdown_output() {
    let dir = TempDir::new().unwrap();
    
    fs::write(
        dir.path().join("lib.py"),
        "def hello():\n    return 'hello'\n",
    ).unwrap();
    
    code_weather()
        .args(["forecast", "--format", "markdown"])
        .arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("# Code Weather Report"))
        .stdout(predicate::str::contains("Temperature"));
}

// Test 6: Init creates config file
#[test]
fn test_init_creates_config() {
    let dir = TempDir::new().unwrap();
    
    code_weather()
        .arg("init")
        .current_dir(dir.path())
        .assert()
        .success();
    
    assert!(dir.path().join(".code-weather.toml").exists());
    
    let content = fs::read_to_string(dir.path().join(".code-weather.toml")).unwrap();
    assert!(content.contains("[thresholds]"));
}

// Test 7: Init refuses to overwrite without force
#[test]
fn test_init_no_overwrite() {
    let dir = TempDir::new().unwrap();
    let config = dir.path().join(".code-weather.toml");
    fs::write(&config, "# existing").unwrap();
    
    code_weather()
        .arg("init")
        .current_dir(dir.path())
        .assert()
        .failure();
}

// Test 8: Init with force overwrites
#[test]
fn test_init_force_overwrite() {
    let dir = TempDir::new().unwrap();
    let config = dir.path().join(".code-weather.toml");
    fs::write(&config, "# old").unwrap();
    
    code_weather()
        .args(["init", "--force"])
        .current_dir(dir.path())
        .assert()
        .success();
    
    let content = fs::read_to_string(&config).unwrap();
    assert!(content.contains("[thresholds]"));
}

// Test 9: Explain lists all conditions
#[test]
fn test_explain_all() {
    code_weather()
        .arg("explain")
        .assert()
        .success()
        .stdout(predicate::str::contains("Sunny"))
        .stdout(predicate::str::contains("Stormy"))
        .stdout(predicate::str::contains("Foggy"))
        .stdout(predicate::str::contains("Frozen"));
}

// Test 10: Explain specific condition
#[test]
fn test_explain_specific() {
    code_weather()
        .args(["explain", "stormy"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Stormy"));
}

// Test 11: Forecast handles multiple languages
#[test]
fn test_multi_language_repo() {
    let dir = TempDir::new().unwrap();
    
    fs::write(dir.path().join("app.ts"), "export function greet(): string { return 'hi'; }").unwrap();
    fs::write(dir.path().join("lib.py"), "def helper():\n    pass").unwrap();
    fs::write(dir.path().join("main.rs"), "fn main() {}").unwrap();
    fs::write(dir.path().join("util.go"), "package main\nfunc main() {}").unwrap();
    
    code_weather()
        .args(["forecast", "--format", "json"])
        .arg(dir.path())
        .assert()
        .success();
}

// Test 12: Verbose flag works with terminal output
#[test]
fn test_verbose_output() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("test.py"), "def foo(): pass").unwrap();
    
    // Verbose flag should not break the command
    code_weather()
        .args(["-v", "forecast"])
        .arg(dir.path())
        .assert()
        .success();
}
