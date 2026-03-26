// src/stdlib.rs
// SLIME Standard Library — Rust-backed intrinsics
// These are injected into every compiled WASM module as host imports or
// inlined by the compiler depending on target.

use crate::ast::{Expr, Type};

/// Stdlib function descriptor — tells the compiler what's available
/// without needing a .slime source file for core builtins.
#[derive(Debug, Clone)]
pub struct StdlibFn {
    pub name: &'static str,
    pub params: &'static [Type],
    pub ret: Type,
    pub kind: StdlibKind,
}

#[derive(Debug, Clone)]
pub enum StdlibKind {
    /// Emitted as a WASM import (host provides implementation)
    HostImport { module: &'static str, field: &'static str },
    /// Inlined as WASM opcodes by the compiler
    Intrinsic { emit: fn(&mut Vec<u8>) },
}

/// The full stdlib function table.
/// The type checker consults this so user code can call these without declaring them.
pub fn stdlib_functions() -> Vec<StdlibFn> {
    vec![
        // ── I/O ──────────────────────────────────────────────────────────────
        StdlibFn {
            name: "print",
            params: &[Type::I32, Type::I32], // ptr, len
            ret: Type::Void,
            kind: StdlibKind::HostImport {
                module: "slime",
                field: "print",
            },
        },
        StdlibFn {
            name: "println",
            params: &[Type::I32, Type::I32], // ptr, len
            ret: Type::Void,
            kind: StdlibKind::HostImport {
                module: "slime",
                field: "println",
            },
        },
        StdlibFn {
            name: "print_i32",
            params: &[Type::I32],
            ret: Type::Void,
            kind: StdlibKind::HostImport {
                module: "slime",
                field: "print_i32",
            },
        },
        StdlibFn {
            name: "print_f64",
            params: &[Type::F64],
            ret: Type::Void,
            kind: StdlibKind::HostImport {
                module: "slime",
                field: "print_f64",
            },
        },

        // ── Math intrinsics (inlined as WASM opcodes) ─────────────────────
        StdlibFn {
            name: "abs_i32",
            params: &[Type::I32],
            ret: Type::I32,
            kind: StdlibKind::Intrinsic {
                emit: |out| {
                    // wasm has no i32.abs; emit: local.tee 0; i32.const 0; i32.sub; local.get 0; i32.gt_s; select
                    out.extend_from_slice(&[
                        0x22, 0x00,       // local.tee 0
                        0x41, 0x00,       // i32.const 0
                        0x6b,             // i32.sub
                        0x20, 0x00,       // local.get 0
                        0x41, 0x00,       // i32.const 0
                        0x4a,             // i32.gt_s
                        0x1b,             // select
                    ]);
                },
            },
        },
        StdlibFn {
            name: "abs_f64",
            params: &[Type::F64],
            ret: Type::F64,
            kind: StdlibKind::Intrinsic {
                emit: |out| {
                    out.push(0x99); // f64.abs
                },
            },
        },
        StdlibFn {
            name: "sqrt",
            params: &[Type::F64],
            ret: Type::F64,
            kind: StdlibKind::Intrinsic {
                emit: |out| {
                    out.push(0x9f); // f64.sqrt
                },
            },
        },
        StdlibFn {
            name: "floor",
            params: &[Type::F64],
            ret: Type::F64,
            kind: StdlibKind::Intrinsic {
                emit: |out| {
                    out.push(0x9c); // f64.floor
                },
            },
        },
        StdlibFn {
            name: "ceil",
            params: &[Type::F64],
            ret: Type::F64,
            kind: StdlibKind::Intrinsic {
                emit: |out| {
                    out.push(0x9b); // f64.ceil
                },
            },
        },
        StdlibFn {
            name: "min_i32",
            params: &[Type::I32, Type::I32],
            ret: Type::I32,
            kind: StdlibKind::Intrinsic {
                emit: |out| {
                    // a, b on stack → local.tee tmp; local.get a; local.get tmp; i32.lt_s; select
                    // simplified: just emit both args then select via comparison
                    out.extend_from_slice(&[
                        0x22, 0x01,  // local.tee 1 (b)
                        0x20, 0x00,  // local.get 0 (a)
                        0x20, 0x01,  // local.get 1 (b)
                        0x20, 0x00,  // local.get 0 (a)
                        0x20, 0x01,  // local.get 1 (b)
                        0x48,        // i32.lt_s
                        0x1b,        // select
                    ]);
                },
            },
        },
        StdlibFn {
            name: "max_i32",
            params: &[Type::I32, Type::I32],
            ret: Type::I32,
            kind: StdlibKind::Intrinsic {
                emit: |out| {
                    out.extend_from_slice(&[
                        0x22, 0x01,  // local.tee 1 (b)
                        0x20, 0x00,  // local.get 0 (a)
                        0x20, 0x01,  // local.get 1 (b)
                        0x20, 0x00,  // local.get 0 (a)
                        0x20, 0x01,  // local.get 1 (b)
                        0x4a,        // i32.gt_s
                        0x1b,        // select
                    ]);
                },
            },
        },
        StdlibFn {
            name: "min_f64",
            params: &[Type::F64, Type::F64],
            ret: Type::F64,
            kind: StdlibKind::Intrinsic {
                emit: |out| {
                    out.push(0xa1); // f64.min
                },
            },
        },
        StdlibFn {
            name: "max_f64",
            params: &[Type::F64, Type::F64],
            ret: Type::F64,
            kind: StdlibKind::Intrinsic {
                emit: |out| {
                    out.push(0xa2); // f64.max
                },
            },
        },

        // ── Memory ───────────────────────────────────────────────────────────
        StdlibFn {
            name: "memory_size",
            params: &[],
            ret: Type::I32,
            kind: StdlibKind::Intrinsic {
                emit: |out| {
                    out.extend_from_slice(&[0x3f, 0x00]); // memory.size
                },
            },
        },
        StdlibFn {
            name: "memory_grow",
            params: &[Type::I32],
            ret: Type::I32,
            kind: StdlibKind::Intrinsic {
                emit: |out| {
                    out.extend_from_slice(&[0x40, 0x00]); // memory.grow
                },
            },
        },

        // ── Panic ─────────────────────────────────────────────────────────
        StdlibFn {
            name: "panic",
            params: &[Type::I32, Type::I32], // ptr, len
            ret: Type::Void,
            kind: StdlibKind::HostImport {
                module: "slime",
                field: "panic",
            },
        },
    ]
}

/// Lookup a stdlib function by name. Used by the type checker and code gen.
pub fn lookup_stdlib(name: &str) -> Option<StdlibFn> {
    stdlib_functions().into_iter().find(|f| f.name == name)
}

/// Returns true if name is a stdlib function (so parser/checker don't error on undeclared calls)
pub fn is_stdlib_fn(name: &str) -> bool {
    lookup_stdlib(name).is_some()
}
