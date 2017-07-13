use llvm_sys::analysis::{LLVMVerifierFailureAction, LLVMVerifyFunction, LLVMViewFunctionCFG, LLVMViewFunctionCFGOnly};
use llvm_sys::core::{LLVMAddIncoming, LLVMCountParams, LLVMGetBasicBlocks, LLVMGetElementType, LLVMGetFirstBasicBlock, LLVMGetFirstParam, LLVMGetLastBasicBlock, LLVMGetNextParam, LLVMGetParam, LLVMGetReturnType, LLVMGetValueName, LLVMIsAConstantArray, LLVMIsAConstantDataArray, LLVMIsAFunction, LLVMIsConstant, LLVMIsNull, LLVMIsUndef, LLVMPrintTypeToString, LLVMPrintValueToString, LLVMSetGlobalConstant, LLVMSetValueName, LLVMTypeOf, LLVMGetTypeKind};
use llvm_sys::LLVMTypeKind;
use llvm_sys::prelude::LLVMValueRef;

use std::ffi::{CString, CStr};
use std::fmt;
use std::mem::transmute;

use basic_block::BasicBlock;
use types::{AnyTypeEnum, ArrayType, BasicTypeEnum, PointerType, FloatType, IntType, StructType};

mod private {
    // This is an ugly privacy hack so that Type can stay private to this module
    // and so that super traits using this trait will be not be implementable
    // outside this library
    use llvm_sys::prelude::LLVMValueRef;

    pub trait AsValueRef {
        fn as_value_ref(&self) -> LLVMValueRef;
    }
}

pub(crate) use self::private::AsValueRef;

pub struct Value {
    value: LLVMValueRef,
}

impl Value {
    pub(crate) fn new(value: LLVMValueRef) -> Value {
        assert!(!value.is_null());

        Value {
            value: value
        }
    }

    fn set_global_constant(&self, num: i32) { // REVIEW: Need better name for this arg
        unsafe {
            LLVMSetGlobalConstant(self.value, num)
        }
    }

    fn set_name(&self, name: &str) {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        unsafe {
            LLVMSetValueName(self.value, c_string.as_ptr());
        }
    }

    fn get_name(&self) -> &CStr {
        unsafe {
            CStr::from_ptr(LLVMGetValueName(self.value))
        }
    }

    // REVIEW: Untested
    // REVIEW: Is incoming_values really ArrayValue? Or an &[AnyValue]?
    fn add_incoming(&self, incoming_values: &AnyValue, incoming_basic_block: &BasicBlock, count: u32) {
        let value = &mut [incoming_values.as_value_ref()];
        let basic_block = &mut [incoming_basic_block.basic_block];

        unsafe {
            LLVMAddIncoming(self.value, value.as_mut_ptr(), basic_block.as_mut_ptr(), count);
        }
    }

    // REVIEW: Untested
    fn is_undef(&self) -> bool {
        unsafe {
            LLVMIsUndef(self.value) == 1
        }
    }

    fn get_type(&self) -> AnyTypeEnum {
        let type_ = unsafe {
            LLVMTypeOf(self.value)
        };

        AnyTypeEnum::new(type_)
    }

    // REVIEW: Remove?
    // fn get_type_kind(&self) -> LLVMTypeKind {
    //     (*self.get_type()).as_llvm_type_ref().get_kind()
    // }

    // fn is_pointer(&self) -> bool {
    //     match self.get_type_kind() {
    //         LLVMTypeKind::LLVMPointerTypeKind => true,
    //         _ => false,
    //     }
    // }

    // fn is_int(&self) -> bool {
    //     match self.get_type_kind() {
    //         LLVMTypeKind::LLVMIntegerTypeKind => true,
    //         _ => false,
    //     }
    // }

    // fn is_f32(&self) -> bool {
    //     match self.get_type_kind() {
    //         LLVMTypeKind::LLVMFloatTypeKind => true,
    //         _ => false,
    //     }
    // }

    // fn is_f64(&self) -> bool {
    //     match self.get_type_kind() {
    //         LLVMTypeKind::LLVMDoubleTypeKind => true,
    //         _ => false,
    //     }
    // }

