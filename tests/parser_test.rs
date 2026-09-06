use smvx::parser::parse_smvx; // Assumes your crate name in Cargo.toml is nusmv_ext

#[test]
fn test_parse_smvx_file() {
    let file_path = "tests/fixtures/counter.smvx";
    parse_smvx(file_path);
}