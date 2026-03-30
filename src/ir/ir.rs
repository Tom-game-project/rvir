use std::collections::HashMap;

use crate::unit::size::Byte;

// vertual register
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VReg(pub usize);

pub trait RegisterAllocator {
    /// Analyze the IR and argument information, and assign a Location to every VReg in VregArena.
    /// It returns the final 16-byte-aligned stack size.
    fn allocate(
        &mut self,
        ir: &RvIR,
        arena: &mut VregArena,
        args: &[VReg],
    ) -> Byte;
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConstValue {
    I32(i32),
    I64(i64),
    U32(u32),
    U64(u64),
}

impl ConstValue {
    pub fn get_size(&self) -> Byte {
        match self {
            ConstValue::I32(_) | ConstValue::U32(_) => {
                Byte::new(4)
            }
            ConstValue::I64(_) | ConstValue::U64(_) => {
                Byte::new(8)
            }
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BasePointer {
    Sp(i32), // stack pointer
    Fp(i32), // frame pointer
}

impl BasePointer {
    pub fn get_base_and_offset(&self) -> (PhysReg, i32) {
        match self {
            BasePointer::Sp(offset) => (PhysReg::Sp, *offset),
            BasePointer::Fp(offset) => (PhysReg::Fp, *offset),
        }
    }
}

// TODO
use crate::codegen::rv64::{PhysReg, PhysStack};
use crate::codegen::rv64_asm::*; 

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Location {
    Register(PhysReg),
    Stack(PhysStack), // frame offset
                      // fp base
}

impl Location {
    pub fn gen_load_asm(&self, other: &Location) -> Result<Asm, GenAsmErr> {
        match self {
            Location::Stack(phys_stack) => phys_stack.gen_load_asm(other),
            Location::Register(phys_reg) => phys_reg.gen_load_asm(other)
        }
    }
}

pub struct VRegData {
    pub v_reg: VReg,
    pub name: String,
    pub size: Byte,
    pub location: Option<Location> // TODO
                                   // in the future, I try to optimize this alloc by using register
}

#[derive(Debug)]
pub enum GenAsmErr {
    VregNotLocated,
    UnsupportedByteAlignment,
    NotImplimentedYet,
}

pub struct VregArena {
    pub regs: Vec<VRegData>
}

impl VregArena {

    pub fn alloc(&mut self, size: Byte, name: Option<String>) -> VReg {
        let v_reg = VReg(self.regs.len());
        let reg = VRegData {
            v_reg,
            name: name.expect("name does not set"),
            size,
            location: None,
        };
        self.regs.push(reg);
        v_reg
    }

    pub fn get_vregdata(&self, vreg: &VReg) -> Option<&VRegData> {
        Some(self
            .regs
            .iter()
            .find(|a| &a.v_reg == vreg)?)
    }
}

// function identifer
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FuncId (pub usize);

// function
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Func(pub FuncId);

// jump label
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Label(pub String);

// default operator
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operator {
    Add, Sub, Mul, Div, Eq, Neq,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Operand {
    Const(ConstValue),
    Reg(VReg),
}

pub type Dest = VReg;

#[derive(Debug, Clone)]
pub enum Instruction {
    BinOp {
        op: Operator,
        dest: Dest,
        lhs: Operand,
        rhs: Operand,
    },

    Assign {
        dest: Dest,
        src: Operand,
    },

    // dest = func(operand...);
    Call {
        dest: Option<Dest>,
        func: Func,
        args: Vec<Operand>,
    },

    Ret {
        val: Option<Operand>,
    },

    Alloca {
        dest: Dest,
        size: Byte,
    },
    
    Jump {
        target: Label,
    },

    Branch {
        cond: Operand,
        true_label: Label,
        false_label: Label,
    }
}

// IR
pub struct RvIR (pub Vec<Instruction>);

// Func Definition
pub struct FuncDef {
    pub name: String, // name of function
    func_id: FuncId,
    pub args: Vec<VReg>,
    pub arg_size: Byte,
    pub local_size: Byte, // local value size

    pub vreg_arena: VregArena,
    pub ir: RvIR,
}

impl FuncDef {
    pub fn new (
        name: &str,
        arg_size: Byte /*byte*/, 
        local_size: Byte /*byte*/,
        // func_id: FuncId,
        // args: Vec<VReg>
) -> Self {
        Self {
            name: name.to_string(),
            arg_size,
            local_size,
            func_id: FuncId(0),
            args: vec![],
            vreg_arena: VregArena { regs: vec![] },
            ir: RvIR(vec![])
        }
    }

    pub fn set_ir(&mut self, rv_ir: RvIR) {
        self.ir = rv_ir;
    }

    pub fn set_args(&mut self, args: Vec<VReg>) {
        self.args = args;
    }
}


pub struct Symbols (
    pub HashMap<FuncId, String>
);

// Set of functions
pub struct ModuleContext {
    pub symbols: Symbols,
    pub funcs: Vec<FuncDef>
} impl ModuleContext { 

    pub fn new() -> Self { Self { symbols: Symbols(HashMap::new()), funcs: vec![] } }

    pub fn create_func(&mut self, name: &str, arg_size: Byte, local_size: Byte) -> FuncId {
        let func_id = FuncId(self.funcs.len());
        
        // 1. シンボルテーブルに名前を「先」に登録する
        self.symbols.0.insert(func_id, name.to_string());
        
        // 2. IRが空っぽの（未完成な）FuncDefをリストに追加する
        let empty_func = FuncDef::new(
            name, 
            arg_size, 
            local_size, 
        );
        self.funcs.push(empty_func);

        // 3. 確定したIDを返す
        func_id
    }

    pub fn add_func(&mut self, mut func_def: FuncDef) -> FuncId {
        let func_id = FuncId(self.funcs.len());
        func_def.func_id = func_id;
        self.symbols.0.insert(func_id, func_def.name.clone());
        self.funcs.push(
            func_def
        );
        func_id
    }

    pub fn get_func_mut(&mut self, id: FuncId) -> Option<&mut FuncDef> {
        self.funcs.get_mut(id.0)
    }

    pub fn get_func(&self, id: FuncId) -> Option<&FuncDef> {
        self.funcs.get(id.0)
    }
}

#[cfg(test)]
mod ir_ir_test {
    use crate::*;

    // #[test]
    // fn test00 (){
    // 
    // }
}

