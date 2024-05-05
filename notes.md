# Kaleidrs, The Process of Learning Compilers through both Rust and the LLVM tutorial

## Lexer
- Go into basics here, explaining first reading a string a producing meaningful tokens
- Expand tokens from just tags to invariants with tagged unions, showcase String vs. &str for identifier
- Showcase tests
- Showcase expanding functionality with BufRead

## Parser
- Demonstrate the struggle of a static hashmap, go over compiler messages suggesting third party crates, lazy_static, once_cell
- Demonstrate the obstacles of using trait objects for AST, the question of how to equate trees for testing purposes, the Any trait, type erasure downcasting. Then demonstrate how an Enum dispatch model might work better.
- Talk about how tokens are passed in C++ example with globals, equate it to Rust. Talk about peekable iterators, consuming vs non-consuming calls.