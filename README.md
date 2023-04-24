# apex.rs

Required Services
https://www.aviation-ia.com/support-files/arinc653h

Extended Services
https://www.aviation-ia.com/support-files/arinc653p2h


## TODOS
- Write Readme
- Write RustDoc

## Position of **apex.rs**

**apex.rs** is supposed to be used as a dependency by the Hypervisor-Wrapper-Library, partitions and processes.

A potential Hypervisor-Wrapper-Library would use introduced Apex traits for providing functionality expected of an ARINC653 compliant hypervisor.

Downstream partitions and processes written in Rust would use **apex.rs** in combination with any Hypervisor-Wrapper-Library implementing **apex.rs** for gaining safe abstracted Apex structures for using Apex functionalities.

For both cases *preludes* were defined offering all required structs, traits and type-aliases.
- `bindings`: for the Hypervisor-Wrapper-Library
- `prelude`: for downstream users (partitions and processes)

## Design Decisions

This crate offers multiple traits, targeted towards specific functionalities defined in the ARINC653 standard.
- ApexBlackboardP1
- ApexBufferP1
- ApexErrorP1/P4
- ...

These traits require the implementer to implement static functions which are closely related to the functions defined in the ARINC653 standard (see header file in [Required Services](https://www.aviation-ia.com/support-files/arinc653h)). Most likely the implementation of these functions is just a [bindgen](https://rust-lang.github.io/rust-bindgen/) to underlying c-functions of the hypervisor

> The separation of these static functions into specific traits was done because this way hypervisors with limited capabilities can at least offer a limited set instead of none.

While the required functions are closely related to the functions defined in the ARINC653 standard they are not identical. None of the functions in the standard return any value. Instead, references are passed into them, which are mutated during the function call. While our functions could have used the same structure we opted for a more Rust-like function style, in which we at least return a Result with an expected `Ok`-type in the ReturnCode `NoError`-case and a `ErrorReturnCode` as the `Err`-type in all other cases.

> `ErrorReturnCode` is just the ARINC653 specific ReturnCode enum type without the `NoError`-variant.

### Unsafe Functions

While it is expected that the implementing Hypervisor-Wrapper-Library will need to use a bunch of unsafe code blocks and functions, only one type of function in our traits is explicitly marked as unsafe: functions taking a mutable reference as an argument. Mutable references are only passed to *read/receive* functions, which take mutable references to `ApexByte`-slices. Since it is exceptionally unsafe should the user provide a mutable slice which is too small, this function is marked as unsafe.

> Because of this we generally provide two functions within our abstractions. One safe-function which checks that the mutable slice provided by the user can at least hold the defined max-message-size and one unsafe-function which just hopes that the read/received message is not longer than what the user expected.

## Decisions towards Downstream User (Partitions and Processes)

TODO

# Licensing Information

The ARINC 653 standard belongs to the Aeronautical Radio, Incorporated (ARINC).

For this library the copyright belongs to the German Aerospace Center / Deutsches Zentrum für Luft- und Raumfahrt e.V. (DLR): 

Copyright © 2023 Deutsches Zentrum für Luft- und Raumfahrt e.V. (DLR)