    // fn is_f128(&self) -> bool {
    //     match self.get_type_kind() {
    //         LLVMTypeKind::LLVMFP128TypeKind => true,
    //         _ => false,
    //     }
    // }

    // fn is_float(&self) -> bool {
    //     match self.get_type_kind() {
    //         LLVMTypeKind::LLVMHalfTypeKind |
    //         LLVMTypeKind::LLVMFloatTypeKind |
    //         LLVMTypeKind::LLVMDoubleTypeKind |
    //         LLVMTypeKind::LLVMX86_FP80TypeKind |
    //         LLVMTypeKind::LLVMFP128TypeKind |
    //         LLVMTypeKind::LLVMPPC_FP128TypeKind => true,
    //         _ => false,
    //     }
    // }

    // fn is_struct(&self) -> bool {
    //     match self.get_type_kind() {
    //         LLVMTypeKind::LLVMStructTypeKind => true,
    //         _ => false,
    //     }
    // }

    // fn is_array(&self) -> bool {
    //     match self.get_type_kind() {
    //         LLVMTypeKind::LLVMArrayTypeKind => true,
    //         _ => false,
    //     }
    // }

    // fn is_void(&self) -> bool {
    //     match self.get_type_kind() {
    //         LLVMTypeKind::LLVMVoidTypeKind => true,
    //         _ => false,
    //     }
    // }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let llvm_value = unsafe {
            CStr::from_ptr(LLVMPrintValueToString(self.value))
        };
        let llvm_type = unsafe {
            CStr::from_ptr(LLVMPrintTypeToString(LLVMTypeOf(self.value)))
        };
        let name = unsafe {
            CStr::from_ptr(LLVMGetValueName(self.value))
        };
        let is_const = unsafe {
            LLVMIsConstant(self.value) == 1
        };
        let is_null = unsafe {
            LLVMIsNull(self.value) == 1
        };

        write!(f, "Value {{\n    name: {:?}\n    address: {:?}\n    is_const: {:?}\n    is_null: {:?}\n    llvm_value: {:?}\n    llvm_type: {:?}\n}}", name, self.value, is_const, is_null, llvm_value, llvm_type)
    }
}

pub struct FunctionValue {
    fn_value: Value,
}

impl FunctionValue {
    pub(crate) fn new(value: LLVMValueRef) -> FunctionValue {
        assert!(!value.is_null());

        unsafe {
            assert!(!LLVMIsAFunction(value).is_null())
        }

        FunctionValue {
            fn_value: Value::new(value)
        }
    }

    // TODO: Maybe support LLVMAbortProcessAction?
    pub fn verify(&self, print: bool) {
        let action = if print {
            LLVMVerifierFailureAction::LLVMPrintMessageAction
        } else {
            LLVMVerifierFailureAction::LLVMReturnStatusAction
        };

        let code = unsafe {
            LLVMVerifyFunction(self.fn_value.value, action)
        };

        if code == 1 {
            panic!("LLVMGenError")
        }
    }

    pub fn get_first_param(&self) -> Option<BasicValueEnum> {
        let param = unsafe {
            LLVMGetFirstParam(self.fn_value.value)
        };

        if param.is_null() {
            return None;
        }

        Some(BasicValueEnum::new(param))
    }

    pub fn get_first_basic_block(&self) -> Option<BasicBlock> {
        let bb = unsafe {
            LLVMGetFirstBasicBlock(self.fn_value.value)
        };

        if bb.is_null() {
            return None;
        }

        Some(BasicBlock::new(bb))
    }

    pub fn get_nth_param(&self, nth: u32) -> Option<BasicValueEnum> {
        let count = self.count_params();

        if nth + 1 > count {
            return None;
        }

        let param = unsafe {
            LLVMGetParam(self.fn_value.value, nth)
        };

        Some(BasicValueEnum::new(param))
    }

    pub fn count_params(&self) -> u32 {
        unsafe {
            LLVMCountParams(self.fn_value.value)
        }
    }

