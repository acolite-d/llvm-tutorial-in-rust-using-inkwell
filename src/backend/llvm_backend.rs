use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::sync::Mutex;

use inkwell::builder::{self, Builder};
use inkwell::context::Context;
use inkwell::execution_engine::ExecutionEngine;
use inkwell::llvm_sys::{LLVMModule, LLVMValue};
use inkwell::module::{Linkage, Module};
use inkwell::targets::{Target, TargetMachine, RelocMode, CodeModel};
use inkwell::types::BasicMetadataTypeEnum;
use inkwell::values::{AnyValue, AnyValueEnum, BasicMetadataValueEnum, BasicValue, FloatValue, FunctionValue};
use inkwell::OptimizationLevel;
use inkwell::passes::{PassBuilderOptions, PassManager, PassManagerSubType};
use thiserror::Error;

use crate::frontend::ast::{
    BinaryExpr, CallExpr, Function, NumberExpr, Prototype, VariableExpr, AST,
};
use crate::frontend::lexer::Ops;

type IRGenResult<'ir> = Result<AnyValueEnum<'ir>, BackendError>;

#[derive(Error, PartialEq, Debug)]
pub enum BackendError {
    #[error("Unknown variable name {0}")]
    UnknownVariable(String),

    #[error("Undefined function {0}")]
    UndefinedFunction(String),

    #[error("Function {0} defined twice")]
    MultipleFunctionDefs(String),

    #[error("Incorrect number of arguments passed to {func_name}, expected {param_cnt}")]
    IncorrectNumberOfArgs { func_name: String, param_cnt: u32 },

    #[error("LLVM failed to verify function {0}")]
    FailedToVerifyFunc(String),
}

#[derive(Debug)]
pub struct LLVMContext<'ctx> {
    context: &'ctx Context,
    builder: Builder<'ctx>,
    module: Module<'ctx>,
    machine: TargetMachine,
    // fn_pass_manager: PassManager<FunctionValue<'ctx>>,
    // exec_engine: ExecutionEngine<'ctx>,
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

        // let exec_engine = module.create_execution_engine()
        //     .expect("FATAL: Failed to create LLVM execution engine for JIT!");

        // module.set_data_layout(
        //     &exec_engine.get_target_data().get_data_layout()
        // );

        Self {
            context,
            builder,
            module,
            machine,
            // exec_engine,
            sym_table: RefCell::new(HashMap::new()),
        }
    }

    pub fn dump_module(&self) {
        self.module.print_to_stderr();
    }

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
}

pub trait LLVMCodeGen {
    fn codegen<'ctx: 'ir, 'ir>(&self, context: &LLVMContext<'ctx>) -> IRGenResult<'ir>;
}

impl LLVMCodeGen for NumberExpr {
    fn codegen<'ctx: 'ir, 'ir>(&self, context: &LLVMContext<'ctx>) -> IRGenResult<'ir> {
        let float_type = context.context.f64_type();
        Ok(float_type.const_float(self.0).as_any_value_enum())
    }
}

impl LLVMCodeGen for VariableExpr {
    fn codegen<'ctx: 'ir, 'ir>(&self, context: &LLVMContext<'ctx>) -> IRGenResult<'ir> {
        if let Some(llvm_val) = context.sym_table.borrow().get(&self.name) {
            Ok(*llvm_val)
        } else {
            Err(BackendError::UnknownVariable(self.name.clone()))
        }
    }
}

impl LLVMCodeGen for BinaryExpr {
    fn codegen<'ctx: 'ir, 'ir>(&self, context: &LLVMContext<'ctx>) -> IRGenResult<'ir> {
        let left = self
            .left
            .codegen(context)
            .map(AnyValueEnum::into_float_value)?;
        let right = self
            .right
            .codegen(context)
            .map(AnyValueEnum::into_float_value)?;

        let float_res = match self.op {
            Ops::Plus => context.builder.build_float_add(left, right, &"addtmp"),

            Ops::Minus => context.builder.build_float_sub(left, right, &"subtmp"),

            Ops::Mult => context.builder.build_float_mul(left, right, &"multmp"),

            Ops::Div => context.builder.build_float_div(left, right, &"divtmp"),

            _ => panic!(),
        };

        Ok(float_res.expect("Irrecoverable: LLVM failed to generate insn").as_any_value_enum())
    }
}

impl LLVMCodeGen for CallExpr {
    fn codegen<'ctx: 'ir, 'ir>(&self, context: &LLVMContext<'ctx>) -> IRGenResult<'ir> {
        let function = context
            .module
            .get_function(&self.name)
            .ok_or(BackendError::UndefinedFunction(self.name.clone()))?;

        let param_cnt = function.count_params();

        if param_cnt != self.args.len() as u32 {
            return Err(BackendError::IncorrectNumberOfArgs {
                func_name: self.name.clone(),
                param_cnt,
            });
        }

        let llvm_val_args = self
            .args
            .iter()
            .map(|arg| arg.codegen(context))
            .collect::<Result<Vec<_>, BackendError>>()?;

        let llvm_val_args: Vec<BasicMetadataValueEnum> = llvm_val_args
            .into_iter()
            .map(|val| BasicMetadataValueEnum::FloatValue(val.into_float_value()))
            .collect();

        let call = context
            .builder
            .build_call(function, llvm_val_args.as_slice(), &"calltmp")
            .expect("Irrecoverable: LLVM failed to build call expression");

        Ok(call.as_any_value_enum())
    }
}

impl LLVMCodeGen for Prototype {
    fn codegen<'ctx: 'ir, 'ir>(&self, context: &LLVMContext<'ctx>) -> IRGenResult<'ir> {
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

impl LLVMCodeGen for Function {
    fn codegen<'ctx: 'ir, 'ir>(&self, context: &LLVMContext<'ctx>) -> IRGenResult<'ir> {

        let fn_val = match context.module.get_function(&self.proto.name) {
            Some(fn_val) => fn_val,
            None => self.proto.codegen(context)?.into_function_value(),
        };

        if fn_val.get_first_basic_block().is_some() {
            return Err(BackendError::MultipleFunctionDefs(self.proto.name.clone()));
        }

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

        if let Ok(body) = self.body.codegen(context) {
            context
                .builder
                .build_return(Some(&body.into_float_value() as &dyn BasicValue));

            if !fn_val.verify(true) {
                return Err(BackendError::FailedToVerifyFunc(self.proto.name.clone()))
            }

            // context.fn_pass_manager.run_on(&fn_val);
        }

        Ok(fn_val.as_any_value_enum())
    }
}
