# QuickBuilder: Compile-Time Builder with Enforcement of Run-Time Invariants

![build](https://github.com/geo-ant/quick-builder/actions/workflows/build.yml/badge.svg?branch=main)
![tests](https://github.com/geo-ant/quick-builder/actions/workflows/tests.yml/badge.svg?branch=main)
![lints](https://github.com/geo-ant/quick-builder/actions/workflows/lints.yml/badge.svg?branch=main)
[![crates](https://img.shields.io/crates/v/quick-builder)](https://crates.io/crates/quick-builder)
![maintenance-status](https://img.shields.io/badge/maintenance-actively--developed-brightgreen.svg)

This crate offers a simple, but powerful, compile-time builder pattern generator.
The philosophy is to verify as much as possible at compile-time, while also
providing a straightforward way to enforce run-time invariants.

## When Should You Try QuickBuilder?

Give QuickBuilder a shot if you want to derive a builder for your struct,
that

* makes it a **compile error** if you forget to set a field _and_
* allows you to **specify run-time invariants** for your struct that
  are enforced at run-time _and_
* you can live with the more ascetic interface that the builder provides,
  see the sections on limitations and alternatives.

## Motivation

There are great builder crates, like [bon](https://docs.rs/bon/latest/bon/)
or [typed-builder](https://crates.io/crates/typed-builder), that allow you
to create idiomatic builders enforcing that all necessary fields 
have been set at compile-time. Like those crates, QuickBuilder
can generate a builder that only allows to call the final `.build()` method if
all fields were initialized.

```rust
use quick_builder::QuickBuilder;

#[derive(QuickBuilder)]
struct ImageRef<'a, T> {
    width: usize,
    height: usize,
    data: &'a [T],
}

fn main() {
    let image_data = &[1, 2, 3, 4, 5, 6];
    let imgref = ImageRef::builder()
        .width(3)
        .height(2)
        .data(image_data)
        .build();
}
```

However, the example above is not the main usecase for QuickBuilder. If that's
all you ever need to do, check out the [bon](https://docs.rs/bon/latest/bon/) or 
[typed-builder](https://crates.io/crates/typed-builder) crates. Those offer,
among other things, more exhaustive support for idioms like optional and
default parameters, as well as great ergonomics. QuickBuilder shines when
you additionally need to enforce run-time invariants about your data structure.

## Enforcing Run-Time Invariants

In the example above we might want to enforce a couple of invariants
about the data structure which we can only check at run-time. The following 
example shows, how we can use QuickBuilder to enforce that...

1. ...the width of the image is greater 0 _and_
2. the height of the image is an even number greater 0 _and_
3. the product of width and height is equal to the length of the given slice.

```rust
use quick_builder::QuickBuilder;

#[derive(QuickBuilder)]
#[invariant(|my| my.width * my.height == my.data.len())]
struct ImageRef<'a, T> {
    #[invariant(|w|*w>0)]
    width: usize,
    #[invariant(check_height)]
    height: usize,
    data: &'a [T],
}

// if the conditions to check invariants are too
// unwieldy to put into a closure, you can also
// define a standalone function.
fn check_height(height :&usize) -> bool {
  *height > 0 && *height % 2 == 0
}

fn main() {
    let image_data = &[1, 2, 3, 4, 5, 6];
    let imgref = ImageRef::builder()
        .width(3)
        .height(2)
        .data(image_data)
        .build()
        .unwrap();
}
```

One (or zero) `#[invariant(...)]` attributes can be applied to each field or
to the struct itself. The attributes take a closure or function name to check
if the invariant holds. The function (or closure) must take its
argument by reference and return a `bool`, where `true` means that the invariant
holds and `false` means it's violated.

As soon as an `#[invariant(...)]` attribute is encountered, the `build` function
changes its signature. It now returns an optional instance of the original
structure, where the optional contains a value if and only if all invariants
where upheld during construction.

## Limitations

* **Build Order**: The builder function must be executed in the order of
  field declarations in the struct. Typically, IDE support is good enough
  to provide you with the next allowed option, so you don't have to look
  up the struct fields. The `bon` and `typed-builder` crates allow arbitrary
  orders, but they don't have a mechanism for enforcing run-time invariants.
* **Default/Optional Arguments**: there is no support for default or optional
  arguments (yet).
* **Weird Generics**: The builder structure contains a bit of generic magic
  and is not meant for passing around.
* **Consuming Builder Pattern Only**: The builder uses the consuming pattern always.
  If you need to set fields conditionally, check out the [apply_if](https://crates.io/crates/apply_if)
  crate.

## Protecting Field Access: Getters

If care about enforcing invariants about your data, you probably want to provide getters
to your fields rather than making them publicly accessible. In that case, you'll
be happy to hear that this crate works seamlessly with the popular
[getset](https://crates.io/crates/getset) and [derive-getters](https://crates.io/crates/derive-getters)
crates. Those offer derive macros for getters and setters.

## Alternatives

There is a great [overview of builder crates](https://elastio.github.io/bon/guide/alternatives)
by the `bon` team. Of the compile-time builders in this list, to my knowledge, only the `bon`
crate provides a way to enforce run-time invariants. 
