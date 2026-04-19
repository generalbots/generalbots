#[test]
fn test_unit_module_loads() {
    let module_name = module_path!();
    assert!(module_name.contains("unit"));
}
