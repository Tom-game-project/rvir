use std::collections::HashMap;
use std::marker::PhantomData;

use crate::unit::size::Byte;

// vertual register
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VReg(pub usize);

pub trait RegisterAllocator<R, S> {
    /// Analyze the IR and argument information, and assign a Location to every VReg in VregArena.
    /// It returns the final 16-byte-aligned stack size.
    fn allocate(
        &mut self,
        ir: &RvIR,
        arena: &mut VregArena<R, S>,
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
pub enum Location<R, S> {
    Register(R),
    Stack(S), // frame offset
                      // fp base
}

/// あるデータを、自身にロードする命令の機械語を出力する
pub trait GenLoadAsm<R, S, A /*Asm*/, E> {
    fn gen_load_asm(&self, location: &Location<R, S>) -> Result<A, E>;
}

pub struct VRegData<R, V> {
    pub v_reg: VReg,
    pub name: String,
    pub size: Byte,
    // pub location: Option<IrLocation> // TODO
    pub location: Option<Location<R, V>>
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

pub struct VregArena<R, V> {
    pub regs: Vec<VRegData<R, V>>
}

impl<R, V> VregArena<R, V> {

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

    pub fn get_vregdata(&self, vreg: &VReg) -> Option<&VRegData<R, V>> {
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


pub struct Setting;
/// IRの設定が終了
pub struct IRSetted;
pub struct AllSetted;

pub struct BasicBlockList<State> {
    pub list: Vec<BasicBlock>,
    pub entry_basic_block_id: BasicBlockId,
    pub statement_list: StatementList,
    _phantom: PhantomData<State>
}

impl BasicBlockList<Setting> {
    pub fn new() -> Self {
        Self { 
            list: Vec::new(),
            entry_basic_block_id: BasicBlockId(0),
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
                dom: Vec::new(),
                idom: None,
                vreg_state_set: VregStateSet { 
                    reach: Vec::new(),
                    live: Vec::new(),
                    def: Vec::new(),
                    kill: Vec::new(),
                    use_: Vec::new(),
                    kill_dash: Vec::new()
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

    pub fn set_entry_block(&mut self, basic_block_id: BasicBlockId) {
        self.entry_basic_block_id = basic_block_id;
    }

    /// irの設定が終了したら、状態を移す
    pub fn finish_ir_setting (self) -> BasicBlockList<IRSetted> {
        BasicBlockList { 
            list: self.list,
            entry_basic_block_id: self.entry_basic_block_id,
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

/// 命令列に基づいて、文の集合のuse(x)やdef(x)を定義する

pub trait BlockListState {}
impl BlockListState for IRSetted {}
impl BlockListState for AllSetted {}

impl<T: BlockListState> BasicBlockList<T> {
    pub fn get_basic_block(&self, id: BasicBlockId) -> &BasicBlock {
        &self.list[id.0]
    }

    pub fn get_basic_block_by_label(&self, label: &Label) -> Option<&BasicBlock> {
        self.list.iter().find(|b | b.label == *label)
    }
}

/// IRの設定が終わったときに存在するメソッド
impl BasicBlockList<IRSetted> {

    // vreg_state_set setup functions

    /// def, kill,の初期化
    /// DEF(B)の設定
    /// ある変数xについてp(def(x)) ∈ Bで且つpの後にp'(def(x)) ∈ Bなるp'が無い
    /// Bの中で定義され、Bの出口まで有効な文の集合
    fn setup_def_set(&mut self) {
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
    }

    // Kill(B)の設定
    // 自分のブロック内で定義された変数を上書きするような文の集合
    //
    fn setup_kill_set(&mut self) {
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
    }

    fn setup_use_set(&mut self) {
        for basic_block in &mut self.list {
            // [p0, p1, p2, ...]
            let statement_id_list = basic_block.statement_id_list.iter().enumerate().fold(Vec::<StatementId>::new(),|mut acc, (index, statement_id)| 
                {
                    let statement = self.statement_list.get_vreg_statement_by_id(*statement_id);
                    if basic_block.statement_id_list[0..index]
                        .iter()
                        .all(|pre_statement_id|{
                        let pre_statement = self.statement_list.get_vreg_statement_by_id(*pre_statement_id);
                        if let Some (def_vreg) = pre_statement.p_def_x {
                            // pre_statementに上書きされていなければOk
                            !statement.p_uses_x.contains(&def_vreg)
                        } else {
                            // 何もない場合は、「xの定義はない」に該当するため
                            true
                        }
                    }) /*もし、自分の使っている変数がブロックの開始から現在の文まで上書きされていなければ*/ {
                        acc.push(*statement_id);
                    }
                    acc
                });

            basic_block.vreg_state_set.set_use(statement_id_list);
        }
    }

    fn setup_kill_dash_set(&mut self) {
        for basic_block in &mut self.list {
            let statement_id_list = basic_block
                .statement_id_list
                .iter()
                .filter(|&&statement_id| {
                    let statement = self.statement_list.get_vreg_statement_by_id(statement_id);
                    statement.p_def_x.is_some()
                })
                .copied()
                .collect::<Vec<StatementId>>();
            basic_block.vreg_state_set.set_kill_dash(statement_id_list);
        }
    }

    /// reachを求めるためのループ
    /// 変化がなくなるまでループさせ続ける
    fn setup_reach_set(&mut self) {
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

            // 更新フェーズ
            for (basic_block, new_reach) in &mut self.list.iter_mut().zip(new_reaches) {
                if set_neq(&basic_block.vreg_state_set.reach, &new_reach) {
                    basic_block.vreg_state_set.reach = new_reach;
                    reach_changed = true;
                }
            }
        }
    }

    /// liveを求めるためのループ
    /// 変化がなくなるまでループさせ続ける
    fn setup_live_set(&mut self) {
        let mut live_changed = true;

        while live_changed {
            live_changed = false;
            let mut new_live = Vec::new();

            // new_live計算フェーズ
            for basic_block in &self.list {
                let a = basic_block.vreg_state_set.derive_new_live(
                    &basic_block.id, 
                    &self);
                new_live.push(a);
            }

            // 更新フェーズ
            for (basic_block, new_live) in &mut self.list.iter_mut().zip(new_live) {
                if set_neq(&basic_block.vreg_state_set.live, &new_live) {
                    basic_block.vreg_state_set.live = new_live;
                    live_changed = true;
                }
            }
        }

    }

    /// すべてのbasic blockの支配ブロックを計算する
    fn setup_dom(&mut self) {
        // basic_blockを格納するリストの長さ
        let length = self.list.len();

        // 初期設定
        for basic_block in &mut self.list {
            if basic_block.id == self.entry_basic_block_id {
                // DOM(B0) = {B0}
                // プログラムの先頭だったとき
                basic_block.dom = basic_block.only_contain_my_self(length);
            } else {
                // DOM(B) = { ブロック全体 } (B not eq B0)
                basic_block.dom = vec![true; length];
            }
        }

        let mut dom_changed = true;
        while dom_changed {
            dom_changed = false;
            let mut new_doms = Vec::new();

            for basic_block in &self.list {
                if basic_block.id == self.entry_basic_block_id {
                    new_doms.push(basic_block.only_contain_my_self(length));
                    continue;
                }

                new_doms.push(
                    set_or(
                        &basic_block.only_contain_my_self(length),
                        &basic_block
                            .pred
                            .iter()
                            .fold(
                                vec![true; length],
                                |mut acc, pred_basic_block_id| {
                            let pred_basic_block = self.get_basic_block(*pred_basic_block_id);
                            acc = set_and(&acc, &pred_basic_block.dom);
                            acc
                        })
                    )
                );
            }

            for (basic_block, new_dom) in self.list.iter_mut().zip(new_doms) {
                if set_neq(&basic_block.dom, &new_dom) {
                    basic_block.dom = new_dom;
                    dom_changed = true;
                }
            }
        }
    }

    fn setup_idom(&mut self) {
        let mut idoms = Vec::new();

        for i in &self.list {
            idoms.push(i.get_immediately_dominate(&self));
        }

        for (basic_block, idom) in self.list.iter_mut().zip(idoms) {
            basic_block.idom = idom;
        }
    }

    /// できればBasicBlockListの状態を変えてirを再設定できないようにする
    ///
    /// ir設定後　呼び出されることを想定する
    ///
    /// BasicBlock(.pred, .succ)を設定
    /// vreg_state_set:VregStateSetの各種パラメータの設定をする
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
                if let Some(statement_id) = self.statement_list.generate_and_set_statement_ids(j) { /* 発行されたstatement_idに対応付けられたデータはここに保存される */
                    basic_block.statement_id_list.push(statement_id);
                }
            }
        }

        // self.statement_listですべてのブロック内に存在する文の収集が完了する

        self.setup_def_set();
        self.setup_kill_set();
        self.setup_reach_set();

        self.setup_use_set();
        self.setup_kill_dash_set();
        self.setup_live_set();

        // 支配関係の導出
        self.setup_dom();
        self.setup_idom();

        Ok(BasicBlockList {
            list: self.list,
            entry_basic_block_id: self.entry_basic_block_id,
            statement_list: self.statement_list,
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

    #[cfg(debug_assertions)]
    pub fn get_basic_block_name(&self, basic_block_list: &BasicBlockList<AllSetted>) -> Label {
        basic_block_list.list[self.0].label.clone()
    }
}

// =================================================================================
//                         到達する定義、生存区間解析用構造体
// =================================================================================

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

    /// 命令をみて、変数の定義、使用をするようなものを文として解釈し、statement_idを割与える
    fn generate_and_set_statement_ids(&mut self, j: &Instruction) -> Option<StatementId> {
        match j { 
            Instruction::Call { dest, func:_func, args } => {
                let statement_id = self
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
                let statement_id = self
                    .gen_vreg_statement(
                        Some(*dest),
                        p_uses_x);
                Some(statement_id)
            }

            Instruction::Assign { dest, src } => {
                let statement_id = self
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
}

#[derive(Debug, Clone, Copy)]
pub struct StatementId(usize);

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
    live: Vec<bool>,
    /// DEF(B)の設定
    /// ある変数xについてp(def(x)) ∈ Bで且つpの後にp'(def(x)) ∈ Bなるp'が無い
    /// Bの中で定義され、Bの出口まで有効な文の集合
    def: Vec<bool>,
    /// Kill(B)の設定
    /// 自分のブロック内で定義された変数を上書きするような文の集合
    kill: Vec<bool>,
    /// USE(B)
    /// p(use(x)) ∈ Bで、且つBの入口からpまでの間にxの定義がない
    use_: Vec<bool>,
    /// KILL'(B)
    /// p(def(x)) ∈ なるpがある 
    kill_dash: Vec<bool>,
}

// VregStateSet helper functions

// Vec<bool>形式の集合を扱いやすくするために導入した表現をStatementIdに直す
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

fn set_and (a: &[bool], b: &[bool]) -> Vec<bool> {
    a.iter().zip(b).map(|(i, j)| *i && *j).collect()
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
            live: vec![false; v.statements.len()], 
            def: vec![false; v.statements.len()],
            kill: vec![false; v.statements.len()],
            use_: vec![false; v.statements.len()],
            kill_dash: vec![false; v.statements.len()],
        }
    }

    // setting functions 

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

    fn set_use(&mut self, statement_id_list: Vec<StatementId>) {
        for i in statement_id_list {
            self.use_[i.0] = true;
        }
    }

    fn set_kill_dash(&mut self, statement_id_list: Vec<StatementId>) {
        for i in statement_id_list {
            self.kill_dash[i.0] = true;
        }
    }

    // statement_idのリストを返す

    pub fn get_def_statement_id_list(&self) -> Vec<StatementId> {
        derive_vreg_statement_id_from_bool_list(&self.def)
    }

    pub fn get_kill_statement_id_list(&self) -> Vec<StatementId> {
        derive_vreg_statement_id_from_bool_list(&self.kill)
    }

    pub fn get_reach_statement_id_list(&self) -> Vec<StatementId> {
        derive_vreg_statement_id_from_bool_list(&self.reach)
    }

    pub fn get_live_statement_id_list(&self) -> Vec<StatementId> {
        derive_vreg_statement_id_from_bool_list(&self.live)
        
    }

    /// reachのcore logic
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

    /// liveのcore logic
    fn derive_new_live(&self, basic_block_id: &BasicBlockId, basic_block_list: &BasicBlockList<IRSetted>) -> Vec<bool> {
        let current_basic_block = basic_block_list.get_basic_block(*basic_block_id);
        let new_live = set_or(
            &current_basic_block.vreg_state_set.use_, 
            &set_minus(
                &current_basic_block
                    .succ
                    .iter()
                    .fold(vec![false; basic_block_list.statement_list.statements.len()], |mut acc, pred_basic_block_id| {
                        let pred_block = basic_block_list.get_basic_block(*pred_basic_block_id);
                        acc = set_or(&pred_block.vreg_state_set.live, &acc);
                        acc }), 
                &current_basic_block.vreg_state_set.kill_dash));

        new_live
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
    pub vreg_state_set: VregStateSet, // 文の集合
    // 自分を支配するブロックの集合
    pub dom: Vec<bool>, // BasicBlockList.listのindexに対応
    pub idom: Option<BasicBlockId>,
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

    fn only_contain_my_self(&self, length_of_basic_block_list: usize) -> Vec<bool>{
        let mut b = vec![false; length_of_basic_block_list];
        b[self.id.0] = true;
        b
    }

    /// 自分の支配ブロックを出力する
    pub fn get_dom_basic_block_ids(&self) -> Vec<BasicBlockId> {
        self.dom.iter().enumerate().fold(Vec::<BasicBlockId>::new(), |mut acc, (id, b)| {
            if *b {
                acc.push(BasicBlockId(id))
            }
            acc
        })
    }

    /// selfが引数を厳密に支配しているかどうかを判定するメソッド
    pub fn is_strictly_dominate(&self, basic_block_id: BasicBlockId) -> bool {
        self.get_dom_basic_block_ids().contains(&basic_block_id) && self.id != basic_block_id
    }

    // selfが、与えられたbasic_block_idを支配するかどうか調べる
    pub fn is_ancestor(&self, basic_block_list: &BasicBlockList<IRSetted>, basic_block_id: BasicBlockId) -> bool {
        let basic_block = basic_block_list.get_basic_block(basic_block_id);

        basic_block.get_dom_basic_block_ids()
            .iter()
            .filter(|br_id| {
                // let block = basic_block_list.get_basic_block(**br_id);
                // block.is_strictly_dominate(self.id)
                **br_id != basic_block_id
            })
            .map(|br_id| *br_id)
            .collect::<Vec<BasicBlockId>>()
            .contains(&self.id)
    }

    /// 直接支配(IDOM)を探す
    pub fn get_immediately_dominate(&self, basic_block_list: &BasicBlockList<IRSetted>) -> Option<BasicBlockId> {
        // 自分を支配するブロックの集合
        let dom_myself: Vec<BasicBlockId> = self
            .get_dom_basic_block_ids()
            .iter()
            .filter(|br_id| {
                // let block = basic_block_list.get_basic_block(**br_id);
                // block.is_strictly_dominate(self.id)
                **br_id != self.id
            })
            .map(|br_id| *br_id)
            .collect();

        dom_myself
            .iter()
            .copied()
            .find(|b| {
                let block = basic_block_list.get_basic_block(*b);

                !dom_myself
                    .iter()
                    .filter(|br| *br != b)
                    .any(|br| {
                        block.is_ancestor(&basic_block_list, *br)
                    })
            })

        // println!("自分のid {:?}", self.id);
        // println!("自身を支配するブロックたち  {:?}", dom_myself);
        // println!("idom  {:?}\n", idom);

        // 自分を支配するブロックが、
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
pub struct FuncDef<R, V> {
    pub name: String, // name of function
    func_id: FuncId,
    pub args: Vec<VReg>,
    pub arg_size: Byte,
    pub local_size: Byte, // local value size

    pub vreg_arena: VregArena<R, V>,
    pub ir: RvIR,
}

impl<R, V> FuncDef<R, V> {
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
pub struct ModuleContext<R, S> {
    pub symbols: Symbols,
    pub funcs: Vec<FuncDef<R, S>>
}

impl<R, S> ModuleContext<R, S> { 

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

    pub fn add_func(&mut self, mut func_def: FuncDef<R, S>) -> FuncId {
        let func_id = FuncId(self.funcs.len());
        func_def.func_id = func_id;
        self.symbols.0.insert(func_id, func_def.name.clone());
        self.funcs.push(
            func_def
        );
        func_id
    }

    pub fn get_func_mut(&mut self, id: FuncId) -> Option<&mut FuncDef<R, S>> {
        self.funcs.get_mut(id.0)
    }

    pub fn get_func(&self, id: FuncId) -> Option<&FuncDef<R, S>> {
        self.funcs.get(id.0)
    }
}

