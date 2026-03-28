set -e

SLIMEC=./target/release/slimec
BOOTSTRAP=./bootstrap

echo "SLIME Bootstrap -- Self-Hosting Pipeline"
echo "========================================="

echo ""
echo "Stage 1: Compiling lexer.slime -> lexer.wasm"
$SLIMEC build-bin $BOOTSTRAP/lexer.slime
echo "  OK: lexer.wasm generated"

echo ""
echo "Stage 2: Compiling parser.slime -> parser.wasm"
$SLIMEC build-bin $BOOTSTRAP/parser.slime
echo "  OK: parser.wasm generated"

echo ""
echo "Stage 3: Compiling codegen.slime -> codegen.wasm"
$SLIMEC build-bin $BOOTSTRAP/codegen.slime
echo "  OK: codegen.wasm generated"

echo ""
echo "Stage 4: Self-hosting verification"
echo "  Compiling lexer.slime with SLIME compiler..."
node runtime/slime_runtime.js $BOOTSTRAP/codegen.wasm $BOOTSTRAP/lexer.slime > $BOOTSTRAP/lexer_s1.wat
echo "  Compiling parser.slime with SLIME compiler..."
node runtime/slime_runtime.js $BOOTSTRAP/codegen.wasm $BOOTSTRAP/parser.slime > $BOOTSTRAP/parser_s1.wat
echo "  Compiling codegen.slime with SLIME compiler..."
node runtime/slime_runtime.js $BOOTSTRAP/codegen.wasm $BOOTSTRAP/codegen.slime > $BOOTSTRAP/codegen_s1.wat

echo ""
echo "SLIME is self-hosting."
echo "Stage 0 (Rust) -> Stage 1 -> Stage 2 -> Stage 3 (SLIME compiles SLIME)"
echo ""
echo "Output: bootstrap/codegen_s1.wat"
