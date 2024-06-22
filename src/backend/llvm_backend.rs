use std::cell::RefCell;
use std::collections::HashMap;

use inkwell::basic_block::BasicBlock;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::execution_engine::JitFunction;
use inkwell::module::{Linkage, Module};
use inkwell::passes::PassBuilderOptions;
use inkwell::targets::{CodeModel, RelocMode, Target, TargetMachine};
use inkwell::types::BasicMetadataTypeEnum;
use inkwell::values::{AnyValue, AnyValueEnum, BasicMetadataValueEnum, BasicValue};
use inkwell::FloatPredicate;
use inkwell::OptimizationLevel;
use thiserror::Error;

use crate::cli::OptLevel;
use crate::frontend::{
    lexer::Ops,
    ast::{ASTExpr, Function, Prototype}
};

type IRGenResult<'ir, 'src> = Result<AnyValueEnum<'ir>, BackendError<'src>>;
type TopLevelSignature = unsafe extern "C" fn() -> f64;

macro_rules! to_llvm_float {
    ($context:expr, $int_val:expr) => {
        $context
            .builder
            .build_unsigned_int_to_float($int_val, $context.context.f64_type(), &"booltmp")
            .expect("FATAL: LLVM failed to convert int to float")
    };
}

// Possible errors that might result when generating/JIT'ing
// LLVM IR
#[derive(Error, PartialEq, Debug)]
pub enum BackendError<'src> {
    #[error("Unknown variable name {0}")]
    UnknownVariable(&'src str),

    #[error("Undefined function {0}")]
    UndefinedFunction(&'src str),

    #[error("Function {0} defined twice")]
    MultipleFunctionDefs(String),

    #[error("Incorrect number of arguments passed to {func_name}, expected {param_cnt}")]
    IncorrectNumberOfArgs {
        func_name: &'src str,
        param_cnt: u32,
    },

    #[error("LLVM failed to verify function {0}")]
    FailedToVerifyFunc(String),

    #[error("Undefined operator used: {0:?}")]
    UndefinedOperator(Ops),

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
    pub fn new(context: &'ctx Context, opt_level: OptLevel) -> Self {
        let builder = context.create_builder();
        let module = context.create_module("kaleidrs_module");

        let triple = TargetMachine::get_default_triple();
        let target = Target::from_triple(&triple).unwrap();

        let machine = target
            .create_target_machine(
                &triple,
                "generic",
                "",
                opt_level.into(),
                RelocMode::Default,
                CodeModel::Default,
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
        unsafe {
            self.module
                .get_function("__anonymous_expr")
                .map(|f| f.delete())
        };
    }

    // Optimization passes
    pub fn run_passes(&self, passes: &str) {
        let pass_options = PassBuilderOptions::create();

        // Default passes
        pass_options.set_verify_each(true);
        pass_options.set_debug_logging(false);
        pass_options.set_loop_interleaving(true);
        pass_options.set_loop_vectorization(true);
        pass_options.set_loop_slp_vectorization(true);
        pass_options.set_loop_unrolling(true);
        pass_options.set_forget_all_scev_in_loop_unroll(true);
        pass_options.set_licm_mssa_opt_cap(1);
        pass_options.set_licm_mssa_no_acc_for_promotion_cap(10);
        pass_options.set_call_graph_profile(true);
        pass_options.set_merge_functions(true);

        self.module
            .run_passes(passes, &self.machine, pass_options)
            .unwrap();
    }

    // JIT evalution, creates an ExecutionEngine object, JIT compiles the function,
    // then attempts to call the function, will return the resulting floating point val.
    pub unsafe fn jit_eval(&self) -> Result<f64, BackendError> {
        let exec_engine = self
            .module
            .create_jit_execution_engine(OptimizationLevel::None)
            .expect("FATAL: Failed to create JIT execution engine!");

        let jitted_fn: JitFunction<'ctx, TopLevelSignature> = exec_engine
            .get_function("__anonymous_expr")
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
    'ctx: 'ir,
{
    fn codegen(&self, context: &LLVMContext<'ctx>) -> IRGenResult<'ir, 'src>;
}

impl<'ctx, 'ir, 'src> LLVMCodeGen<'ctx, 'ir, 'src> for ASTExpr<'src>
where
    'ctx: 'ir,
{
    fn codegen(&self, context: &LLVMContext<'ctx>) -> IRGenResult<'ir, 'src> {
        use ASTExpr::*;

        // To generate code for any expression, we must handle the number, variable, call, and
        // binary expression cases.
        match self {
            // Number expression case, just grab a number constant from context space
            NumberExpr(num) => {
                let float_type = context.context.f64_type();
                Ok(float_type.const_float(*num).as_any_value_enum())
            }

            // To handle variable case, make sure the variable exists in symbol table,
            // if it doesn't return error, otherwise, fetch the LLVM Value for that variable
            VariableExpr(varname) => {
                if let Some(llvm_val) = context.sym_table.borrow().get(*varname) {
                    Ok(*llvm_val)
                } else {
                    Err(BackendError::UnknownVariable(varname))
                }
            }

            // Unary Expressions, all fall into the category of overloaded operators
            UnaryExpr { op, operand } => {
                let fn_name = format!("unary{}", op.as_str());
                
                if let Some(unary_overload_fn) = context.module.get_function(&fn_name) {
                    let operand_genval = operand.codegen(context).map(|anyval| {
                        BasicMetadataValueEnum::FloatValue(anyval.into_float_value())
                    })?;

                    let unary_op_call = context
                        .builder
                        .build_call(unary_overload_fn, &[operand_genval], "unarytmp")
                        .expect("FATAL: LLVM failed to build call!");

                    Ok(unary_op_call.as_any_value_enum())
                } else {
                    Err(BackendError::UndefinedOperator(*op))
                }
            }

            // Generate the left and right code first, then build the correct
            // instruction depending on the operator.
            BinaryExpr { op, left, right } => {
                let left_genval = left.codegen(context).map(AnyValueEnum::into_float_value)?;

                let right_genval = right.codegen(context).map(AnyValueEnum::into_float_value)?;

                match *op {
                    Ops::Plus => {
                        let add = context
                            .builder
                            .build_float_add(left_genval, right_genval, &"addtmp")
                            .unwrap();

                        Ok(add.as_any_value_enum())
                    }

                    Ops::Minus => {
                        let sub = context
                            .builder
                            .build_float_sub(left_genval, right_genval, &"subtmp")
                            .unwrap();

                        Ok(sub.as_any_value_enum())
                    }

                    Ops::Mult => {
                        let mult = context
                            .builder
                            .build_float_mul(left_genval, right_genval, &"multmp")
                            .unwrap();

                        Ok(mult.as_any_value_enum())
                    }

                    Ops::Div => {
                        let div = context
                            .builder
                            .build_float_div(left_genval, right_genval, &"divtmp")
                            .unwrap();

                        Ok(div.as_any_value_enum())
                    }

                    // For the comparison operators, map() a conversion back to float, Kaleidoscope only works with floating point nums!
                    Ops::Eq => {
                        let cmp = context
                            .builder
                            .build_float_compare(
                                FloatPredicate::OEQ,
                                left_genval,
                                right_genval,
                                &"eqtmp",
                            )
                            .map(|int_val| to_llvm_float!(context, int_val))
                            .unwrap();

                        Ok(cmp.as_any_value_enum())
                    }

                    Ops::Neq => {
                        let cmp = context
                            .builder
                            .build_float_compare(
                                FloatPredicate::ONE,
                                left_genval,
                                right_genval,
                                &"neqtmp",
                            )
                            .map(|int_val| to_llvm_float!(context, int_val))
                            .unwrap();

                        Ok(cmp.as_any_value_enum())
                    }

                    Ops::Gt => {
                        let cmp = context
                            .builder
                            .build_float_compare(
                                FloatPredicate::OGT,
                                left_genval,
                                right_genval,
                                &"gttmp",
                            )
                            .map(|int_val| to_llvm_float!(context, int_val))
                            .unwrap();

                        Ok(cmp.as_any_value_enum())
                    }

                    Ops::Lt => {
                        let cmp = context
                            .builder
                            .build_float_compare(
                                FloatPredicate::OLT,
                                left_genval,
                                right_genval,
                                &"lttmp",
                            )
                            .map(|int_val| to_llvm_float!(context, int_val))
                            .unwrap();

                        Ok(cmp.as_any_value_enum())
                    }

                    overloaded_op => {
                        // First, we have to check if the operator has been defined, if not, then
                        // we return error, because we cannot apply an operator that has not been defined
                        // yet!
                        let fn_name = format!("binary{}", overloaded_op.as_str());

                        if let Some(binary_overload_fn) = context.module.get_function(&fn_name) {
                            let args = [left_genval, right_genval]
                                .into_iter()
                                .map(|anyval| BasicMetadataValueEnum::FloatValue(anyval))
                                .collect::<Vec<_>>();

                            let overload_call = context
                                .builder
                                .build_call(binary_overload_fn, args.as_slice(), &"calltmp")
                                .expect("FATAL: LLVM failed to build call!");

                            Ok(overload_call.as_any_value_enum())
                        } else {
                            Err(BackendError::UndefinedOperator(overloaded_op))
                        }
                    }
                }
            }

            // This one is the most complex expression to handle...
            CallExpr { ref callee, args } => {
                // First, see if the function is defined in LLVM module, if not, we have
                // an undefined function trying to be called
                let function = context
                    .module
                    .get_function(callee)
                    .ok_or(BackendError::UndefinedFunction(callee))?;

                let param_cnt = function.count_params();

                if param_cnt != args.len() as u32 {
                    // verify parameter counts
                    return Err(BackendError::IncorrectNumberOfArgs {
                        func_name: callee,
                        param_cnt,
                    });
                }

                // Generate code for the arguments passed, call site expressions,
                // Any of the arguments could also produce a backend error, so propogate up
                let llvm_val_args = args
                    .iter()
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
            }

            IfExpr {
                cond,
                then_branch,
                else_branch,
            } => {
                let cond_codegen = cond.codegen(context)?;

                let one = context.context.f64_type().const_float(1.0);

                // Compute the truth of the condition by comparing value of expression to one
                let cond_bool = context
                    .builder
                    .build_float_compare(
                        FloatPredicate::OEQ,
                        cond_codegen.into_float_value(),
                        one,
                        &"iftemp",
                    )
                    .expect("FATAL: LLVM failed to build float compare!");

                let function = context
                    .builder
                    .get_insert_block()
                    .unwrap()
                    .get_parent()
                    .unwrap();

                // Basic blocks to be added for this branch
                // true path - 1, not true - 2, merge - 3
                let bbs = [&"then", &"else", &"ifcont"]
                    .into_iter()
                    .map(|bb_name| context.context.append_basic_block(function, bb_name))
                    .collect::<Vec<BasicBlock<'ctx>>>();

                let _llvm_br_insn = context
                    .builder
                    .build_conditional_branch(cond_bool, bbs[0], bbs[1])
                    .expect("FATAL: LLVM failed to build br instruction!");

                // IMPORTANT: Be sure you set the builder cursor to the appropriate block
                // before calling codegen() methods on then and else expressions, otherwise
                // we would generate code in wrong basic block and mess everything up.
                context.builder.position_at_end(bbs[0]);
                let then_v = then_branch.codegen(context)?;

                // Don't forget to branch back to merge basic block!!!
                context
                    .builder
                    .build_unconditional_branch(bbs[2])
                    .expect("FATAL: LLVM failed to build branch!");

                context.builder.position_at_end(bbs[1]);
                let else_v = else_branch.codegen(context)?;
                context
                    .builder
                    .build_unconditional_branch(bbs[2])
                    .expect("FATAL: LLVM failed to build branch!");

                context.builder.position_at_end(bbs[2]);
                let phi_node = context
                    .builder
                    .build_phi(context.context.f64_type(), &"iftmp")
                    .expect("LLVM failed to create PHI!");

                phi_node.add_incoming(&[
                    (&then_v.into_float_value() as &dyn BasicValue<'ctx>, bbs[0]),
                    (&else_v.into_float_value() as &dyn BasicValue<'ctx>, bbs[1]),
                ]);

                Ok(phi_node.as_any_value_enum())
            }

            ForLoopExpr {
                varname,
                start,
                end,
                step,
                body,
            } => {
                let start_genval = start.codegen(context)?;

                let preheader_bb = context.builder.get_insert_block().unwrap();
                let function = preheader_bb.get_parent().unwrap();

                let loop_bb = context.context.append_basic_block(function, &"loop");

                context.builder.position_at_end(preheader_bb);
                context
                    .builder
                    .build_unconditional_branch(loop_bb)
                    .expect("FATAL: LLVM failed to build branch!");

                context.builder.position_at_end(loop_bb);

                let loop_phi_var = context
                    .builder
                    .build_phi(context.context.f64_type(), varname)
                    .expect("FATAL: LLVM failed to create PHI!");

                loop_phi_var.add_incoming(&[(
                    &start_genval.into_float_value() as &dyn BasicValue<'ctx>,
                    preheader_bb,
                )]);

                let shadowed_var = context.sym_table.borrow().get(*varname).copied();
                context
                    .sym_table
                    .borrow_mut()
                    .insert(varname.to_string(), loop_phi_var.as_any_value_enum());

                body.codegen(context)?;

                let step_genval = {
                    if let Some(step_expr) = step {
                        step_expr.codegen(context)?
                    } else {
                        context
                            .context
                            .f64_type()
                            .const_float(1.0)
                            .as_any_value_enum()
                    }
                };

                let next_var = context
                    .builder
                    .build_float_add(
                        loop_phi_var.as_basic_value().into_float_value(),
                        step_genval.into_float_value(),
                        &"nextvar",
                    )
                    .unwrap();

                let end_cond = end.codegen(context)?;

                let cmp_val = context
                    .builder
                    .build_float_compare(
                        FloatPredicate::ONE,
                        end_cond.into_float_value(),
                        context.context.f64_type().const_float(0.0),
                        &"loopcond",
                    )
                    .expect("FATAL: LLVM failed to build comparison instruction!");

                let afterloop_bb = context.context.append_basic_block(function, "afterloop");

                context.builder.position_at_end(loop_bb);

                context
                    .builder
                    .build_conditional_branch(cmp_val, loop_bb, afterloop_bb)
                    .unwrap();

                context.builder.position_at_end(afterloop_bb);

                loop_phi_var.add_incoming(&[(&next_var as &dyn BasicValue<'ctx>, afterloop_bb)]);

                if let Some(variable) = shadowed_var {
                    context
                        .sym_table
                        .borrow_mut()
                        .insert(varname.to_string(), variable);
                } else {
                    context.sym_table.borrow_mut().remove(*varname);
                }

                Ok(context
                    .context
                    .f64_type()
                    .const_float(0.0)
                    .as_any_value_enum())
            }
        }
    }
}

// At prototype node, we need to establish arguments (all floats of course)
// Add the function to module with type as fn(), fn (float) fn(float, float), etc...
impl<'ctx, 'ir, 'src> LLVMCodeGen<'ctx, 'ir, 'src> for Prototype<'src>
where
    'ctx: 'ir,
{
    fn codegen(&self, context: &LLVMContext<'ctx>) -> IRGenResult<'ir, 'src> {
        use Prototype::*;

        let fn_name = self.get_name();

        let param_types 
            = vec![BasicMetadataTypeEnum::FloatType(context.context.f64_type()); self.get_num_params()];

        let fn_type = context
            .context
            .f64_type()
            .fn_type(param_types.as_slice(), false);

        let fn_val = context
            .module
            .add_function(&fn_name, fn_type, Some(Linkage::External));

        match self {
            FunctionProto { args, .. } => {
                // Set the names of params so the body expression can have resolution
                // to the names of the parameters of function!
                for (idx, param) in fn_val.get_params().iter().enumerate() {
                    param.set_name(&args[idx])
                }
            }

            OverloadedUnaryOpProto { arg, .. } => {
                fn_val.get_params()[0].set_name(&arg);
            }

            OverloadedBinaryOpProto { args: (lhs, rhs), precedence, .. } => {
                let params = fn_val.get_params();
                params[0].set_name(&lhs);
                params[1].set_name(&rhs);
            }
        }

        Ok(fn_val.as_any_value_enum())
    }
}

impl<'ctx, 'ir, 'src> LLVMCodeGen<'ctx, 'ir, 'src> for Function<'src>
where
    'ctx: 'ir,
{
    fn codegen(&self, context: &LLVMContext<'ctx>) -> IRGenResult<'ir, 'src> {
        // See if function has been defined, if not, generate prototype
        // to get the LLVM function value.
        let fn_val = match context.module.get_function(&self.proto.get_name()) {
            Some(fn_val) => fn_val,
            None => self.proto.codegen(context)?.into_function_value(),
        };

        // To make sure we aren't defining functions twice, I just check if it
        // has no entry basic block, if it does, then propogate error.
        if fn_val.get_first_basic_block().is_some() {
            return Err(BackendError::MultipleFunctionDefs(self.proto.get_name()));
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
            let owned_str = arg
                .into_float_value()
                .get_name()
                .to_str()
                .unwrap()
                .to_string();

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
            return Err(BackendError::FailedToVerifyFunc(self.proto.get_name()));
        }

        Ok(fn_val.as_any_value_enum())
    }
}
