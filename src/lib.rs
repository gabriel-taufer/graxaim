// `core` and `errors` form the stable public API consumed by integration tests.
// `commands` and `ui` are CLI-specific internals declared directly in
// `src/main.rs` (the binary entry point) and are not part of the library API.
pub mod core;
pub mod errors;
