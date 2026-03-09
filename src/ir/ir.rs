use crate::unit::size::Byte;

// vertual register
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VReg(pub usize);

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
    Const(i32),
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

    Call {
        dest: Option<Dest>,
        func_name: Func,
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
    name: String, // name of function
    func_id: FuncId,
    pub arg_size: Byte,
    pub local_size: Byte, // local value size
    ir: RvIR,
}

impl FuncDef {
    fn new (name: &str, arg_size: Byte /*byte*/, local_size: Byte /*byte*/, func_id: FuncId) -> Self {
        Self {
            name: name.to_string(),
            arg_size,
            local_size,
            func_id,
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

    pub fn get_func_mut(&mut self, id: FuncId) -> &mut FuncDef {
        &mut self.funcs[id.0]
    }

    pub fn get_func(&self, id: FuncId) -> &FuncDef {
        &self.funcs[id.0]
    }
}
