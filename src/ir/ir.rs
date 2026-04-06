use std::collections::HashMap;
use std::marker::PhantomData;
use std::os::linux::raw::stat;

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
    pub statement_list: StatementList,
    _phantom: PhantomData<State>
}

impl BasicBlockList<Setting> {
    pub fn new() -> Self {
        Self { 
            list: Vec::new(),
            statement_list: StatementList { statements: Vec::new() },
            _phantom: PhantomData }
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
                vreg_state_set: VregStateSet { 
                    reach: Vec::new(),
                    def: Vec::new(),
                    kill: Vec::new(),
                },
                // 定義、生存区間解析用
                statement_id_list: Vec::new(),
            }
        );
        index
    }

    pub fn set_inst(&mut self, id: BasicBlockId, insts: Vec<Instruction>) {
        self.list[id.0].insts = insts;
    }

    /// irの設定が終了したら、状態を移す
    pub fn finish_ir_setting (self) -> BasicBlockList<IRSetted> {
        BasicBlockList { 
            list: self.list,
            statement_list: StatementList { statements: Vec::new() },
            _phantom: PhantomData 
        }
    }
}

pub enum BasicBlockListError {
    UndefinedLabel(Label)
}

// =================================================================================
//                   helper function for BasicBlockList<IRSetted>
// =================================================================================

