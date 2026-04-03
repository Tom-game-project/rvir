use std::collections::HashMap;
use std::marker::PhantomData;

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
pub struct VRegInfo {
    pub v_reg: VReg,
    pub crossed_call: bool, // 生存期間中に関数のcallがある
    pub life_span: LifeSpan // 生存期間
}

#[derive(Clone, Copy, Debug)]
pub struct LifeSpan(pub usize, pub usize);

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

use std::collections::HashSet;

pub struct Setting;
/// IRの設定が終了
pub struct IRSetted;
pub struct AllSetted;

pub struct BasicBlockList<State> {
    pub list: Vec<BasicBlock>,
    _phantom: PhantomData<State>
}

impl BasicBlockList<Setting> {
    pub fn new() -> Self {
        Self { list: Vec::new(), _phantom: PhantomData }
    }

    pub fn alloc(&mut self, label: Label) -> BasicBlockId {
        let index = BasicBlockId(self.list.len());
        self.list.push(
            BasicBlock { 
                label,
                id: index, 
                pred: Vec::new(), 
                succ: Vec::new(), 
                insts: Vec::new(), 
                reach: HashSet::new() 
            }
        );
        index
    }

    pub fn set_inst(&mut self, id: BasicBlockId, insts: Vec<Instruction>) {
        self.list[id.0].insts = insts;
    }

    /// irの設定が終了したら、状態を移す
    pub fn finish_ir_setting (self) -> BasicBlockList<IRSetted> {
        BasicBlockList { list: self.list, _phantom: PhantomData }
    }
}

pub enum BasicBlockListError {
    UndefinedLabel(Label)
}

fn labelmap2idmap(label_map: HashMap<&Label, Vec<&Label>>, basic_block_list: &BasicBlockList<IRSetted>) -> Result<HashMap<BasicBlockId, Vec<BasicBlockId>>, BasicBlockListError> {
    Ok(label_map
        .iter()
        .map(|(&k, v)| {
            let new_k = basic_block_list
                .get_basic_block_by_label(&k)
                .ok_or_else(|| BasicBlockListError::UndefinedLabel(k.clone())
            )?;
            let new_v= v.iter().map(|&c| {
                basic_block_list
                    .get_basic_block_by_label(c)
                    .ok_or_else(|| BasicBlockListError::UndefinedLabel(c.clone()))
            })
            .collect::<Result<Vec<_>, _>>()?
            .iter()
            .map(|a| a.id)
            .collect::<Vec<BasicBlockId>>();
            Ok((new_k.id, new_v))
        })
        .collect::<Result<Vec<(BasicBlockId, Vec<BasicBlockId>)>, BasicBlockListError>>()?
        .into_iter()
        .fold(
            HashMap::<BasicBlockId, Vec<BasicBlockId>, _>::new(),
            |mut acc:HashMap<BasicBlockId, Vec<BasicBlockId>>, (k, v)| {
                acc.insert(k, v);
                acc
            }
        ))
}

/// IRの設定が終わったときに存在するメソッド
impl BasicBlockList<IRSetted> {

    pub fn get_basic_block(&self, id: BasicBlockId) -> &BasicBlock {
        &self.list[id.0]
    }

    pub fn get_basic_block_by_label(&self, label: &Label) -> Option<&BasicBlock>{
        self.list.iter().find(|b | b.label == *label)
    }

    /// TODO: グラフを作る
    /// できればBasicBlockListの状態を変えてirを再設定できないようにする
    ///
    /// ir設定後　呼び出されることを想定する
    pub fn set_pred_and_succ(mut self) -> Result<BasicBlockList<AllSetted>, BasicBlockListError> {
        let mut pred_dict : HashMap<&Label, Vec<&Label>> = HashMap::new();
        let mut succ_dict : HashMap<&Label, Vec<&Label>> = HashMap::new();

        for i in &self.list {
            pred_dict.insert(&i.label, Vec::new());
            succ_dict.insert(&i.label, Vec::new());
        }

        for i in &self.list { // ブロックそれぞれについて
            let succ = i.extract_successor_labels();

            succ_dict.insert(&i.label, succ.clone());
            for j in succ { // それぞれのsuccとなっているラベルを主体としてみたとき
                if let Some(pred) = pred_dict.get_mut(j /* この後続(succ)のラベルにとってiは先行するラベルなので追加する */) {
                    pred.push(&i.label);
                } else {
                    return Err(BasicBlockListError::UndefinedLabel(j.clone())); 
                }
            }
        }

        let pred_dict = labelmap2idmap(pred_dict, &self)?;
        let succ_dict = labelmap2idmap(succ_dict, &self)?;

        for i in &mut self.list {
            i.pred = pred_dict
                .get(&i.id)
                .unwrap() // 一旦全てのラベルに対して初期化しているので、ここは安全なunwrap
                .clone();

            i.succ = succ_dict
                .get(&i.id)
                .unwrap() // 一旦全てのラベルに対して初期化しているので、ここは安全なunwrap
                .clone();
        }

        Ok(BasicBlockList {
            list: self.list,
            _phantom: PhantomData 
        })
    }
}

impl BasicBlockList<AllSetted> {

}

#[derive(Hash, Copy, Clone, PartialEq, Eq, Debug)]
pub struct BasicBlockId(usize);

impl BasicBlockId {
    pub fn new(id: usize) -> Self {
        Self(id)
    }
}

