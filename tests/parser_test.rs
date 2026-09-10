use smvx::parser::parse_program;
use smvx::codegen::generate_smv;
use smvx::ir::lower;

#[test]
fn test_parse_smvx_list() {
    let file_path = "tests/fixtures/list.smvx";
    let result = parse_program(file_path);
    let lowered = lower(result.unwrap());
    let smv = generate_smv(&lowered);

    print!("{}", smv);
    assert!(!smv.is_empty());
}

#[test]
fn test_parse_smvx_loop() {
    let file_path = "tests/fixtures/loop.smvx";
    let result = parse_program(file_path);
    let lowered = lower(result.unwrap());
    let smv = generate_smv(&lowered);

    print!("{}", smv);
    assert!(!smv.is_empty());
}
