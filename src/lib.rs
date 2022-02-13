// TODO :
//   - [x] load every file in the crate
//   - implement the architecture layer assertions
//   - [x] implement visibility assertion
//   - implement function/struct/enum/modules matchers

extern crate core;

pub mod assertion_result;
pub mod ast;
pub mod rule;
use ast::ModuleTree;
