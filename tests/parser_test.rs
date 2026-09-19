use smvx::transpiler::SMVXTranspiler;

#[test]
fn test_parse_smvx_list() {
    let file_path = "tests/fixtures/list/list.smvx";
    let expected_path = "tests/fixtures/list/list.smv";

    let mut transpiler = SMVXTranspiler::new();
    let smv = transpiler.transpile(file_path.into()).unwrap();

    print!("{}", smv);
    assert!(!smv.is_empty());
    assert_eq!(smv, std::fs::read_to_string(expected_path).unwrap());
}

#[test]
fn test_parse_smvx_loop() {
    let file_path = "tests/fixtures/loop/loop.smvx";
    let expected_path = "tests/fixtures/loop/loop.smv";

    let mut transpiler = SMVXTranspiler::new();
    let smv = transpiler.transpile(file_path.into()).unwrap();

    print!("{}", smv);
    assert!(!smv.is_empty());
    assert_eq!(smv, std::fs::read_to_string(expected_path).unwrap());
}

#[test]
fn test_parse_smvx_import() {
    let file_path = "tests/fixtures/import/main.smvx";
    let expected_path = "tests/fixtures/import/import.smv";

    let mut transpiler = SMVXTranspiler::new();
    let smv = transpiler.transpile(file_path.into()).unwrap();

    print!("{}", smv);
    assert!(!smv.is_empty());
    assert_eq!(smv, std::fs::read_to_string(expected_path).unwrap());
}

#[test]
fn test_parse_smvx_expand() {
    let file_path = "tests/fixtures/expand/expand.smvx";
    let expected_path = "tests/fixtures/expand/expand.smv";

    let mut transpiler = SMVXTranspiler::new();
    let smv = transpiler.transpile(file_path.into()).unwrap();

    print!("{}", smv);
    assert!(!smv.is_empty());
    assert_eq!(smv, std::fs::read_to_string(expected_path).unwrap());
}