    // REVIEW: Untested; probably doesn't work. Should remove transmute.
    pub fn get_basic_blocks(&self) -> Vec<BasicBlock> {
        let mut blocks = vec![];

        unsafe {
            LLVMGetBasicBlocks(self.fn_value.value, blocks.as_mut_ptr());

            transmute(blocks)
        }
    }

    pub fn get_return_type(&self) -> BasicTypeEnum {
        let type_ = unsafe {
            LLVMGetReturnType(LLVMGetElementType(LLVMTypeOf(self.fn_value.value)))
        };

        BasicTypeEnum::new(type_)
    }

    pub fn params(&self) -> ParamValueIter {
        ParamValueIter {
            param_iter_value: self.fn_value.value,
            start: true,
        }
    }

    pub fn get_last_basic_block(&self) -> BasicBlock {
        let bb = unsafe {
            LLVMGetLastBasicBlock(self.fn_value.value)
        };

        BasicBlock::new(bb)
    }

    pub fn get_name(&self) -> &CStr {
        self.fn_value.get_name()
    }

    // REVIEW: Untested
    pub fn view_function_config(&self) {
        unsafe {
            LLVMViewFunctionCFG(self.as_value_ref())
        }
    }

    // REVIEW: Untested
    pub fn view_function_config_only(&self) {
        unsafe {
            LLVMViewFunctionCFGOnly(self.as_value_ref())
        }
    }
}

impl AsValueRef for FunctionValue {
    fn as_value_ref(&self) -> LLVMValueRef {
        self.fn_value.value
    }
}

impl fmt::Debug for FunctionValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let llvm_value = unsafe {
            CStr::from_ptr(LLVMPrintValueToString(self.fn_value.value))
        };
        let llvm_type = unsafe {
            CStr::from_ptr(LLVMPrintTypeToString(LLVMTypeOf(self.fn_value.value)))
        };
        let name = unsafe {
            CStr::from_ptr(LLVMGetValueName(self.fn_value.value))
        };
        let is_const = unsafe {
            LLVMIsConstant(self.fn_value.value) == 1
        };
        let is_null = unsafe {
            LLVMIsNull(self.fn_value.value) == 1
        };

        write!(f, "FunctionValue {{\n    name: {:?}\n    address: {:?}\n    is_const: {:?}\n    is_null: {:?}\n    llvm_value: {:?}\n    llvm_type: {:?}\n}}", name, self.fn_value, is_const, is_null, llvm_value, llvm_type)
    }
}

pub struct ParamValueIter {
    param_iter_value: LLVMValueRef,
    start: bool,
}

impl Iterator for ParamValueIter {
    type Item = BasicValueEnum;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start {
            let first_value = unsafe {
                LLVMGetFirstParam(self.param_iter_value)
            };

            if first_value.is_null() {
                return None;
            }

            self.start = false;

            self.param_iter_value = first_value;

            return Some(BasicValueEnum::new(first_value));
        }

        let next_value = unsafe {
            LLVMGetNextParam(self.param_iter_value)
        };

        if next_value.is_null() {
            return None;
        }

        self.param_iter_value = next_value;

        Some(BasicValueEnum::new(next_value))
    }
}

#[derive(Debug)]
pub struct IntValue {
    int_value: Value,
}

impl IntValue {
    pub(crate) fn new(value: LLVMValueRef) -> Self {
        IntValue {
            int_value: Value::new(value),
        }
    }

    pub fn get_name(&self) -> &CStr {
        self.int_value.get_name()
    }

    pub fn set_name(&self, name: &str) {
        self.int_value.set_name(name);
    }

    pub fn get_type(&self) -> IntType {
        let int_type = unsafe {
            LLVMTypeOf(self.as_value_ref())
        };

        IntType::new(int_type)
    }
}

impl AsValueRef for IntValue {
    fn as_value_ref(&self) -> LLVMValueRef {
        self.int_value.value
    }
}

