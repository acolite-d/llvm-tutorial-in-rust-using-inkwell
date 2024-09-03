# A Rust Rewrite of the LLVM Tutorial, using Inkwell
Original tutorial found here https://llvm.org/docs/tutorial/#kaleidoscope-implementing-a-language-with-llvm. Have rewritten everything up to Part 7. Code is more or less the same, but uses less global state, more modularity, more organization. There are also complete redesigns of certain aspects of the code, including an AST that does not rely on dynamic dispatch, and a robust command line interface that allows users to better visualize the process of compilation.

The code is material for these blog posts:
- [Lexer/Parser](https://find.thedoorman.xyz/building-your-own-programming-language-learning-about-compiler-design-llvm-with-a-rust-rewrite-of-the-official-llvm-tutorial-part-1-lexer-parser/)
- [IR Generation](https://find.thedoorman.xyz/building-your-own-programming-language-learning-about-compiler-design-llvm-with-a-rust-rewrite-of-the-official-llvm-tutorial-part-2-ir-generation/)
- [Optimization Passes, JIT/AoT Compilation](https://find.thedoorman.xyz/building-your-own-programming-language-with-llvm-rust-part-3-optimization-compilation/)
- [Language Extensions (if-then-else, for-loops, user-defined operators, mutable variables)](https://find.thedoorman.xyz/building-your-own-programming-language-with-llvm-rust-part-4-control-flow-user-defined-operators-mutability/)

## Building

In order to build you will need the following:

- Rust Compiler and toolchain, please use https://rustup.rs/ if not already installed.
- Clang installed, for building shared IO libraries with C. The `build.rs` script can be adapted for GCC, MSVC, or others, but currently hard-coded to invoke `clang` and build a shared library to link against. See `src/clib/io.c`
- LLVM, either built from source or installed via package manager. Code has been tested with version 17.0.6, but inkwell can support anywhere from version 4-18 at the moment. For users with a system with apt, I recommend using https://apt.llvm.org/, otherwise, follow directions here for building LLVM https://llvm.org/docs/UserGuides.html

**Be sure the installation of LLVM is locatable within your PATH.**

Code is setup as a typical Cargo project.

- Use `cargo b/build` to build.
- Use `cargo t/test` to run tests, a few are there for the frontend.
- Use `cargo r/run` to run an interpreter session, JIT compiled. To pass arguments, use `cargo run -- ` followed by whatever flags you want to pass. To compile a file instead of starting REPL, pass a file as a positional argument.

## How to Use
Project has a command line interface (built via the clap crate).

```sh
cargo run -- --help
Usage: kaleidrs [OPTIONS] [FILE]

Arguments:
  [FILE]
          A positional file containing Kaleidoscope code to compile to object/assembly, if not given, starts interpreter instead

Options:
      --target <TARGET>
        Specifies a non-native target to compile for, can be any one of the CPUs listed in "llc --version", or valid target triple

  -O, --opt-level <OPT_LEVEL>
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

  -o, --output <OUTPUT>
          When compiling a file, specifies an output file to write to
          
          [default: a.out]

  -S, --assembly
          When compiling a file, specifies the output should be assembly instead of object file

      --inspect-tree
          When interpreting, prints out AST to stdout after every line entered into interpreter

      --inspect-ir
          When interpreting, prints out the LLVM intermediate representation after every line entered into interpreter

      --inspect-asm
          When interpreting, prints out assembly to stdout after every line entered into interpreter

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```

## A Breakdown of the Features of Kaleidoscope
As per the LLVM tutorial, all aspects of language, all features, including the fleshed out ones found in the latter chapters, are implemented. For starters, observe this simple REPL session, where each line prints out IR that is JIT compiled and executed directly on the host CPU.

```sh
kaleidrs$ cargo run -- --inspect-ir
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.07s
     Running `target/debug/kaleidrs --inspect-ir`

Ready >> 2+2;

LLVM IR Representation:
; ModuleID = 'kaleidrs_module'
source_filename = "kaleidrs_module"

define double @__anonymous_expr() {
entry:
  ret double 4.000000e+00
}


Jit compiled and evaluated to: 4
Ready >> (10 - 2) * 5;

LLVM IR Representation:
; ModuleID = 'kaleidrs_module'
source_filename = "kaleidrs_module"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"

define double @__anonymous_expr() {
entry:
  ret double 4.000000e+01
}

Jit compiled and evaluated to: 40
Ready >> def dub(num) num*2;

LLVM IR Representation:
; ModuleID = 'kaleidrs_module'
source_filename = "kaleidrs_module"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"

define double @dub(double %num) {
entry:
  %multmp = fmul double %num, 2.000000e+00
  ret double %multmp
}

Ready >> dub(dub(4));

LLVM IR Representation:
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
  %calltmp = call double @dub(double 4.000000e+00)
  %calltmp1 = call double @dub(double %calltmp)
  ret double %calltmp1
}

Jit compiled and evaluated to: 16
Ready >> 
```

### External Function Definitions
You can declare C standard library functions in your program, as long as they only accept double parameters and return double values. 

```sh
kaleidrs$ cargo run -- --inspect-ir
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.46s
     Running `target/debug/kaleidrs --inspect-ir`
Ready >> extern sin(a);
LLVM IR Representation:
; ModuleID = 'kaleidrs_module'
source_filename = "kaleidrs_module"

declare double @sin(double)

Ready >> sin(45);
LLVM IR Representation:
; ModuleID = 'kaleidrs_module'
source_filename = "kaleidrs_module"

declare double @sin(double)

define double @__anonymous_expr() {
entry:
  %calltmp = call double @sin(double 4.500000e+01)
  ret double 0x3FEB3A9A073D9B03
}

Jit compiled and evaluated to: 0.8509035245341184
Ready >> extern log(n);
LLVM IR Representation:
; ModuleID = 'kaleidrs_module'
source_filename = "kaleidrs_module"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"

declare double @sin(double)

declare double @log(double)

Ready >> log(22.3);
LLVM IR Representation:
; ModuleID = 'kaleidrs_module'
source_filename = "kaleidrs_module"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"

declare double @sin(double)

declare double @log(double)

define double @__anonymous_expr() {
entry:
  %calltmp = call double @log(double 2.230000e+01)
  ret double 0x4008D6318A5CDF56
}

Jit compiled and evaluated to: 3.104586678466073
Ready >> 
```

In addition to those, there are two more functions that are compiled with the project that allow for some basic IO. These are found in `src/clib/io.c`. This code is compiled as a shared object and linked along with the Rust crate.

```sh
Ready >> extern putchard(ascii_code);
LLVM IR Representation:
; ModuleID = 'kaleidrs_module'
source_filename = "kaleidrs_module"

declare double @putchard(double)

Ready >> putchard(97);
LLVM IR Representation:
; ModuleID = 'kaleidrs_module'
source_filename = "kaleidrs_module"

declare double @putchard(double)

define double @__anonymous_expr() {
entry:
  %calltmp = call double @putchard(double 9.700000e+01)
  ret double %calltmp
}

a
Jit compiled and evaluated to: 0
Ready >> 
```

### Control Flow
Basic control flow in form of if-then-else, for-loop expression, just like the original LLVM tutorial implementation.

```sh
Ready >> def double_if_less_than(num bound) if num < bound then num*2 else num;

LLVM IR Representation:
; ModuleID = 'kaleidrs_module'
source_filename = "kaleidrs_module"

define double @double_if_less_than(double %num, double %bound) {
entry:
  %lttmp = fcmp olt double %num, %bound
  %multmp = fmul double %num, 2.000000e+00
  %iftmp = select i1 %lttmp, double %multmp, double %num
  ret double %iftmp
}

Ready >> double_if_less_than(25, 100);

LLVM IR Representation:
; ModuleID = 'kaleidrs_module'
source_filename = "kaleidrs_module"

define double @double_if_less_than(double %num, double %bound) {
entry:
  %lttmp = fcmp olt double %num, %bound
  %multmp = fmul double %num, 2.000000e+00
  %iftmp = select i1 %lttmp, double %multmp, double %num
  ret double %iftmp
}

define double @__anonymous_expr() {
entry:
  %calltmp = call double @double_if_less_than(double 2.500000e+01, double 1.000000e+02)
  ret double %calltmp
}

Jit compiled and evaluated to: 50
Ready >> double_if_less_than(700, 100);

LLVM IR Representation:
; ModuleID = 'kaleidrs_module'
source_filename = "kaleidrs_module"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"

define double @double_if_less_than(double %num, double %bound) {
entry:
  %lttmp = fcmp olt double %num, %bound
  %multmp = fmul double %num, 2.000000e+00
  %iftmp = select i1 %lttmp, double %multmp, double %num
  ret double %iftmp
}

define double @__anonymous_expr() {
entry:
  %calltmp = call double @double_if_less_than(double 7.000000e+02, double 1.000000e+02)
  ret double %calltmp
}

Jit compiled and evaluated to: 700

Ready >> for i = 0, i < 5 in putchard(97+i);

LLVM IR Representation:
; ModuleID = 'kaleidrs_module'
source_filename = "kaleidrs_module"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"

define double @double_if_less_than(double %num, double %bound) {
entry:
  %lttmp = fcmp olt double %num, %bound
  %multmp = fmul double %num, 2.000000e+00
  %iftmp = select i1 %lttmp, double %multmp, double %num
  ret double %iftmp
}

declare double @putchard(double)

define double @__anonymous_expr() {
entry:
  br label %loop

loop:                                             ; preds = %loop, %entry
  %i1 = phi double [ %nextvar, %loop ], [ 0.000000e+00, %entry ]
  %addtmp = fadd double %i1, 9.700000e+01
  %calltmp = call double @putchard(double %addtmp)
  %lttmp = fcmp olt double %i1, 5.000000e+00
  %nextvar = fadd double %i1, 1.000000e+00
  br i1 %lttmp, label %loop, label %afterloop

afterloop:                                        ; preds = %loop
  ret double 0.000000e+00
}

a
b
c
d
e
f
Jit compiled and evaluated to: 0
Ready >> 
```

### User-defined Operators
Using the "unary" and "binary" keywords, you can define your own logic upon operators in both unary and binary expressions. There are a set few operators you can implement your custom logic to. They are "!", "|", "^", "&", and ":". Please note that binary operators require a priority.

Below we implement our own bitwise negation (!) and OR (|) operators.

```
kaleidrs$ cargo r -- --inspect-ir
   Compiling kaleidrs v0.1.0 (/home/jdorman/projects/langs-test/kaleidrs)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.28s
     Running `target/debug/kaleidrs --inspect-ir`
Ready >> def unary! (V) if V then 0 else 1;
LLVM IR Representation:
; ModuleID = 'kaleidrs_module'
source_filename = "kaleidrs_module"

define double @"unary!"(double %V) {
entry:
  %ifcond = fcmp ueq double %V, 0.000000e+00
  %. = select i1 %ifcond, double 1.000000e+00, double 0.000000e+00
  ret double %.
}

Ready >> !1;
LLVM IR Representation:
; ModuleID = 'kaleidrs_module'
source_filename = "kaleidrs_module"

define double @"unary!"(double %V) {
entry:
  %ifcond = fcmp ueq double %V, 0.000000e+00
  %. = select i1 %ifcond, double 1.000000e+00, double 0.000000e+00
  ret double %.
}

define double @__anonymous_expr() {
entry:
  %unarytmp = call double @"unary!"(double 1.000000e+00)
  ret double %unarytmp
}

Jit compiled and evaluated to: 0
Ready >> !0;
LLVM IR Representation:
; ModuleID = 'kaleidrs_module'
source_filename = "kaleidrs_module"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"

define double @"unary!"(double %V) {
entry:
  %ifcond = fcmp ueq double %V, 0.000000e+00
  %. = select i1 %ifcond, double 1.000000e+00, double 0.000000e+00
  ret double %.
}

define double @__anonymous_expr() {
entry:
  %unarytmp = call double @"unary!"(double 0.000000e+00)
  ret double %unarytmp
}

Jit compiled and evaluated to: 1
Ready >> 
```

```
kaleidrs$ cargo r -- --inspect-ir
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.27s
     Running `target/debug/kaleidrs --inspect-ir`
Ready >> def binary| 5 (LHS RHS) if LHS then 1 else if RHS then 1 else 0;
LLVM IR Representation:
; ModuleID = 'kaleidrs_module'
source_filename = "kaleidrs_module"

define double @"binary|"(double %LHS, double %RHS) {
entry:
  %ifcond = fcmp ueq double %LHS, 0.000000e+00
  %ifcond5 = fcmp ueq double %RHS, 0.000000e+00
  %. = select i1 %ifcond5, double 0.000000e+00, double 1.000000e+00
  %iftmp9 = select i1 %ifcond, double %., double 1.000000e+00
  ret double %iftmp9
}

Ready >> (1 | 0);
LLVM IR Representation:
; ModuleID = 'kaleidrs_module'
source_filename = "kaleidrs_module"

define double @"binary|"(double %LHS, double %RHS) {
entry:
  %ifcond = fcmp ueq double %LHS, 0.000000e+00
  %ifcond5 = fcmp ueq double %RHS, 0.000000e+00
  %0 = select i1 %ifcond, i1 %ifcond5, i1 false
  %iftmp9 = select i1 %0, double 0.000000e+00, double 1.000000e+00
  ret double %iftmp9
}

define double @__anonymous_expr() {
entry:
  %calltmp = call double @"binary|"(double 1.000000e+00, double 0.000000e+00)
  ret double %calltmp
}

Jit compiled and evaluated to: 1
Ready >> (0 | 0);
LLVM IR Representation:
; ModuleID = 'kaleidrs_module'
source_filename = "kaleidrs_module"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"

define double @"binary|"(double %LHS, double %RHS) {
entry:
  %ifcond = fcmp ueq double %LHS, 0.000000e+00
  %ifcond5 = fcmp ueq double %RHS, 0.000000e+00
  %0 = select i1 %ifcond, i1 %ifcond5, i1 false
  %iftmp9 = select i1 %0, double 0.000000e+00, double 1.000000e+00
  ret double %iftmp9
}

define double @__anonymous_expr() {
entry:
  %calltmp = call double @"binary|"(double 0.000000e+00, double 0.000000e+00)
  ret double %calltmp
}

Jit compiled and evaluated to: 0
Ready >> 
```

### Mutable Variables
All variables are mutable, as per the original C++ implementation. User-defined variables also possible with "var" keyword. Supply a comma separated list of variables names and possible initializers. The absence of an initializer sets the value to 1.

```
Ready >> var x = 3, y = 3, z in x = z;
LLVM IR Representation:
; ModuleID = 'kaleidrs_module'
source_filename = "kaleidrs_module"

define double @__anonymous_expr() {
entry:
  ret double 1.000000e+00
}

Jit compiled and evaluated to: 1
Ready >>   
```

### Other Cool Things You Can Do
The language itself is no different than the original tutorial implementation, but there is some additional tooling in form of a CLI that allow you to configure different parts of compilation to compare and contrast. One of the more interesting features is the ability to freely inspect the abstract syntax tree, IR, and final assembly code after every line entered in the REPL using the `--inspect-*` flags.

```sh
kaleidrs$ cargo run -- --inspect-tree --inspect-ir --inspect-asm
   Compiling kaleidrs v0.1.0 (/home/jdorman/projects/langs-test/kaleidrs)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 4.54s
     Running `target/debug/kaleidrs --inspect-tree --inspect-ir --inspect-asm`
Ready >> def fibonacci(n) if n < 3 then 1 else fibonacci(n-1)+fibonacci(n-2);
Abstract Syntax Tree Representation:
Function {
    proto: FunctionProto {
        name: "fibonacci",
        args: [
            "n",
        ],
    },
    body: IfExpr {
        cond: BinaryExpr {
            op: Lt,
            left: VariableExpr(
                "n",
            ),
            right: NumberExpr(
                3.0,
            ),
        },
        then_branch: NumberExpr(
            1.0,
        ),
        else_branch: BinaryExpr {
            op: Plus,
            left: CallExpr {
                callee: "fibonacci",
                args: [
                    BinaryExpr {
                        op: Minus,
                        left: VariableExpr(
                            "n",
                        ),
                        right: NumberExpr(
                            1.0,
                        ),
                    },
                ],
            },
            right: CallExpr {
                callee: "fibonacci",
                args: [
                    BinaryExpr {
                        op: Minus,
                        left: VariableExpr(
                            "n",
                        ),
                        right: NumberExpr(
                            2.0,
                        ),
                    },
                ],
            },
        },
    },
}

LLVM IR Representation:
; ModuleID = 'kaleidrs_module'
source_filename = "kaleidrs_module"

define double @fibonacci(double %n) {
entry:
  %lttmp = fcmp olt double %n, 3.000000e+00
  br i1 %lttmp, label %ifcont, label %else

else:                                             ; preds = %entry
  %subtmp = fadd double %n, -1.000000e+00
  %calltmp = call double @fibonacci(double %subtmp)
  %subtmp5 = fadd double %n, -2.000000e+00
  %calltmp6 = call double @fibonacci(double %subtmp5)
  %addtmp = fadd double %calltmp, %calltmp6
  br label %ifcont

ifcont:                                           ; preds = %entry, %else
  %iftmp = phi double [ %addtmp, %else ], [ 1.000000e+00, %entry ]
  ret double %iftmp
}

Assembly Representation:
        .text
        .file   "kaleidrs_module"
        .section        .rodata.cst8,"aM",@progbits,8
        .p2align        3, 0x0
.LCPI0_0:
        .quad   0x3ff0000000000000
.LCPI0_1:
        .quad   0x4008000000000000
.LCPI0_2:
        .quad   0xbff0000000000000
.LCPI0_3:
        .quad   0xc000000000000000
        .text
        .globl  fibonacci
        .p2align        4, 0x90
        .type   fibonacci,@function
fibonacci:
        .cfi_startproc
        movapd  %xmm0, %xmm1
        movsd   .LCPI0_1(%rip), %xmm0
        ucomisd %xmm1, %xmm0
        jbe     .LBB0_2
        movsd   .LCPI0_0(%rip), %xmm0
        retq
.LBB0_2:
        subq    $24, %rsp
        .cfi_def_cfa_offset 32
        movsd   .LCPI0_2(%rip), %xmm0
        addsd   %xmm1, %xmm0
        movsd   %xmm1, 8(%rsp)
        callq   fibonacci@PLT
        movsd   %xmm0, 16(%rsp)
        movsd   8(%rsp), %xmm0
        addsd   .LCPI0_3(%rip), %xmm0
        callq   fibonacci@PLT
        addsd   16(%rsp), %xmm0
        addq    $24, %rsp
        .cfi_def_cfa_offset 8
        retq
.Lfunc_end0:
        .size   fibonacci, .Lfunc_end0-fibonacci
        .cfi_endproc

        .section        ".note.GNU-stack","",@progbits
```

You can also configure the LLVM optimization levels with the `-O{0,1,2,3}, --opt-level` flags, and even pass specific LLVM optimization passes using the `-p, --passes` flag. This is a great feature to use in tandem with the inspect flags to see how passes and levels affect the final product that is run on the CPU in the JIT interpreted session. Great for experimentation.

```sh
kaleidrs$ cargo run -- --inspect-ir --inspect-asm  --passes ""
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.04s
     Running `target/debug/kaleidrs --inspect-ir --inspect-asm --passes ''`
Ready >> def fibonacci(n) if n < 3 then 1 else fibonacci(n-1)+fibonacci(n-2);
LLVM IR Representation:
; ModuleID = 'kaleidrs_module'
source_filename = "kaleidrs_module"

define double @fibonacci(double %n) {
entry:
  %n1 = alloca double, align 8
  store double %n, ptr %n1, align 8
  %n2 = load double, ptr %n1, align 8
  %lttmp = fcmp olt double %n2, 3.000000e+00
  %booltmp = uitofp i1 %lttmp to double
  %iftemp = fcmp oeq double %booltmp, 1.000000e+00
  br i1 %iftemp, label %then, label %else

then:                                             ; preds = %entry
  br label %ifcont

else:                                             ; preds = %entry
  %n3 = load double, ptr %n1, align 8
  %subtmp = fsub double %n3, 1.000000e+00
  %calltmp = call double @fibonacci(double %subtmp)
  %n4 = load double, ptr %n1, align 8
  %subtmp5 = fsub double %n4, 2.000000e+00
  %calltmp6 = call double @fibonacci(double %subtmp5)
  %addtmp = fadd double %calltmp, %calltmp6
  br label %ifcont

ifcont:                                           ; preds = %else, %then
  %iftmp = phi double [ 1.000000e+00, %then ], [ %addtmp, %else ]
  ret double %iftmp
}

Assembly Representation:
        .text
        .file   "kaleidrs_module"
        .section        .rodata.cst8,"aM",@progbits,8
        .p2align        3, 0x0
.LCPI0_0:
        .quad   0x3ff0000000000000
.LCPI0_1:
        .quad   0x4008000000000000
.LCPI0_2:
        .quad   0xbff0000000000000
.LCPI0_3:
        .quad   0xc000000000000000
        .text
        .globl  fibonacci
        .p2align        4, 0x90
        .type   fibonacci,@function
fibonacci:
        .cfi_startproc
        subq    $24, %rsp
        .cfi_def_cfa_offset 32
        movapd  %xmm0, %xmm1
        movsd   %xmm0, 8(%rsp)
        cmpltsd .LCPI0_1(%rip), %xmm1
        movsd   .LCPI0_0(%rip), %xmm0
        andpd   %xmm0, %xmm1
        ucomisd %xmm0, %xmm1
        jne     .LBB0_1
        jp      .LBB0_1
        addq    $24, %rsp
        .cfi_def_cfa_offset 8
        retq
.LBB0_1:
        .cfi_def_cfa_offset 32
        movsd   8(%rsp), %xmm0
        addsd   .LCPI0_2(%rip), %xmm0
        callq   fibonacci@PLT
        movsd   %xmm0, 16(%rsp)
        movsd   8(%rsp), %xmm0
        addsd   .LCPI0_3(%rip), %xmm0
        callq   fibonacci@PLT
        addsd   16(%rsp), %xmm0
        addq    $24, %rsp
        .cfi_def_cfa_offset 8
        retq
.Lfunc_end0:
        .size   fibonacci, .Lfunc_end0-fibonacci
        .cfi_endproc

        .section        ".note.GNU-stack","",@progbits


Ready >> ^C
kaleidrs$ cargo run -- --inspect-ir --inspect-asm  --passes "instcombine,reassociate,gvn,simplifycfg,mem2reg"
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.04s
     Running `target/debug/kaleidrs --inspect-ir --inspect-asm --passes instcombine,reassociate,gvn,simplifycfg,mem2reg`
Ready >> def fibonacci(n) if n < 3 then 1 else fibonacci(n-1)+fibonacci(n-2);
LLVM IR Representation:
; ModuleID = 'kaleidrs_module'
source_filename = "kaleidrs_module"

define double @fibonacci(double %n) {
entry:
  %lttmp = fcmp olt double %n, 3.000000e+00
  br i1 %lttmp, label %ifcont, label %else

else:                                             ; preds = %entry
  %subtmp = fadd double %n, -1.000000e+00
  %calltmp = call double @fibonacci(double %subtmp)
  %subtmp5 = fadd double %n, -2.000000e+00
  %calltmp6 = call double @fibonacci(double %subtmp5)
  %addtmp = fadd double %calltmp, %calltmp6
  br label %ifcont

ifcont:                                           ; preds = %entry, %else
  %iftmp = phi double [ %addtmp, %else ], [ 1.000000e+00, %entry ]
  ret double %iftmp
}

Assembly Representation:
        .text
        .file   "kaleidrs_module"
        .section        .rodata.cst8,"aM",@progbits,8
        .p2align        3, 0x0
.LCPI0_0:
        .quad   0x3ff0000000000000
.LCPI0_1:
        .quad   0x4008000000000000
.LCPI0_2:
        .quad   0xbff0000000000000
.LCPI0_3:
        .quad   0xc000000000000000
        .text
        .globl  fibonacci
        .p2align        4, 0x90
        .type   fibonacci,@function
fibonacci:
        .cfi_startproc
        movapd  %xmm0, %xmm1
        movsd   .LCPI0_1(%rip), %xmm0
        ucomisd %xmm1, %xmm0
        jbe     .LBB0_2
        movsd   .LCPI0_0(%rip), %xmm0
        retq
.LBB0_2:
        subq    $24, %rsp
        .cfi_def_cfa_offset 32
        movsd   .LCPI0_2(%rip), %xmm0
        addsd   %xmm1, %xmm0
        movsd   %xmm1, 8(%rsp)
        callq   fibonacci@PLT
        movsd   %xmm0, 16(%rsp)
        movsd   8(%rsp), %xmm0
        addsd   .LCPI0_3(%rip), %xmm0
        callq   fibonacci@PLT
        addsd   16(%rsp), %xmm0
        addq    $24, %rsp
        .cfi_def_cfa_offset 8
        retq
.Lfunc_end0:
        .size   fibonacci, .Lfunc_end0-fibonacci
        .cfi_endproc

        .section        ".note.GNU-stack","",@progbits


Ready >> 
```

In addition, the `--target` flag will allow you to cross compile to whatever CPU or target triple LLVM supports. You can then compare the same program compiled to different targets, and see optimizations take place over various CPU architectures. From ARM to RISC-V to WASM to SPARC and everything in between. Not just the native architecture your computer runs on.

```
kaleidrs$ cat test.ks
def fibonacci(n)
    if n < 3 then 
        1 
    else 
        fibonacci(n-1) + fibonacci(n-2)
;

fibonacci(10);
kaleidrs$ cargo run -- test.ks --target armv7a-none-eabi -S -o test.S
   Compiling kaleidrs v0.1.0 (/home/jdorman/projects/langs-test/kaleidrs)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.65s
     Running `target/debug/kaleidrs test.ks --target armv7a-none-eabi -S -o test.S`
kaleidrs$ cat test.S
        .text
        .syntax unified
        .eabi_attribute 67, "2.09"
        .eabi_attribute 6, 10
        .eabi_attribute 7, 65
        .eabi_attribute 8, 1
        .eabi_attribute 9, 2
        .fpu    vfpv3
        .eabi_attribute 34, 1
        .eabi_attribute 17, 1
        .eabi_attribute 20, 1
        .eabi_attribute 21, 0
        .eabi_attribute 23, 3
        .eabi_attribute 24, 1
        .eabi_attribute 25, 1
        .eabi_attribute 38, 1
        .eabi_attribute 14, 0
        .file   "kaleidrs_module"
        .globl  fibonacci
        .p2align        2
        .type   fibonacci,%function
        .code   32
fibonacci:
        .fnstart
        push    {r11, lr}
        vpush   {d8}
        vmov.f64        d16, #3.000000e+00
        vmov    d8, r0, r1
        vcmp.f64        d8, d16
        vmrs    APSR_nzcv, fpscr
        bpl     .LBB0_2
        vmov.f64        d16, #1.000000e+00
        b       .LBB0_3
.LBB0_2:
        vmov.f64        d16, #-1.000000e+00
        vadd.f64        d16, d8, d16
        vmov    r0, r1, d16
        bl      fibonacci
        vmov.f64        d16, #-2.000000e+00
        vadd.f64        d16, d8, d16
        vmov    r2, r3, d16
        vmov    d8, r0, r1
        mov     r0, r2
        mov     r1, r3
        bl      fibonacci
        vmov    d16, r0, r1
        vadd.f64        d16, d8, d16
.LBB0_3:
        vmov    r0, r1, d16
        vpop    {d8}
        pop     {r11, pc}
.Lfunc_end0:
        .size   fibonacci, .Lfunc_end0-fibonacci
        .fnend

        .globl  __anonymous_expr
        .p2align        2
        .type   __anonymous_expr,%function
        .code   32
__anonymous_expr:
        .fnstart
        push    {r11, lr}
        vmov.f64        d16, #1.000000e+01
        vmov    r0, r1, d16
        bl      fibonacci
        pop     {r11, pc}
.Lfunc_end1:
        .size   __anonymous_expr, .Lfunc_end1-__anonymous_expr
        .fnend

        .section        ".note.GNU-stack","",%progbits
```
