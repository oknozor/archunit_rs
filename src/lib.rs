// TODO :
//   - load every file in the crate
//   - implement the architecture layer assertions
//   - implement visibility assertion
//   - implement function/struct/enum/module matchers

pub mod parse;
pub mod rule;

/*
ArchRule rule = Functions::that()
    .arePublic()
    .and()
    .are_declared_in_struct_that()
    .reside_in_a_module("::controller::")
    .should()
    .derive(Debug)
 */
// Structs::that()
//     .reside_in_a_module("foo::bar")
//     .should()
//     .be_public()