// TODO: IntoIntValue needs to be reworked. Major flaws:
// * Cannot specify context, even optionally. Currently defaults
//   to global context which is likely not the user's context.
// * Cannot specify type or sign and currently assumes i32. It'd
//   be cool to be able to do 42.into_int_value::<i32>(&context)
//   though that does seem like the kind of verbosity that I was
//   originally trying to avoid with IntoIntValue. May as well do
//   context.i32_type().const_int(42, true);
pub trait IntoIntValue {
    fn into_int_value(&self) -> IntValue;
}

impl IntoIntValue for IntValue {
    fn into_int_value(&self) -> IntValue {
        IntValue::new(self.int_value.value)
    }
}

impl IntoIntValue for u64 {
    fn into_int_value(&self) -> IntValue {
        // REVIEWL This will probably assign the global context, not necessarily the one the user is using.
        IntType::i32_type().const_int(*self, false)
    }
}

#[derive(Debug)]
pub struct FloatValue {
    float_value: Value
}

impl FloatValue {
    pub(crate) fn new(value: LLVMValueRef) -> Self {
        FloatValue {
            float_value: Value::new(value),
        }
    }

    pub fn get_name(&self) -> &CStr {
        self.float_value.get_name()
    }

    pub fn set_name(&self, name: &str) {
        self.float_value.set_name(name);
    }

    pub fn get_type(&self) -> FloatType {
        let float_type = unsafe {
            LLVMTypeOf(self.as_value_ref())
        };

        FloatType::new(float_type)
    }
}

impl AsValueRef for FloatValue {
    fn as_value_ref(&self) -> LLVMValueRef {
        self.float_value.value
    }
}

#[derive(Debug)]
pub struct StructValue {
    struct_value: Value
}

impl StructValue {
    pub(crate) fn new(value: LLVMValueRef) -> Self {
        StructValue {
            struct_value: Value::new(value),
        }
    }

    pub fn get_name(&self) -> &CStr {
        self.struct_value.get_name()
    }

    pub fn set_name(&self, name: &str) {
        self.struct_value.set_name(name);
    }

    pub fn get_type(&self) -> StructType {
        let struct_type = unsafe {
            LLVMTypeOf(self.as_value_ref())
        };

        StructType::new(struct_type)
    }
}

impl AsValueRef for StructValue {
    fn as_value_ref(&self) -> LLVMValueRef {
        self.struct_value.value
    }
}

#[derive(Debug)]
pub struct PointerValue {
    ptr_value: Value
}

impl PointerValue {
    pub(crate) fn new(value: LLVMValueRef) -> Self {
        PointerValue {
            ptr_value: Value::new(value),
        }
    }

    pub fn get_name(&self) -> &CStr {
        self.ptr_value.get_name()
    }

    pub fn set_name(&self, name: &str) {
        self.ptr_value.set_name(name);
    }

    pub fn get_type(&self) -> PointerType {
        let pointer_type = unsafe {
            LLVMTypeOf(self.as_value_ref())
        };

        PointerType::new(pointer_type)
    }
}

impl AsValueRef for PointerValue {
    fn as_value_ref(&self) -> LLVMValueRef {
        self.ptr_value.value
    }
}

#[derive(Debug)]
pub struct PhiValue {
    phi_value: Value
}

impl PhiValue {
    pub(crate) fn new(value: LLVMValueRef) -> Self {
        PhiValue {
            phi_value: Value::new(value),
        }
    }

    pub fn add_incoming(&self, incoming_values: &AnyValue, incoming_basic_block: &BasicBlock, count: u32) {
        self.phi_value.add_incoming(incoming_values, incoming_basic_block, count)
    }

    pub fn get_name(&self) -> &CStr {
        self.phi_value.get_name()
    }
}

impl AsValueRef for PhiValue {
    fn as_value_ref(&self) -> LLVMValueRef {
        self.phi_value.value
    }
}

impl AsValueRef for Value { // TODO: Remove
    fn as_value_ref(&self) -> LLVMValueRef {
        self.value
    }
}

pub struct ArrayValue {
    array_value: Value
}

