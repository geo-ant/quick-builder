# QuickBuilder: Compile Time Builder with Enforcement of Run-Time Invariants

This crate offers a simple, but powerful, compile time builder pattern generator.
The philosophy is to verify as much as it can at compile time, while also
providing as straightforward way to check runtime invariants.

## When should you try QuickBuilder?

Give QuickBuilder a shot if you want to derive a builder pattern for your struct,
that

* makes it a **compile error** if you forgot to set a field _and_
* allows you to **specify runtime invariants** for your fields and struct that
  are enforced at runtime _and_
* you can live with the more ascetic interface that the builder provides,
  see the section on limitations.

## Motivation

There are great builder crates, like [bon](https://docs.rs/bon/latest/bon/)
or [typed-builder](https://crates.io/crates/typed-builder), that allow you
to create idiomatic builders that enforce that all necessary fields 
have been set at compile time. Like those crates, QuickBuilder
can generate a builder that only allows to create an instance of the
type if all fields were initialized.

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
[typed-builder](https://crates.io/crates/typed-builder) crates which offer,
among other things, more exhaustive support for things like optional and
default parameters as well as great ergonomics. QuickBuilder shines when
you also need to enforce runtime invariants about your data structure.

## Enforcing Runtime Invariants

In the example above we might want to enforce a couple of invariants
about our instances of our data structure. The following example shows,
how we can use QuickBuilder to enforce that:

1. the width of the image is greater 0,
2. the height of the image is greater 0 and it is even.
3. the product of width and height is equal to the length of the given slice

```rust
use quick_builder::QuickBuilder;

#[derive(QuickBuilder)]
#[invariant(|this| this.width * this.height == this.data.len() )]
struct ImageRef<'a, T> {
    #[invariant(|w|*w>0)]
    width: usize,
    #[invariant(check_height)]
    height: usize,
    data: &'a [T],
}

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
to the outer struct itself. The attributes take a closure or function name to check
if the invariant holds. The function (or closure) must take its
argument by reference and return a `bool`, where `true` means that the invariant
holds and `false` means it's violated.

As soon as an `#[invariant(...)]` attribute is encountered, the `build` function
changes its signature. It now returns an optional instance of the original
structure, where the optional contains a value if and only if all invariants
where upheld during the building.

## Limitations

* **Build Order**: The builder function must be executed in the order of
  field declarations in the struct. Typically, IDE support is good enough
  to provide you with the next allowed option, so you don't have to look
  up the struct fields. The `bon` and `typed-builder` crates allow arbitrary
  orders, but they don't have a mechanism for enforcing run-time invariants.
* **Default/Optional Arguments**: there is no support for default or optional
  arguments (yet).

## Alternatives

There is a great [overview of builder crates](https://elastio.github.io/bon/guide/alternatives)
by the `bon` team. Of those, to my knowledge, only the [derive_builder](https://docs.rs/derive_builder/latest/derive_builder/)
crate provides a way to enforce run-time invariants. However, this crate
does not verify at compile-time that all required fields have been set. It would
be a run-time error in that case. Some might argue, if we have run-time
errors anyways (due to the invariants) we might not care about that. But my
philosophy is that I'd rather validate as much as I can at compile time and
let run-time errors be run-time errors, but that's just me.
