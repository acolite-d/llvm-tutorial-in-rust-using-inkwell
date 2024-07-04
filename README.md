# A Rust Rewrite of the LLVM Tutorial, using Inkwell
Original tutorial found here https://llvm.org/docs/tutorial/#kaleidoscope-implementing-a-language-with-llvm. Have rewritten everything up to Part 7. Code is more or less the same, but uses less global state, more modularity, more organization. There are also complete redesigns of certain aspects of the code, including an AST that does not rely on dynamic dispatch, and a robust command line interface that allows users to better visualize the process of compilation.

The code is material for these blog posts:
- [Lexer/Parser](https://find.thedoorman.xyz/building-your-own-programming-language-learning-about-compiler-design-llvm-with-a-rust-rewrite-of-the-official-llvm-tutorial-part-1-lexer-parser/)
- [IR Generation](https://find.thedoorman.xyz/building-your-own-programming-language-learning-about-compiler-design-llvm-with-a-rust-rewrite-of-the-official-llvm-tutorial-part-2-ir-generation/)
- (More to come soon)

## Building

In order to build you will need the following:

- Rust Compiler and toolchain, please use https://rustup.rs/ if not already installed.
- LLVM, either built from source or installed via package manager. Code has been tested with version 17.0.6, but inkwell can support anywhere from version 4-18 at the moment. For users with a system with apt, I recommend using https://apt.llvm.org/, otherwise, follow directions here for building LLVM https://llvm.org/docs/UserGuides.html

**Be sure the installation of LLVM is locatable within your PATH.**

Code is setup as a typical Cargo project.

- Use `cargo build` to build.
- Use `cargo test` to run tests, a few are there.
- Use `cargo run` to run an interpreter session, JIT compiled. Prints IR, along with what was evaluated when the IR was JIT compiled and executed.

## How to Use
Project has a command line interface (built via the clap crate).

```sh
cargo run -- --help
Usage: kaleidrs [OPTIONS] [FILE]

Arguments:
  [FILE]
          A positional file containing Kaleidoscope code to compile to object code, if not given, starts interpreter instead

Options:
      --opt-level <OPT_LEVEL>
          What optimization level to pass to LLVM
          
          [default: O2]

          Possible values:
          - O0: No optimization
          - O1: Less optimization
          - O2: Default optimization
          - O3: Aggressive optimization

  -p, --passes <PASSES>
          Comma separated list of LLVM passes (use opt for a list, also see https://www.llvm.org/docs/Passes.html)
          
          [default: instcombine,reassociate,gvn,simplifycfg,mem2reg]

      --use-frontend-only
          Interpret with frontend only, output AST, only valid for interpreter use

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```

## A Breakdown of the Features of Kaleidoscope
As per the LLVM tutorial, all features, including the fleshed out ones found in the latter chapters are implemented.

### Basic Control Flow

### Unary/Binary Operator Overloading

### Mutable Variables


## Example

```sh
kaleidrs$ cargo run
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.03s
     Running `target/debug/kaleidrs`
Ready >> 2+2;
Parsed a top level expression.
Running pass: VerifierPass on [module]
Running analysis: VerifierAnalysis on [module]
Running analysis: InnerAnalysisManagerProxy<FunctionAnalysisManager, Module> on [module]
Running pass: InstCombinePass on __anonymous_expr (1 instruction)
Running analysis: AssumptionAnalysis on __anonymous_expr
Running analysis: TargetIRAnalysis on __anonymous_expr
Running analysis: DominatorTreeAnalysis on __anonymous_expr
Running analysis: TargetLibraryAnalysis on __anonymous_expr
Running analysis: OptimizationRemarkEmitterAnalysis on __anonymous_expr
Running analysis: AAManager on __anonymous_expr
Running analysis: BasicAA on __anonymous_expr
Running analysis: ScopedNoAliasAA on __anonymous_expr
Running analysis: TypeBasedAA on __anonymous_expr
Running analysis: OuterAnalysisManagerProxy<ModuleAnalysisManager, Function> on __anonymous_expr
Verifying function __anonymous_expr
Running pass: ReassociatePass on __anonymous_expr (1 instruction)
Verifying function __anonymous_expr
Running pass: GVNPass on __anonymous_expr (1 instruction)
Running analysis: MemoryDependenceAnalysis on __anonymous_expr
Verifying function __anonymous_expr
Running pass: SimplifyCFGPass on __anonymous_expr (1 instruction)
Verifying function __anonymous_expr
; ModuleID = 'kaleidrs_module'
source_filename = "kaleidrs_module"

define double @__anonymous_expr() {
entry:
  ret double 4.000000e+00
}
Jit compiled and evaluated to: 4
Ready >> (10/2) * 5;
Parsed a top level expression.
Running pass: VerifierPass on [module]
Running analysis: VerifierAnalysis on [module]
Running analysis: InnerAnalysisManagerProxy<FunctionAnalysisManager, Module> on [module]
Running pass: InstCombinePass on __anonymous_expr (1 instruction)
Running analysis: AssumptionAnalysis on __anonymous_expr
Running analysis: TargetIRAnalysis on __anonymous_expr
Running analysis: DominatorTreeAnalysis on __anonymous_expr
Running analysis: TargetLibraryAnalysis on __anonymous_expr
Running analysis: OptimizationRemarkEmitterAnalysis on __anonymous_expr
Running analysis: AAManager on __anonymous_expr
Running analysis: BasicAA on __anonymous_expr
Running analysis: ScopedNoAliasAA on __anonymous_expr
Running analysis: TypeBasedAA on __anonymous_expr
Running analysis: OuterAnalysisManagerProxy<ModuleAnalysisManager, Function> on __anonymous_expr
Verifying function __anonymous_expr
Running pass: ReassociatePass on __anonymous_expr (1 instruction)
Verifying function __anonymous_expr
Running pass: GVNPass on __anonymous_expr (1 instruction)
Running analysis: MemoryDependenceAnalysis on __anonymous_expr
Verifying function __anonymous_expr
Running pass: SimplifyCFGPass on __anonymous_expr (1 instruction)
Verifying function __anonymous_expr
; ModuleID = 'kaleidrs_module'
source_filename = "kaleidrs_module"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"

define double @__anonymous_expr() {
entry:
  ret double 2.500000e+01
}
Jit compiled and evaluated to: 25
Ready >> def dub(num) num*2;
Parsed a function definition.
; ModuleID = 'kaleidrs_module'
source_filename = "kaleidrs_module"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"

define double @dub(double %num) {
entry:
  %multmp = fmul double %num, 2.000000e+00
  ret double %multmp
}
Ready >> dub(dub(100));
Parsed a top level expression.
Running pass: VerifierPass on [module]
Running analysis: VerifierAnalysis on [module]
Running analysis: InnerAnalysisManagerProxy<FunctionAnalysisManager, Module> on [module]
Running pass: InstCombinePass on dub (2 instructions)
Running analysis: AssumptionAnalysis on dub
Running analysis: TargetIRAnalysis on dub
Running analysis: DominatorTreeAnalysis on dub
Running analysis: TargetLibraryAnalysis on dub
Running analysis: OptimizationRemarkEmitterAnalysis on dub
Running analysis: AAManager on dub
Running analysis: BasicAA on dub
Running analysis: ScopedNoAliasAA on dub
Running analysis: TypeBasedAA on dub
Running analysis: OuterAnalysisManagerProxy<ModuleAnalysisManager, Function> on dub
Verifying function dub
Running pass: ReassociatePass on dub (2 instructions)
Verifying function dub
Running pass: GVNPass on dub (2 instructions)
Running analysis: MemoryDependenceAnalysis on dub
Verifying function dub
Running pass: SimplifyCFGPass on dub (2 instructions)
Verifying function dub
Running pass: InstCombinePass on __anonymous_expr (3 instructions)
Running analysis: AssumptionAnalysis on __anonymous_expr
Running analysis: TargetIRAnalysis on __anonymous_expr
Running analysis: DominatorTreeAnalysis on __anonymous_expr
Running analysis: TargetLibraryAnalysis on __anonymous_expr
Running analysis: OptimizationRemarkEmitterAnalysis on __anonymous_expr
Running analysis: AAManager on __anonymous_expr
Running analysis: BasicAA on __anonymous_expr
Running analysis: ScopedNoAliasAA on __anonymous_expr
Running analysis: TypeBasedAA on __anonymous_expr
Running analysis: OuterAnalysisManagerProxy<ModuleAnalysisManager, Function> on __anonymous_expr
Verifying function __anonymous_expr
Running pass: ReassociatePass on __anonymous_expr (3 instructions)
Verifying function __anonymous_expr
Running pass: GVNPass on __anonymous_expr (3 instructions)
Running analysis: MemoryDependenceAnalysis on __anonymous_expr
Verifying function __anonymous_expr
Running pass: SimplifyCFGPass on __anonymous_expr (3 instructions)
Verifying function __anonymous_expr
; ModuleID = 'kaleidrs_module'
source_filename = "kaleidrs_module"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"

define double @dub(double %num) {
entry:
  %multmp = fmul double %num, 2.000000e+00
  ret double %multmp
}

define double @__anonymous_expr() {
entry:
  %calltmp = call double @dub(double 1.000000e+02)
  %calltmp1 = call double @dub(double %calltmp)
  ret double %calltmp1
}
Jit compiled and evaluated to: 400
Ready >>
```

