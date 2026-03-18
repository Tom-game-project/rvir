use crate::unit::size::{Byte, Size};

// vertual register
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VReg(pub usize);

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PhysReg {
    T0/*Reserved*/, T1, T2, T3, T4, T5, T6, // temp reg
    S1, S2, S3, S4, S5, S6, S7, S8, S9, S10, S11, // save reg
    A0, A1, A2, A3, A4, A5, A6, A7, // function args
}

impl PhysReg {
    fn get_reg_name(&self) -> &'static str {
        match &self {
            Self::T0 => "t0",
            Self::T1 => "t1",
            Self::T2 => "t2",
            Self::T3 => "t3",
            Self::T4 => "t4",
            Self::T5 => "t5",
            Self::T6 => "t6",
            Self::S1 => "s1",
            Self::S2 => "s2",
            Self::S3 => "s3",
            Self::S4 => "s4",
            Self::S5 => "s5",
            Self::S6 => "s6",
            Self::S7 => "s7",
            Self::S8 => "s8",
            Self::S9 => "s9",
            Self::S10 => "s10",
            Self::S11 => "s11",
            Self::A0 => "a0", // return value
            Self::A1 => "a1", // return value
            Self::A2 => "a2",
            Self::A3 => "a3",
            Self::A4 => "a4",
            Self::A5 => "a5",
            Self::A6 => "a6",
            Self::A7 => "a7",
        }
    }

    /// 
    pub fn gen_load_asm(&self, location: &Location) -> Result<String, GenAsmErr> {
        match location {
            Location::Register(reg) => {
                Ok(format!("mv {}, {}\n", self.get_reg_name(), reg.get_reg_name()))
            }
            Location::Stack(other_stack) => {
                if other_stack.size == Size::new(4) {
                    Ok(format!("lw {}, {}", self.get_reg_name(), other_stack.offset.gen_asm()))
                } else if other_stack.size == Size::new(8) {
                    Ok(format!("ld {}, {}", self.get_reg_name(), other_stack.offset.gen_asm()))
                } else {
                    Err(GenAsmErr::UnsupportedByteAlignment)
                }
            }
        }
    }

    pub fn gen_load_immediate(&self, const_val: &ConstValue) -> String {
        match const_val {
            ConstValue::I32(a) => 
                format!("li {}, {}", self.get_reg_name(), a),
            ConstValue::I64(a) => 
                format!("li {}, {}", self.get_reg_name(), a),
            ConstValue::U32(a) => 
                format!("li {}, {}", self.get_reg_name(), a),
            ConstValue::U64(a) => 
                format!("li {}, {}", self.get_reg_name(), a),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum BasePointer {
    Sp(i32), // stack pointer
    Fp(i32), // frame pointer
}

impl BasePointer {
    fn gen_asm(&self) -> String {
        match self {
            BasePointer::Sp(offset) => format!("{}(sp)", offset),
            BasePointer::Fp(offset) => format!("{}(fp)", offset),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PhysStack {
    offset: BasePointer,
    size: Byte       // 4 byte or 8 byte
}

impl PhysStack {
    pub fn new(offset: BasePointer, size:Byte) -> Self {
        Self { offset, size }
    }

    pub fn gen_load_asm(&self, location: &Location) -> Result<String, GenAsmErr> {
        match location {
            Location::Stack(other_stack) => {
                if self.size == Size::new(4) {
                    Ok(
                        format!("lw t0, {}\n",  other_stack.offset.gen_asm()) +
                        &format!("sw t0, {}", self.offset.gen_asm())
                    )
                } else if self.size == Size::new(8) {
                    Ok(
                        format!("ld t0, {}\n",  other_stack.offset.gen_asm()) +
                        &format!("sd t0, {}", self.offset.gen_asm())
                    )
                } else {
                    return Err(GenAsmErr::UnsupportedByteAlignment)
                }
            }
            Location::Register(other_reg) => {
                if self.size == Size::new(4) {
                    Ok(format!("sw {}, {}", other_reg.get_reg_name(), self.offset.gen_asm()))
                } else if self.size == Size::new(8) {
                    Ok(format!("sd {}, {}", other_reg.get_reg_name(), self.offset.gen_asm()))
                } else {
                    return Err(GenAsmErr::UnsupportedByteAlignment)
                }
            }
        }
    }

    pub fn gen_load_immediate (&self, const_val: &ConstValue) -> String {
        match const_val {
            ConstValue::I32(a) => 
                format!("li t0, {}", a) + 
                &format!("sw t0, {}", self.offset.gen_asm()),
            ConstValue::I64(a) => 
                format!("li t0, {}", a) + 
                &format!("sd t0, {}", self.offset.gen_asm()),
            ConstValue::U32(a) => 
                format!("li t0, {}", a) + 
                &format!("sw t0, {}", self.offset.gen_asm()),
            ConstValue::U64(a) => 
                format!("li t0, {}", a) + 
                &format!("sd t0, {}", self.offset.gen_asm()),
        }
    }
}


#[derive(Debug, Clone, PartialEq)]
pub enum Location {
    Register(PhysReg),
    Stack(PhysStack), // frame offset
                            // fp base
}

pub struct VRegData {
    pub v_reg: VReg,
    pub name: String,
    pub size: Byte,
    pub location: Option<Location> // TODO
                                   // in the future, I try to optimize this alloc by using register
}

pub enum GenAsmErr {
    VregNotLocated,
    UnsupportedByteAlignment,
}

impl VRegData {
    /// ```
    /// self = other;
    /// ```
    fn get_assign_asm(&self, other: &VRegData) -> Result<String, GenAsmErr> {
        match (
            if let Some(a) = &self.location {a} else {return Err(GenAsmErr::VregNotLocated);},
            if let Some(a) = &other.location {a} else {return Err(GenAsmErr::VregNotLocated);}
        ) {
            (Location::Register(self_reg), location) => self_reg.gen_load_asm(location),
            (Location::Stack(self_stack), location) => self_stack.gen_load_asm(location),
        }
    }

}

pub struct VregArena {
    pub regs: Vec<VRegData>
}

impl VregArena {
    fn new() -> Self {
        Self { regs: Vec::new() }
    }

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

    pub fn assign_locations(&mut self) -> Byte {
        todo!()
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
    pub arg_size: Byte,
    pub local_size: Byte, // local value size

    pub vreg_arena: VregArena,
    pub ir: RvIR,
}

impl FuncDef {
    fn new (name: &str, arg_size: Byte /*byte*/, local_size: Byte /*byte*/, func_id: FuncId) -> Self {
        Self {
            name: name.to_string(),
            arg_size,
            local_size,
            func_id,
            vreg_arena: VregArena { regs: vec![] },
            ir: RvIR(vec![])
        }
    }

    pub fn set_ir(&mut self, rv_ir: RvIR) {
        self.ir = rv_ir;
    }
}

// Set of functions
pub struct ModuleContext {
    funcs: Vec<FuncDef>
} impl ModuleContext { 

    pub fn new() -> Self { Self { funcs: vec![] } }

    pub fn create_func(&mut self, name: &str, args: Byte, local_stack_size: Byte) -> FuncId {
        let func_id = FuncId(self.funcs.len());
        self.funcs.push(FuncDef::new(name, args, local_stack_size, func_id));
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