impl ArrayValue {
    pub(crate) fn new(value: LLVMValueRef) -> Self {
        ArrayValue {
            array_value: Value::new(value),
        }
    }

    pub fn get_name(&self) -> &CStr {
        self.array_value.get_name()
    }

    pub fn set_name(&self, name: &str) {
        self.array_value.set_name(name);
    }

    pub fn get_type(&self) -> ArrayType {
        let array_type = unsafe {
            LLVMTypeOf(self.as_value_ref())
        };

        ArrayType::new(array_type)
    }
}

impl AsValueRef for ArrayValue {
    fn as_value_ref(&self) -> LLVMValueRef {
        self.array_value.value
    }
}

impl fmt::Debug for ArrayValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let llvm_value = unsafe {
            CStr::from_ptr(LLVMPrintValueToString(self.as_value_ref()))
        };
        let llvm_type = unsafe {
            CStr::from_ptr(LLVMPrintTypeToString(LLVMTypeOf(self.as_value_ref())))
        };
        let name = unsafe {
            CStr::from_ptr(LLVMGetValueName(self.as_value_ref()))
        };
        let is_const = unsafe {
            LLVMIsConstant(self.as_value_ref()) == 1
        };
        let is_null = unsafe {
            LLVMIsNull(self.as_value_ref()) == 1
        };
        let is_const_array = unsafe {
            !LLVMIsAConstantArray(self.as_value_ref()).is_null()
        };
        let is_const_data_array = unsafe {
            !LLVMIsAConstantDataArray(self.as_value_ref()).is_null()
        };

        write!(f, "Value {{\n    name: {:?}\n    address: {:?}\n    is_const: {:?}\n    is_const_array: {:?}\n    is_const_data_array: {:?}\n    is_null: {:?}\n    llvm_value: {:?}\n    llvm_type: {:?}\n}}", name, self.as_value_ref(), is_const, is_const_array, is_const_data_array, is_null, llvm_value, llvm_type)
    }
}

macro_rules! trait_value_set {
    ($trait_name:ident: $($args:ident),*) => (
        pub trait $trait_name: AsValueRef {}

        $(
            impl $trait_name for $args {}
        )*
    );
}

macro_rules! enum_value_set {
    ($enum_name:ident: $($args:ident),*) => (
        #[derive(Debug)]
        pub enum $enum_name {
            $(
                $args($args),
            )*
        }

        impl AsValueRef for $enum_name {
            fn as_value_ref(&self) -> LLVMValueRef {
                match *self {
                    $(
                        $enum_name::$args(ref t) => t.as_value_ref(),
                    )*
                }
            }
        }

        $(
            impl From<$args> for $enum_name {
                fn from(value: $args) -> $enum_name {
                    $enum_name::$args(value)
                }
            }
        )*
    );
}

enum_value_set! {AnyValueEnum: ArrayValue, IntValue, FloatValue, PhiValue, FunctionValue, PointerValue, StructValue}
enum_value_set! {BasicValueEnum: ArrayValue, IntValue, FloatValue, PointerValue, StructValue}

trait_value_set! {AnyValue: AnyValueEnum, BasicValueEnum, ArrayValue, IntValue, FloatValue, PhiValue, PointerValue, FunctionValue, StructValue, Value} // TODO: Remove Value
trait_value_set! {BasicValue: ArrayValue, BasicValueEnum, IntValue, FloatValue, StructValue, PointerValue}

