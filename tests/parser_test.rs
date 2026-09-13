use smvx::parser::parse_program;
use smvx::codegen::generate_smv;
use smvx::ir::Lowerer;

#[test]
fn test_parse_smvx_list() {
    let file_path = "tests/fixtures/list.smvx";
    // TODO: fix the identation and line breaks later
    let expected_path = "tests/fixtures/list.smv";

    let parsed = parse_program(file_path);
    let lower = Lowerer::new();
    let lowered = lower.lower(parsed.unwrap()).unwrap();
    let smv = generate_smv(&lowered);

    print!("{}", smv);
    assert!(!smv.is_empty());
    assert_eq!(smv, std::fs::read_to_string(expected_path).unwrap());
}

#[test]
fn test_parse_smvx_loop() {
    let file_path = "tests/fixtures/loop.smvx";
    let expected_path = "tests/fixtures/loop.smv";
    
    let parsed = parse_program(file_path);
    let lower = Lowerer::new();
    let lowered = lower.lower(parsed.unwrap()).unwrap();
    let smv = generate_smv(&lowered);

    print!("{}", smv);
    assert!(!smv.is_empty());
    assert_eq!(smv, std::fs::read_to_string(expected_path).unwrap());
}