fn labelmap2idmap(
    label_map: HashMap<&Label, Vec<&Label>>, 
    basic_block_list: &BasicBlockList<IRSetted>
) -> Result<HashMap<BasicBlockId, Vec<BasicBlockId>>, BasicBlockListError> 
{
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

fn generate_and_set_statement_ids(j: &Instruction, statement_list: &mut StatementList) -> Option<StatementId> {
    match j { 
        Instruction::Call { dest, func:_func, args } => {
            let statement_id = statement_list
                .gen_vreg_statement(
                    *dest,
                    args
                    .iter()
                    .filter_map(|operand| 
                        match *operand {
                            Operand::Const(_) => None,
                            Operand::Reg(vreg)=> Some(vreg)
                    })
                    .collect::<Vec<VReg>>());
            Some(statement_id)
        }

        Instruction::BinOp { op: _op, dest, lhs, rhs } => {
            let mut p_uses_x =  Vec::new();
            if let Operand::Reg(vreg) = lhs {
                p_uses_x.push(*vreg);
            }
            if let Operand::Reg(vreg) = rhs {
                p_uses_x.push(*vreg);
            }
            let statement_id = statement_list
                .gen_vreg_statement(
                    Some(*dest),
                    p_uses_x);
            Some(statement_id)
        }

        Instruction::Assign { dest, src } => {
            let statement_id = statement_list
                .gen_vreg_statement(
                    Some(*dest),
                    if let Operand::Reg(vreg) = src { 
                        vec![*vreg] 
                    } else { 
                        Vec::new()
                    }
                );
            Some(statement_id)
        }

        _ => {
            None
        }
    }
}

/// IRの設定が終わったときに存在するメソッド
impl BasicBlockList<IRSetted> {

    pub fn get_basic_block(&self, id: BasicBlockId) -> &BasicBlock {
        &self.list[id.0]
    }

    pub fn get_basic_block_by_label(&self, label: &Label) -> Option<&BasicBlock> {
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

        for basic_block in &mut self.list {
            basic_block.pred = pred_dict
                .get(&basic_block.id)
                .unwrap() // 一旦全てのラベルに対して初期化しているので、ここは安全なunwrap
                .clone();

            basic_block.succ = succ_dict
                .get(&basic_block.id)
                .unwrap() // 一旦全てのラベルに対して初期化しているので、ここは安全なunwrap
                .clone();

            for j in &basic_block.insts {
                // 式を「定義」、または「使用」するような命令について、statement_idを割り当てる
                if let Some(statement_id) = generate_and_set_statement_ids(j, &mut self.statement_list /* 発行されたstatement_idに対応付けられたデータはここに保存される */) {
                    basic_block.statement_id_list.push(statement_id);
                }
            }
        }

        // self.statement_listですべてのブロック内に存在する文の収集が完了する

        // def, kill,の初期化

        // DEF(B)の設定
        // ある変数xについてp(def(x)) ∈ Bで且つpの後にp'(def(x)) ∈ Bなるp'が無い
        // Bの中で定義され、Bの出口まで有効な文の集合
        for basic_block in &mut self.list {
            basic_block.vreg_state_set = VregStateSet::new(&self.statement_list);

            // DEF(B:Block): Set<StatementId>
            let mut def_statement = Vec::new();
            // currentについて、DEF(B)になるかどうかを調べ、満たしていれば
            for (i, current) in basic_block.statement_id_list.iter().enumerate() {
                // 現在のインデックス + 1 から最後までを切り出す
                let rest = &basic_block.statement_id_list[i + 1..];

                let current_vreg_statement = self.statement_list.get_vreg_statement_by_id(*current);

                if let Some(current_vreg) = current_vreg_statement.p_def_x {
                    // 現在の文が変数を定義しているとき
                    if rest.iter().any(|j| {
                        let succ_vreg_statement = self.statement_list.get_vreg_statement_by_id(*j);
                        if let Some(succ_vreg) = succ_vreg_statement.p_def_x {
                            current_vreg == succ_vreg
                        } else {
                            false
                        }
                    }) {
                        // 後続する同じブロック内に同じ仮想レジスタを上書きする定義がある
                    } else {
                        // 後続する同じブロック内に同じ仮想レジスタを上書きする定義がない
                        def_statement.push(*current);
                    }
                } else {
                    // 現在の変数は何も定義していない
                }
                // println!("処理中: {}, 後続のデータ: {:?}", current, rest);
            }
            basic_block.vreg_state_set.set_def(def_statement);

            // KILL(B)の設定
            // ブロック内で宣言された変数を上書きするような文の集合
        }

        // Kill(B)の設定
        //
        // 自分のブロック内で定義された変数を上書きするような文の集合
        //
        let statement_id_list = self.list.iter().map(|basic_block| {
            // DEF(B:Block): Set<StatementId>
            let vregs_defined_in_this_blcok = basic_block
                .statement_id_list
                .iter()
                .fold(Vec::<VReg>::new(), |mut acc, statement_id| {
                    let vreg_statement = self.statement_list.get_vreg_statement_by_id(*statement_id);
                    if let Some(defined_vreg) = vreg_statement.p_def_x {
                        acc.push(defined_vreg);
                    }
                    acc
                });

            let kill_statement = self.list
                .iter()
                .fold(Vec::<StatementId>::new(), |mut acc, b| {
                    // 自分を含まないブロックのなかで、同じレジスタが上書きされている場合はその文を追加
                    if basic_block.id != b.id {
                        for other_statement_id in &b.statement_id_list {
                            let vreg_statement = self.statement_list.get_vreg_statement_by_id(*other_statement_id);
                            if let Some(vreg) = vreg_statement.p_def_x {
                                if vregs_defined_in_this_blcok.contains(&vreg) {
                                    acc.push(*other_statement_id);
                                }
                            }
                        }
                    }
                    acc
                });

            kill_statement
        }).collect::<Vec<Vec<StatementId>>>();

        // 所有権の問題で切り分けている
        for (basic_block, statement_id_list) in self.list.iter_mut().zip(statement_id_list) {
            basic_block.vreg_state_set.set_kill(statement_id_list);
        }

        let mut reach_changed = true;

        while reach_changed {
            reach_changed = false;
            let mut new_reaches = Vec::new();

            // new_reach計算フェーズ
            for basic_block in &self.list {
                let a = basic_block.vreg_state_set.derive_new_reach(
                    &basic_block.id, 
                    &self);
                new_reaches.push(a);
            }

            // for i in &new_reaches {
            //     println!("hello world {:?}", i);
            // }

            // 更新フェーズ
            for (basic_block, new_reach) in &mut self.list.iter_mut().zip(new_reaches) {
                if set_neq(&basic_block.vreg_state_set.reach, &new_reach) {
                    basic_block.vreg_state_set.reach = new_reach;
                    reach_changed = true;
                }
            }
        }

        Ok(BasicBlockList {
            list: self.list,
            statement_list: self.statement_list,
            _phantom: PhantomData 
        })
    }
}

impl BasicBlockList<AllSetted> {

    /// reachを導出する
    pub fn derive_reach(mut self) {
    }
}

#[derive(Hash, Copy, Clone, PartialEq, Eq, Debug)]
pub struct BasicBlockId(usize);

impl BasicBlockId {
    pub fn new(id: usize) -> Self {
        Self(id)
    }
}

// =================================================================================
//                         到達する定義、生存区間解析用構造体
// =================================================================================

struct StatementSetting;
struct StatementSetted;

/// 使い方
/// 命令列をみて、VregStatementを`gen_vreg_statement`を使ってセットしていく
pub struct StatementList {
    statements: Vec<VregStatement>, // You can access to this list by statement_id
}

impl StatementList {
    fn gen_vreg_statement(&mut self, p_def_x: Option<VReg>, p_uses_x: Vec<VReg>) -> StatementId {
        let id = StatementId(self.statements.len());
        self.statements.push(VregStatement { id, p_def_x, p_uses_x });
        id
    }

    fn get_vreg_statement_by_id(&self, statement_id: StatementId) -> &VregStatement {
        &self.statements[statement_id.0]
    }
}

#[derive(Clone, Copy)]
struct StatementId(usize);

/// p_def_x: 文pで変数xが定義されているとき,p(def(x))
/// p_uses_x: 文pで変数xが使用されているとき,p(use(x))と書く
pub struct VregStatement {
    id: StatementId,
    p_def_x: Option<VReg>,       // p(def(x))
    p_uses_x: Vec<VReg>, // p(use(x)) なる vregの集合
}

// indexがStatementListと対応します
// 
pub struct VregStateSet {
    reach: Vec<bool>,
    /// DEF(B)の設定
    /// ある変数xについてp(def(x)) ∈ Bで且つpの後にp'(def(x)) ∈ Bなるp'が無い
    /// Bの中で定義され、Bの出口まで有効な文の集合
    def: Vec<bool>,
    /// Kill(B)の設定
    /// 自分のブロック内で定義された変数を上書きするような文の集合
    kill: Vec<bool>,
}

// VregStateSet helper
fn derive_vreg_statement_id_from_bool_list(l: &[bool]) -> Vec<StatementId> {
    l
    .iter()
    .enumerate()
    .fold(Vec::<StatementId>::new(), |mut acc, (index, b)| {
        if *b {
            acc.push(StatementId(index));
        }
        acc
    })
}

fn set_or (a: &[bool], b: &[bool]) -> Vec<bool> {
    a.iter().zip(b).map(|(i, j)| *i || *j).collect()
}

fn set_minus (a: &[bool], b: &[bool]) -> Vec<bool> {
    a.iter().zip(b).map(|(i, j)| *i && !*j).collect()
}

fn set_formula (a: &[bool], b: &[bool], c: &[bool]) -> Vec<bool> {
    set_or(a, &set_minus(b, c))
}

fn set_neq(a: &[bool], b: &[bool]) -> bool {
    a.iter().zip(b).any(|(i, j)| *i ^ *j)
}

impl VregStateSet {
    // StatementListと同じ長さの配列で初期化
    fn new(v: &StatementList) -> Self {
        Self { // すべて空集合として設定する
            reach: vec![false; v.statements.len()], 
            def: vec![false; v.statements.len()],
            kill: vec![false; v.statements.len()],
        }
    }

    /// DEF(B1) = {p1, p2}
    /// の場合
    /// self.def = [false, true, true, false, ...];
    fn set_def(&mut self, statement_id_list: Vec<StatementId>) {
        for i in statement_id_list {
            self.def[i.0] = true;
        }
    }

    fn set_kill(&mut self, statement_id_list: Vec<StatementId>) {
        for i in statement_id_list {
            self.kill[i.0] = true;
        }
    }

    // statement_idのリストを返す

    pub fn get_def_statement_id_list(&self) -> Vec<StatementId> {
        derive_vreg_statement_id_from_bool_list(&self.def)
    }

    pub fn get_kill_statement_id_list(&self) -> Vec<StatementId> {
        derive_vreg_statement_id_from_bool_list(&self.kill)
    }

    fn derive_new_reach (&self, basic_block_id: &BasicBlockId, basic_block_list: &BasicBlockList<IRSetted>) -> Vec<bool> {
        let current_basic_block = basic_block_list.get_basic_block(*basic_block_id);
        let new_reach: Vec<bool> = current_basic_block
            .pred // すべての先行ブロックについて、
            .iter()
            .fold(vec![false; basic_block_list.statement_list.statements.len()], |mut acc,pred_basic_block_id| {
                let pred_block = basic_block_list.get_basic_block(*pred_basic_block_id);
                acc = set_or(&set_formula(
                    &pred_block.vreg_state_set.def,
                    &pred_block.vreg_state_set.reach,
                    &pred_block.vreg_state_set.kill
                ), &acc);
                acc
            });
    
        new_reach
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
    /// 「生存区間」、「到達する定義」のチェック
    pub vreg_state_set: VregStateSet,
    /// このブロック内に含まれる、変数を操作する文
    pub statement_id_list: Vec<StatementId>,
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

// =================================================================================
//                              Instruction及びIR
// =================================================================================

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
    use crate::ir::ir::{BasicBlockList, ConstValue, Instruction, Label, ModuleContext, Operand, Operator};

    #[test]
    fn test00 (){
        let mut mod_ctx = ModuleContext::new();

        let func_id = mod_ctx.create_func(
            "test_func",
            Byte::new(0),
            Byte::new(0x10 * 6));

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
                    true_label: Label("block1".to_string()), 
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
                    println!("    reach {:?}", basic_block.vreg_state_set.reach);
                    println!("    def {:?}", basic_block.vreg_state_set.def);
                    println!("    kill {:?}", basic_block.vreg_state_set.kill);
                }
            } else {
                println!("failed to setting basic_block_list");
            }
        }

    }
}

