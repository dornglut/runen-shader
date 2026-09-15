pub(crate) fn parse_wgsl(source: &str) -> Result<naga::Module, naga::front::wgsl::ParseError> {
    let mut frontend = naga::front::wgsl::Frontend::new_with_options(naga::front::wgsl::Options {
        parse_doc_comments: false,
        capabilities: naga::valid::Capabilities::all(),
    });
    frontend.parse(source)
}

pub(crate) fn validate_wgsl(
    module: &naga::Module,
) -> Result<(), naga::WithSpan<naga::valid::ValidationError>> {
    let mut validator = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::all(),
    );
    validator.subgroup_stages(naga::valid::ShaderStages::all());
    validator.subgroup_operations(naga::valid::SubgroupOperationSet::all());
    validator.validate(module).map(|_| ())
}
