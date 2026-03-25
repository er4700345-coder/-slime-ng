# Changelog

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
