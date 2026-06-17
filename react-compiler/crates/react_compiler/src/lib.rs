#[cfg(debug_assertions)]
pub mod debug_print;

/// Release builds compile out the HIR debug printer entirely. It is only used
/// by `yarn snap -d` (the per-pass HIR dump), which builds the napi addon in
/// debug mode; the shipped release binary never sets `debug_enabled`. This stub
/// keeps the `debug_print::debug_hir` call sites in the pipeline compiling while
/// the heavy formatter code (`DebugPrinter`, `PrintFormatter`) is dropped from
/// the shipped binary.
#[cfg(not(debug_assertions))]
pub mod debug_print {
    use react_compiler_hir::HirFunction;
    use react_compiler_hir::environment::Environment;

    #[inline(always)]
    pub fn debug_hir(_hir: &HirFunction, _env: &Environment) -> String {
        String::new()
    }
}

pub mod entrypoint;
pub mod fixture_utils;
pub mod timing;

// Re-export from new crates for backwards compatibility
pub use react_compiler_diagnostics;
pub use react_compiler_hir;
pub use react_compiler_hir as hir;
pub use react_compiler_hir::environment;
pub use react_compiler_lowering::lower;
