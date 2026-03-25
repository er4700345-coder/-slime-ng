# SLIME Lexer Specification

## Token Types

| Token | Regex | Example |
|-------|-------|---------|
| FN | `fn` | `fn` |
| LET | `let` | `let` |
| RETURN | `return` | `return` |
| TARGET | `target` | `target` |
| TRUE | `true` | `true` |
| FALSE | `false` | `false` |
| IF | `if` | `if` |
| ELSE | `else` | `else` |
| WHILE | `while` | `while` |
| IDENTIFIER | `[a-zA-Z_][a-zA-Z0-9_]*` | `main`, `x1` |
| INTEGER | `[0-9]+` | `42`, `0` |
| FLOAT | `[0-9]+\.[0-9]+` | `3.14` |
| STRING | `"[^"]*"` | `"hello"` |
| ARROW | `->` | `->` |
| EQ | `==` | `==` |
| NEQ | `!=` | `!=` |
| LTE | `<=` | `<=` |
| GTE | `>=` | `>=` |
| AND | `&&` | `&&` |
| OR | `\|\|` | `\|\|` |
| ASSIGN | `=` | `=` |
| PLUS | `+` | `+` |
| MINUS | `-` | `-` |
| STAR | `*` | `*` |
| SLASH | `/` | `/` |
| LT | `<` | `<` |
| GT | `>` | `>` |
| BANG | `!` | `!` |
| LPAREN | `(` | `(` |
| RPAREN | `)` | `)` |
| LBRACE | `{` | `{` |
| RBRACE | `}` | `}` |
| LBRACKET | `[` | `[` |
| RBRACKET | `]` | `]` |
| SEMICOLON | `;` | `;` |
| COLON | `:` | `:` |
| COMMA | `,` | `,` |
| DOT | `.` | `.` |
| EOF | (end of file) | — |

## Keywords

Reserved: `fn`, `let`, `return`, `target`, `true`, `false`, `if`, `else`, `while`, `i32`, `i64`, `f32`, `f64`, `bool`, `string`, `void`

## Example Token Stream

Input:
fn main() {  let x = 42; }
Tokens:
FN, IDENTIFIER("main"), LPAREN, RPAREN, LBRACE, LET, IDENTIFIER("x"), ASSIGN, INTEGER(42), SEMICOLON, RBRACE, EOF

## Error Handling

- Invalid character: report line, column, character
- Unterminated string: report start position
