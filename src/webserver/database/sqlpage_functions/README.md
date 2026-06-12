# Built-in SQL functions

Each built-in `sqlpage.*` function is a plain `async fn` in its own file in [`functions/`](functions),
with an ordinary Rust signature:

```rust
#[allow(clippy::wildcard_imports)]
use super::*;

pub(super) async fn example(request: &RequestInfo, value: Option<Cow<'_, str>>) -> Option<String> {
    // ...
}
```

To register it, add one line to the list in [`functions.rs`](functions.rs), giving the SQL names of
its arguments (used only in error messages):

```rust
sqlpage_functions! {
    // ...
    example("value");
}
```

The [`sqlpage_functions!`](function_traits.rs) macro declares the modules and generates the
`SqlPageFunctionName` enum the SQL engine dispatches on. That is the only compile-time code: there is
no build script involvement and no per-function generated code. Argument extraction, dispatch, and
return-value conversion are handled generically in [`function_traits.rs`](function_traits.rs) by the
`Extract`, `Handler`, and `IntoCowResult` traits. A function's argument and return types are read
straight from its signature, so the supported argument types are exactly those that implement
`Extract` (add an `impl` there to support a new one).

Keep helpers and unit tests that are specific to a function in that function's file. Shared helpers can
be made `pub(super)` and used by sibling function modules through `use super::*`.
