# SLIME Changelog

## [0.2.0-wasm-foundation] - 2026-05-17

### Added
- Full source → WASM pipeline (lexer → parser → typechecker → IR → WASM)
- IR foundation (Program, Function, BasicBlock, Instruction, Value, Type)
- WASM validation with wasmparser
- wasmi execution tests for return, binary, function calls
- Stdlib host imports: print, len, to_string, input (env.*)
- Binary operations: +, -, *, /, ==, !=
- CLI: `slimec compile <file.slime> --emit wasm`
- End-to-end examples and tests for all stdlib builtins
- Module imports/exports organization
- Placeholder removal and honest error handling

### Changed
- WASM codegen strengthened with real IR-driven lowering
- CLI pipeline integrated with full stdlib support

### Limitations (documented)
- No native backend yet
- Stdlib uses host imports (no full runtime)
- Module/import system not finalized
- Optimizations are minimal
- No self-hosting yet

[0.2.0-wasm-foundation]: https://github.com/er4700345-coder/-slime-ng/releases/tag/v0.2.0-wasm-foundation

## [0.1.0] - 2026-03-25

### Added
- Initial lexer with full token set
- Recursive descent parser
- Hindley-Milner type checker
- WASM text format backend
- CLI tool with lex, parse, check, compile, build commands
- Basic error reporting with line/column info

### Known Issues
- Binary WASM output not yet implemented
- Limited standard library
- No native backend
- String concatenation not type-checked properly