impl BasicValueEnum {
    pub(crate) fn new(value: LLVMValueRef) -> BasicValueEnum {
        let type_kind = unsafe {
            LLVMGetTypeKind(LLVMTypeOf(value))
        };

        match type_kind {
            LLVMTypeKind::LLVMFloatTypeKind |
            LLVMTypeKind::LLVMFP128TypeKind |
            LLVMTypeKind::LLVMDoubleTypeKind |
            LLVMTypeKind::LLVMHalfTypeKind |
            LLVMTypeKind::LLVMX86_FP80TypeKind |
            LLVMTypeKind::LLVMPPC_FP128TypeKind => BasicValueEnum::FloatValue(FloatValue::new(value)),
            LLVMTypeKind::LLVMIntegerTypeKind => BasicValueEnum::IntValue(IntValue::new(value)),
            LLVMTypeKind::LLVMStructTypeKind => BasicValueEnum::StructValue(StructValue::new(value)),
            LLVMTypeKind::LLVMPointerTypeKind => BasicValueEnum::PointerValue(PointerValue::new(value)),
            LLVMTypeKind::LLVMArrayTypeKind => BasicValueEnum::ArrayValue(ArrayValue::new(value)),
            LLVMTypeKind::LLVMVectorTypeKind => panic!("TODO: Unsupported type: Vector"),
            _ => unreachable!("Unsupported type"),
        }
    }

    pub fn get_type(&self) -> BasicTypeEnum {
        let type_ = unsafe {
            LLVMTypeOf(self.as_value_ref())
        };

        BasicTypeEnum::new(type_)
    }

    pub fn into_int_value(self) -> IntValue {
        if let BasicValueEnum::IntValue(i) = self {
            i
        } else {
            panic!("Called BasicValueEnum.into_int_value on {:?}", self);
        }
    }

    pub fn into_float_value(self) -> FloatValue {
        if let BasicValueEnum::FloatValue(f) = self {
            f
        } else {
            panic!("Called BasicValueEnum.into_float_value on {:?}", self);
        }
    }

    pub fn into_pointer_value(self) -> PointerValue {
        if let BasicValueEnum::PointerValue(p) = self {
            p
        } else {
            panic!("Called BasicValueEnum.into_pointer_value on {:?}", self);
        }
    }

    pub fn into_struct_value(self) -> StructValue {
        if let BasicValueEnum::StructValue(s) = self {
            s
        } else {
            panic!("Called BasicValueEnum.into_struct_value on {:?}", self);
        }
    }

    pub fn into_array_value(self) -> ArrayValue {
        if let BasicValueEnum::ArrayValue(a) = self {
            a
        } else {
            panic!("Called BasicValueEnum.into_array_value on {:?}", self);
        }
    }

    pub fn as_int_value(&self) -> &IntValue {
        if let BasicValueEnum::IntValue(ref i) = *self {
            i
        } else {
            panic!("Called BasicValueEnum.as_int_value on {:?}", self);
        }
    }

    pub fn as_float_value(&self) -> &FloatValue {
        if let BasicValueEnum::FloatValue(ref f) = *self {
            f
        } else {
            panic!("Called BasicValueEnum.as_float_value on {:?}", self);
        }
    }

    pub fn as_pointer_value(&self) -> &PointerValue {
        if let BasicValueEnum::PointerValue(ref p) = *self {
            p
        } else {
            panic!("Called BasicValueEnum.as_pointer_value on {:?}", self);
        }
    }

    pub fn as_struct_value(&self) -> &StructValue {
        if let BasicValueEnum::StructValue(ref s) = *self {
            s
        } else {
            panic!("Called BasicValueEnum.as_struct_value on {:?}", self);
        }
    }

    pub fn as_array_value(&self) -> &ArrayValue {
        if let BasicValueEnum::ArrayValue(ref a) = *self {
            a
        } else {
            panic!("Called BasicValueEnum.as_array_value on {:?}", self);
        }
    }

    pub fn is_int_value(&self) -> bool {
        if let BasicValueEnum::IntValue(_) = *self {
            true
        } else {
            false
        }
    }

    pub fn is_float_value(&self) -> bool {
        if let BasicValueEnum::FloatValue(_) = *self {
            true
        } else {
            false
        }
    }

    pub fn is_pointer_value(&self) -> bool {
        if let BasicValueEnum::PointerValue(_) = *self {
            true
        } else {
            false
        }
    }

    pub fn is_struct_value(&self) -> bool {
        if let BasicValueEnum::StructValue(_) = *self {
            true
        } else {
            false
        }
    }

    pub fn is_array_value(&self) -> bool {
        if let BasicValueEnum::ArrayValue(_) = *self {
            true
        } else {
            false
        }
    }
}