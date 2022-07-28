// TODO :
//   - [x] load every file in the crate
//   - [ ] implement the architecture layer assertions
//   - [x] implement visibility assertion
//   - implement
//     - [ ] fn
//     - [x] struct
//     - [x] enum
//     - [x] modules

extern crate core;

pub mod assertion_result;
pub mod ast;
pub mod rule;
use ast::ModuleTree;

pub use rule::modules::Modules;
pub use rule::structs::Structs;