/// 基本ブロックの定義
///
/// 1) 次の文を基本ブロックの先頭とする
///
///   a. そのプログラムの先頭の文
///   b. 飛越し文の行先の先頭 (条件付き含む)
/// 2) 次までを基本ブロックとする
///   次の 1)のような文の直前まで
///   1.aは何かの次の行にはなり得ないので、簡単に言えば、飛越し文まで
///   
pub struct BasicBlock {
    pub label: Label,               // ブロックの入り口（ここにラベルを持つ）
    pub id: BasicBlockId,
    pub pred: Vec<BasicBlockId>,
    pub succ: Vec<BasicBlockId>,
    pub insts: Vec<Instruction>,    // 中身の命令列（ここにはLabelDefは絶対入らない）
    pub reach: HashSet<VReg>,
}

impl BasicBlock {
    /// BasicBlockの最後のブロックの可能なジャンプ先を調べる
    fn extract_successor_labels(&self) -> Vec<&Label>{
        if let Some(a) = self.insts.last() {
            match a {
                Instruction::Jump { target } => {
                    vec![target]
                } 
                Instruction::Branch { cond:_cond, true_label, false_label } => {
                    vec![true_label, false_label]
                } 
                _ => {
                    Vec::new()
                }
            }
        } else {
            Vec::new()
        }
    }
}

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
        cond: Operand,      // Operandが0のときfalse label
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
    use crate::unit::size::Byte;
    use crate::ir::ir::{BasicBlockList, ConstValue, Func, FuncDef, Instruction, Label, ModuleContext, Operand, Operator, RvIR};

    #[test]
    fn test00 (){
        let mut mod_ctx = ModuleContext::new();

        let func_id = mod_ctx.create_func(
            "test_func",
            Byte::new(0),
            Byte::new(0x10 * 5));

        {
            let func = mod_ctx.get_func_mut(func_id).unwrap();
            let tmp_reg_i = func.vreg_arena.alloc(Byte::new(8), Some(String::from("i")));
            let tmp_reg_j = func.vreg_arena.alloc(Byte::new(8), Some(String::from("j")));
            let tmp_reg_c = func.vreg_arena.alloc(Byte::new(8), Some(String::from("c")));

            let mut basic_block_list = BasicBlockList::new();

            // irのユーザーは事前に基本ブロックを構成する必要がある
            let block_id_0 = basic_block_list.alloc(Label("block0".to_string()));
            let block_id_1 = basic_block_list.alloc(Label("block1".to_string()));
            let block_id_2 = basic_block_list.alloc(Label("block2".to_string()));
            let block_id_3 = basic_block_list.alloc(Label("block3".to_string()));
            let block_id_4 = basic_block_list.alloc(Label("block4".to_string()));
            let block_id_5 = basic_block_list.alloc(Label("block5".to_string()));

            basic_block_list.set_inst(
                block_id_0,
                vec![
                Instruction::Assign { dest: tmp_reg_j, src: Operand::Const(ConstValue::I64(10)) },
                Instruction::Assign { dest: tmp_reg_i, src: Operand::Const(ConstValue::I64(-8)) },
                Instruction::Jump { target: Label("block1".to_string()) }
            ]);

            basic_block_list.set_inst( block_id_1, vec![
                Instruction::BinOp { op: Operator::Add, dest: tmp_reg_i, lhs: Operand::Reg(tmp_reg_i), rhs: Operand::Const(ConstValue::I64(1)) },
                Instruction::Jump { target: Label("block2".to_string()) }
            ]);

            basic_block_list.set_inst(block_id_2, vec![
                Instruction::BinOp { op: Operator::Sub, dest: tmp_reg_j, lhs: Operand::Reg(tmp_reg_j), rhs: Operand::Const(ConstValue::I64(1)) },
                Instruction::BinOp { op: Operator::Neq, dest: tmp_reg_c, lhs: Operand::Reg(tmp_reg_j), rhs: Operand::Const(ConstValue::I64(0)) },
                Instruction::Branch {
                    cond: Operand::Reg(tmp_reg_c), 
                    true_label: Label("block2".to_string()), 
                    false_label: Label("block3".to_string()) 
                }
            ]);

            basic_block_list.set_inst(block_id_3, vec![ 
                Instruction::BinOp { op: Operator::Div, dest: tmp_reg_j, lhs: Operand::Reg(tmp_reg_i), rhs: Operand::Const(ConstValue::I64(2)) },
                Instruction::BinOp { op: Operator::Eq, dest: tmp_reg_c, lhs: Operand::Reg(tmp_reg_i), rhs: Operand::Const(ConstValue::I64(8)) }, // サンプルのため教科書とは一致しない部分
                Instruction::Branch {
                    cond: Operand::Reg(tmp_reg_c), 
                    true_label: Label("block4".to_string()), 
                    false_label: Label("block5".to_string()) 
                }

            ]);

            basic_block_list.set_inst(block_id_4, vec![
                Instruction::Assign { dest: tmp_reg_i, src: Operand::Const(ConstValue::I64(2)) },
                Instruction::Jump { target: Label("block5".to_string()) }
            ]);

            basic_block_list.set_inst(block_id_5, vec![ 
                Instruction::Jump { target: Label("block2".to_string()) }
            ]);

            let basic_block_list = basic_block_list.finish_ir_setting();
            if let Ok(basic_block_list ) = basic_block_list.set_pred_and_succ() {
                for basic_block in &basic_block_list.list {
                    println!("{}:", basic_block.label.0);
                    println!("    basic_block id: {}", basic_block.id.0);
                    println!("    basic_block pred: {:?}", basic_block.pred);
                    println!("    basic_block succ: {:?}", basic_block.succ);
                }
            } else {
                println!("failed to setting basic_block_list");
            }
        }

    }
}

