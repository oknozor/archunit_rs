<div align="center">
    <h1># Archunit-rs</h1>
<p>Archunit-rs is an architecture testing library inspired by <a href="https://www.archunit.org/">Archunit</a>.
</p>
  <a href="https://github.com/oknozor/archunit_rs/actions"
    ><img
      src="https://github.com/oknozor/archunit_rs/actions/workflows/CI.yaml/badge.svg"
      alt="GitHub Actions workflow status"
  /></a>
    <a href="https://codecov.io/gh/oknozor/archunit_rs"
    ><img
    src="https://codecov.io/gh/oknozor/archunit_rs/branch/main/graph/badge.svg"
    alt="Code coverage status"/>
    </a>
  <br />
  <a href="https://conventionalcommits.org"
    ><img
      src="https://img.shields.io/badge/Conventional%20Commits-1.0.0-yellow.svg"
      alt="Conventional commits"
  /></a>
  <a href="https://github.com/cocogitto/cocogitto/blob/main/LICENSE"
    ><img
      src="https://img.shields.io/github/license/cocogitto/cocogitto"
      alt="Repository license"
  /></a>
</div>


---

⚠️ **Disclaimer**: this is a work in progress, expect bugs and missing features until first release.

## Example

```rust
#[test]
fn arch_rules() {
    Structs::that(Filters::default())
        .implement("Display")
        .should()
        .be_public()
        .check();

    Modules::that()
        .reside_in_a_module("foo::bar")
        .or()
        .are_declared_private()
        .should()
        .only_have_dependency_module()
        .that()
        .have_simple_name("baz")
        .check();

    Enums::all_should()
        .implement_or_derive("Debug")
        .check();
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


## TODO

1. logical conjuction
Handle Or and And conjuction with the new report model.
i.e. reports should be added to the final result only if the conjuction
result in a logical failure.
ex: Struct::all().should().be_public().or().be_private().check() should not fail

2. Nice API for filtering Modules
currently :  Structs::that(filter).... Assertions
should be :  Structs::filtering(filter).that() ... Assertions

3. Stabilize layer assertions

4. Make sure all report are correct (message + code samples)

5. Docs

6. Book

7. Alpha

8. impl blocks and fn Assertions