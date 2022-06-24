# Archunit-rs

Archunit-rs is an architecture testing library inspired by [ArchUnit](https://www.archunit.org/) for checking
the architecture of your crates using unit tests.

---

⚠️ **Disclaimer**: this is a work in progress, expect bugs and missing features until first release.

## Example

```rust
#[test]
fn arch_rules() {
    Structs::that()
        .implement("Display")
        .should()
        .public()
        .check();

    Modules::that()
        .reside_in_a_module("foo::bar")
        .or()
        .are_declared_private()
        .should()
        .only_have_dependency_module()
        .that()
        .have_simple_name("baz")
        .check()
}
```

## Motivation

Rust’s type system offer many technical guarantee: memory-safety, thread-safety, no undefined variable, no dangling
pointer etc.
These guarantee prevent developers from writing incorrect code and make Rust quite unique as a programming language.
While the compiler ensure our codebase "works" from a machine perspective, humans needs some insurance that it also work
as intended. That's what test are for, adding some functional guarantee on top of those technical ones.

But, when working on large codebase compile time and functional guarantee will not stop you
from messing with the architecture once defined.
Rustc cares no more than your test harness about how your crate modules are organised, how your struct and traits
are named, what functions your public API exposes.

Either explicitly defined on a diagram or implied by the early version of a project, software architecture will
eventually erode.
Initial architectural choice will fade and become less and less explicit.
New developers won't be able to understand the initial intent, and will add more and more changes. The code base will
end up
harder to understand, test and refactor.

Archunit tries to fill that gap by bringing architectural test in the scope of unit testing.
Why write a diagram or an architecture document when you can simply write a unit test that
serve as a architecture documentation and enforce it as the same time ?


