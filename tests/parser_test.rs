use smvx::parser::parse_program; // Assumes your crate name in Cargo.toml is nusmv_ext
use smvx::codegen::generate_smv;

#[test]
fn test_parse_smvx_list() {
    let file_path = "tests/fixtures/list.smvx";
    let result = parse_program(file_path);
    let smv = generate_smv(&result.unwrap());

    print!("{}", smv);
    assert!(!smv.is_empty());
}
