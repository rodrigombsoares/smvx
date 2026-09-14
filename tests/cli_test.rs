use std::fs;
use std::process::Command;

#[test]
fn test_cli_transpile_file() {
    let bin_path = env!("CARGO_BIN_EXE_smvx");
    let input_path = "tests/fixtures/list/list.smvx";
    let expected_path = "tests/fixtures/list/list.smv";
    let output_path = "target/tmp_cli_test_list.smv";

    let status = Command::new(bin_path)
        .arg("-s")
        .arg(input_path)
        .arg("-o")
        .arg(output_path)
        .status()
        .expect("Failed to execute smvx CLI binary");

    assert!(status.success(), "CLI execution failed");

    let generated = fs::read_to_string(output_path).expect("Failed to read output file");
    let expected = fs::read_to_string(expected_path).expect("Failed to read expected file");
    assert_eq!(generated, expected);

    let _ = fs::remove_file(output_path);
}

#[test]
fn test_cli_transpile_import() {
    let bin_path = env!("CARGO_BIN_EXE_smvx");
    let input_path = "tests/fixtures/import/main.smvx";
    let expected_path = "tests/fixtures/import/import.smv";
    let output_path = "target/tmp_cli_test_import.smv";

    let status = Command::new(bin_path)
        .arg("-s")
        .arg(input_path)
        .arg("-o")
        .arg(output_path)
        .status()
        .expect("Failed to execute smvx CLI binary");

    assert!(status.success(), "CLI execution failed");

    let generated = fs::read_to_string(output_path).expect("Failed to read output file");
    let expected = fs::read_to_string(expected_path).expect("Failed to read expected file");
    assert_eq!(generated, expected);

    let _ = fs::remove_file(output_path);
}

#[test]
fn test_cli_missing_required_args() {
    let bin_path = env!("CARGO_BIN_EXE_smvx");

    let output = Command::new(bin_path)
        .output()
        .expect("Failed to execute smvx CLI binary");

    assert!(!output.status.success(), "CLI should fail when missing arguments");
}
