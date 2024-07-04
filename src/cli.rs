use std::path::PathBuf;

use clap::{
    builder::{OsStr, PossibleValue},
    Parser, ValueEnum,
};
use inkwell;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// A positional file containing Kaleidoscope code to compile to object/assembly, if not given, starts interpreter instead
    pub file: Option<PathBuf>,

    /// What optimization level to pass to LLVM
    #[arg(long, value_enum, default_value = OptLevel::O2)]
    pub opt_level: OptLevel,

    /// Comma separated list of LLVM passes (use opt for a list, also see https://www.llvm.org/docs/Passes.html)
    #[arg(
        short,
        long,
        default_value = "instcombine,reassociate,gvn,simplifycfg,mem2reg"
    )]
    pub passes: String,

    /// When AOT compiling, specifies an output file to write to
    #[arg(short, long, default_value = "a.out")]
    pub output: PathBuf,

    /// When AOT compiling, specifies the output should be assembly instead of object file
    #[arg(short = 'S', long = "assembly")]
    pub asm_p: bool,

    /// When JIT compiling, prints out AST to stdout after every line entered into interpreter
    #[arg(long)]
    pub inspect_tree: bool,

    /// When JIT compiling, prints out assembly to stdout after every line entered into interpreter
    #[arg(long)]
    pub inspect_asm: bool,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum OptLevel {
    O0,
    O1,
    O2,
    O3,
}

impl ValueEnum for OptLevel {
    fn value_variants<'a>() -> &'a [Self] {
        &[OptLevel::O0, OptLevel::O1, OptLevel::O2, OptLevel::O3]
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        Some(match self {
            OptLevel::O0 => PossibleValue::new("O0").help("No optimization"),
            OptLevel::O1 => PossibleValue::new("O1").help("Less optimization"),
            OptLevel::O2 => PossibleValue::new("O2").help("Default optimization"),
            OptLevel::O3 => PossibleValue::new("O3").help("Aggressive optimization"),
        })
    }
}

impl Into<OsStr> for OptLevel {
    fn into(self) -> OsStr {
        match self {
            OptLevel::O0 => "O0".into(),
            OptLevel::O1 => "O1".into(),
            OptLevel::O2 => "O2".into(),
            OptLevel::O3 => "O3".into(),
        }
    }
}

// Convert to a inkwell optimization level, reflection of an actual LLVM level
impl Into<inkwell::OptimizationLevel> for OptLevel {
    fn into(self) -> inkwell::OptimizationLevel {
        match self {
            OptLevel::O0 => inkwell::OptimizationLevel::None,
            OptLevel::O1 => inkwell::OptimizationLevel::Less,
            OptLevel::O2 => inkwell::OptimizationLevel::Default,
            OptLevel::O3 => inkwell::OptimizationLevel::Aggressive,
        }
    }
}
