use std::cell::RefCell;
use std::collections::HashMap;

use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::execution_engine::JitFunction;
use inkwell::module::{Linkage, Module};
use inkwell::targets::{Target, TargetMachine, RelocMode, CodeModel};
use inkwell::types::BasicMetadataTypeEnum;
use inkwell::values::{AnyValue, AnyValueEnum, BasicMetadataValueEnum, BasicValue};
use inkwell::OptimizationLevel;
use inkwell::passes::PassBuilderOptions;
use thiserror::Error;

use crate::frontend::ast::{ASTExpr, Prototype, Function};
use crate::frontend::lexer::Ops;

type IRGenResult<'ir, 'src> = Result<AnyValueEnum<'ir>, BackendError<'src>>;
type TopLevelSignature = unsafe extern "C" fn() -> f64;

// Possible errors that might result when generating/JIT'ing
// LLVM IR
#[derive(Error, PartialEq, Debug)]
pub enum BackendError<'src> {
    #[error("Unknown variable name {0}")]
    UnknownVariable(&'src str),

    #[error("Undefined function {0}")]
    UndefinedFunction(&'src str),

    #[error("Function {0} defined twice")]
    MultipleFunctionDefs(&'src str),

    #[error("Incorrect number of arguments passed to {func_name}, expected {param_cnt}")]
    IncorrectNumberOfArgs { func_name: &'src str, param_cnt: u32 },

    #[error("LLVM failed to verify function {0}")]
    FailedToVerifyFunc(&'src str),

    #[error("Failed to JIT top level function expression!")]
    FailedToJIT,
}

// Our context object that we will pass to recursive calls of codegen
// as we generate LLVM IR from our tree.
#[derive(Debug)]
pub struct LLVMContext<'ctx> {
    context: &'ctx Context,
    builder: Builder<'ctx>,
    module: Module<'ctx>,
    target: Target,
    machine: TargetMachine,
    sym_table: RefCell<HashMap<String, AnyValueEnum<'ctx>>>,
}

impl<'ctx> LLVMContext<'ctx> {
    pub fn new(context: &'ctx Context) -> Self {
        let builder = context.create_builder();
        let module = context.create_module("kaleidrs_module");

        let triple = TargetMachine::get_default_triple();
        let target = Target::from_triple(&triple).unwrap();

        let machine = target
            .create_target_machine(
                &triple, 
                "generic", 
                "", 
                OptimizationLevel::None, 
                RelocMode::Default, 
                CodeModel::Default
            )
            .unwrap();

        Self {
            context,
            builder,
            module,
            target,
            machine,
            sym_table: RefCell::new(HashMap::new()),
        }
    }

    // This method will just print the contents of the module,
    // which will show us what the IR we just generated looks like
    // within our context.
    pub fn dump_module(&self) {
        self.module.print_to_stderr();
    }

    // Small helper method to remove the top level anonymous expression,
    // needed for REPL so that we don't define top level twice, just delete
    // it and then define it again.
    pub fn delete_top_level_expr(&self) {
        unsafe { self.module.get_function("__anonymous_expr").map(|f| f.delete()) };
    }

    // Optimization passes
    pub fn run_passes(&self) {
        let pass_options = PassBuilderOptions::create();
        pass_options.set_verify_each(true);
        pass_options.set_debug_logging(true);
        pass_options.set_loop_interleaving(true);
        pass_options.set_loop_vectorization(true);
        pass_options.set_loop_slp_vectorization(true);
        pass_options.set_loop_unrolling(true);
        pass_options.set_forget_all_scev_in_loop_unroll(true);
        pass_options.set_licm_mssa_opt_cap(1);
        pass_options.set_licm_mssa_no_acc_for_promotion_cap(10);
        pass_options.set_call_graph_profile(true);
        pass_options.set_merge_functions(true);

        self.module.run_passes(
            "instcombine,reassociate,gvn,simplifycfg", 
            &self.machine, 
            pass_options
        ).unwrap();
    }

    // JIT evalution, creates an ExecutionEngine object, JIT compiles the function,
    // then attempts to call the function, will return the resulting floating point val.
    pub unsafe fn jit_eval(&self) -> Result<f64, BackendError> {

        let exec_engine = self.module.create_jit_execution_engine(OptimizationLevel::None)
            .expect("FATAL: Failed to create JIT execution engine!");

        let jitted_fn: JitFunction<'ctx, TopLevelSignature> = exec_engine.get_function("__anonymous_expr")
            .expect("FATAL: symbol '__anonymous_expr' not present in module!");

        let res = jitted_fn.call();

        exec_engine.remove_module(&self.module).unwrap();

        Ok(res)
    }
}

// There are three lifetimes at play when working with references from the
// source code (AST), and the LLVM objects (the context object and IR it produces)
// The IR is bound by context, as seen below in the where portion of this trait.
// IR and context are related in that IR can live no longer than the context that
// creates it.
//
// The method codegen will try to generate IR, given a context to work with.
// This method is called recursively across the AST, from node to node.
// This method, if successful, will return AnyValueEnum, which enables
// this method to return a different LLVM value depending on node
pub trait LLVMCodeGen<'ctx, 'ir, 'src>
where
    'ctx: 'ir // Context and IR have a unique relationship, bounded.
{
    fn codegen(
        &self, context: &LLVMContext<'ctx>
    ) -> IRGenResult<'ir, 'src>;
}

impl<'ctx, 'ir, 'src> LLVMCodeGen<'ctx, 'ir, 'src> for ASTExpr<'src>
where
    'ctx: 'ir 
{
    fn codegen(
        &self, context: &LLVMContext<'ctx>
    ) -> IRGenResult<'ir, 'src> {
        use ASTExpr::*;

        // To generate code for any expression, we must handle the number, variable, call, and
        // binary expression cases.
        match self {
            // Number expression case, just grab a number constant from context space
            NumberExpr(num) => {
                let float_type = context.context.f64_type();
                Ok(float_type.const_float(*num).as_any_value_enum())
            },

            // To handle variable case, make sure the variable exists in symbol table,
            // if it doesn't return error, otherwise, fetch the LLVM Value for that variable
            VariableExpr(varname) => {
                if let Some(llvm_val) = context.sym_table.borrow().get(*varname) {
                    Ok(*llvm_val)
                } else {
                    Err(BackendError::UnknownVariable(varname))
                }
            },

            // Generate the left and right code first, then build the correct
            // instruction depending on the operator.
            BinaryExpr { op, left, right } => {
                let left_genval = left
                    .codegen(context)
                    .map(AnyValueEnum::into_float_value)?;

                let right_genval = right
                    .codegen(context)
                    .map(AnyValueEnum::into_float_value)?;
        
                let float_res = match *op {
                    Ops::Plus => context.builder.build_float_add(
                        left_genval, right_genval, &"addtmp"
                    ),
        
                    Ops::Minus => context.builder.build_float_sub(
                        left_genval, right_genval, &"subtmp"
                    ),
        
                    Ops::Mult => context.builder.build_float_mul(
                        left_genval, right_genval, &"multmp"
                    ),
        
                    Ops::Div => context.builder.build_float_div(
                        left_genval, right_genval, &"divtmp"
                    ),
                };
        
                Ok(float_res.expect("Irrecoverable: LLVM failed to generate insn").as_any_value_enum())
            },

            // This one is the most complex expression to handle...
            CallExpr { ref callee, args } => {

                // First, see if the function is defined in LLVM module, if not, we have 
                // an undefined function trying to be called
                let function = context
                    .module
                    .get_function(callee)
                    .ok_or(BackendError::UndefinedFunction(callee))?;
        
                let param_cnt = function.count_params();
        
                if param_cnt != args.len() as u32 { // verify parameter counts
                    return Err(BackendError::IncorrectNumberOfArgs {
                        func_name: callee,
                        param_cnt,
                    });
                }
                
                // Generate code for the arguments passed, call site expressions,
                // Any of the arguments could also produce a backend error, so propogate up
                let llvm_val_args = args.iter()
                    .map(|arg| arg.codegen(context))
                    .collect::<Result<Vec<_>, BackendError>>()?;
        
                let llvm_val_args: Vec<BasicMetadataValueEnum> = llvm_val_args
                    .into_iter()
                    .map(|val| BasicMetadataValueEnum::FloatValue(val.into_float_value()))
                    .collect();
                
                // Building a call requires arguments be of type BasicMetadataValueEnum,
                // as a slice of them, had to convert, but does produce LLVM call instruction.
                let call = context
                    .builder
                    .build_call(function, llvm_val_args.as_slice(), &"calltmp")
                    .expect("Irrecoverable: LLVM failed to build call expression");
        
                Ok(call.as_any_value_enum())
            },
        }
    }
}

// At prototype node, we need to establish arguments (all floats of course)
// Add the function to module with type as fn(), fn (float) fn(float, float), etc...
impl<'ctx, 'ir, 'src> LLVMCodeGen<'ctx, 'ir, 'src> for Prototype<'src>
where
    'ctx: 'ir 
{
    fn codegen(&self, context: &LLVMContext<'ctx>) -> IRGenResult<'ir, 'src> {
        let param_types =
            vec![BasicMetadataTypeEnum::FloatType(context.context.f64_type()); self.args.len()];

        let fn_type = context
            .context
            .f64_type()
            .fn_type(param_types.as_slice(), false);

        let fn_val = context
            .module
            .add_function(&self.name, fn_type, Some(Linkage::External));

        // Set the names of params so the body expression can have resolution
        // to the names of the parameters of function!
        for (idx, param) in fn_val.get_params().iter().enumerate() {
            param.set_name(&self.args[idx])
        }

        Ok(fn_val.as_any_value_enum())
    }
}

impl<'ctx, 'ir, 'src> LLVMCodeGen<'ctx, 'ir, 'src> for Function<'src>
where
    'ctx: 'ir  
{
    fn codegen(&self, context: &LLVMContext<'ctx>) -> IRGenResult<'ir, 'src> {

        // See if function has been defined, if not, generate prototype
        // to get the LLVM function value.
        let fn_val = match context.module.get_function(&self.proto.name) {
            Some(fn_val) => fn_val,
            None => self.proto.codegen(context)?.into_function_value(),
        };

        // To make sure we aren't defining functions twice, I just check if it
        // has no entry basic block, if it does, then propogate error.
        if fn_val.get_first_basic_block().is_some() {
            return Err(BackendError::MultipleFunctionDefs(self.proto.name));
        }

        // This sets our cursor for creating instructions to the basic block
        // for this function
        let bb_entry = context.context.append_basic_block(fn_val, "entry");
        context.builder.position_at_end(bb_entry);

        // Update the symbol table with the args names and references 
        // to their LLVM values.
        context.sym_table.borrow_mut().clear();
        for arg in fn_val.get_params() {

            // TODO: Change the named value key to a non-owned CStr reference
            // so I am not copying and cloning to Rust Strings
            let owned_str = arg.into_float_value()
                .get_name().to_str().unwrap().to_string();

            context
                .sym_table
                .borrow_mut()
                .insert(owned_str, arg.as_any_value_enum());
        }

        // Generate code for the body of the function as an ASTExpr node
        let ir_body = self.body.codegen(context)?;

        // We need to add a return at the end so we return from functions we call
        context
            .builder
            .build_return(Some(&ir_body.into_float_value() as &dyn BasicValue))
            .expect("FATAL: LLVM failed to build a return!");

        if !fn_val.verify(true) {
            return Err(BackendError::FailedToVerifyFunc(self.proto.name));
        }

        Ok(fn_val.as_any_value_enum())
    }
}
