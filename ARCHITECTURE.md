# SLIME Architecture

SLIME is a compiler and toolchain project designed for **interactive multiplatform execution**.

---

## Compilation Pipeline

SLIME currently follows this high-level pipeline:

```text
Source Code
→ Lexer
→ Tokens
→ Parser
→ AST
→ Type Checker
→ Backend
