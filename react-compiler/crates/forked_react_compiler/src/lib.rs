pub mod debug_print;
pub mod entrypoint;
pub mod fixture_utils;
pub mod timing;

// Re-export from new crates for backwards compatibility
pub use forked_react_compiler_diagnostics;
pub use forked_react_compiler_hir;
pub use forked_react_compiler_hir as hir;
pub use forked_react_compiler_hir::environment;
pub use forked_react_compiler_lowering::lower;
