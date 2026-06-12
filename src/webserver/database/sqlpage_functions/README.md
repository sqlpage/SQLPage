# Built-in SQL functions

Each built-in `sqlpage.*` function is a plain `async fn` in its own file in
[`functions/`](functions). The file stem is the SQL function name and must match the Rust function it
exports:

```rust
use std::borrow::Cow;

use crate::webserver::http_request_info::RequestInfo;

pub(super) async fn example(request: &RequestInfo, value: Option<Cow<'_, str>>) -> Option<String> {
    // ...
}
```

To add `sqlpage.example`, create `functions/example.rs` and add it to the
[`sqlpage_functions!`](function_traits.rs) call in [`functions.rs`](functions.rs):

```rust
sqlpage_functions! {
    // ...
    example,
}
```

The [`sqlpage_functions!`](function_traits.rs) macro declares the modules and generates the
`SqlPageFunctionName` enum the SQL engine dispatches on. Per-function argument extraction, dispatch,
and return-value conversion are handled generically in [`function_traits.rs`](function_traits.rs) by
the `Extract`, `Handler`, and `IntoCowResult` traits. A function's argument and return types are read
straight from its signature, so supported argument types are the types that implement `Extract` there.
Functions can take up to five arguments.

Keep helpers and unit tests that are specific to a function in that function's file. Shared helpers can
be made `pub(super)` and imported by name from sibling function modules.
