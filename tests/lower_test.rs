use smvx::transpiler::SMVXTranspiler;

#[test]
// Tests that the lowerer two-pass architecture is able to pick up lists defined on DECLARE
// and referenced on VAR block on top
fn test_lower_picks_var_declare() {
    let file_path = "tests/fixtures/dependency/dependency.smvx";
    let expected_path = "tests/fixtures/dependency/dependency.smv";

    let mut transpiler = SMVXTranspiler::new();
    let smv = transpiler.transpile(file_path.into()).unwrap();

    print!("{}", smv);
    assert!(!smv.is_empty());
    assert_eq!(smv, std::fs::read_to_string(expected_path).unwrap());
}

