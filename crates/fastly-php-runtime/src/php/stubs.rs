use ext_php_rs::describe::ToStub;

pub fn generate_fastly_ce_stubs() -> String {
    let description = fastly_ce_module::ext_php_rs_describe_module();

    description
        .module
        .to_stub()
        .expect("stub generation failed")
}
