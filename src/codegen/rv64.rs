use crate::ir::ir::*;
use crate::unit::size::*;

use crate::codegen::rv64_asm::*;

pub struct NaiveAllocator;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysReg {
    T0/*Reserved*/, T1, T2, T3, T4, T5, T6, // temp reg
    S1, S2, S3, S4, S5, S6, S7, S8, S9, S10, S11, // save reg
    A0, A1, A2, A3, A4, A5, A6, A7, // function args

    Sp,
    Fp,
    Ra, // return addr
}

impl PhysReg {
    pub fn get_reg_name(&self) -> &'static str {
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

            Self::Fp => "fp",
            Self::Sp => "sp",
            Self::Ra => "ra",
        }
    }

    pub fn gen_load_asm(&self, location: &Location) -> Result<Asm, GenAsmErr> {
        match location {
            Location::Register(reg) => {
                // Ok(Asm(format!("mv {}, {}\n", self.get_reg_name(), reg.get_reg_name())))
                Ok(Asm{ statements: vec![AsmStatement::Instruction(RvInst::Mv { rd: *self, rs: *reg })]})
            }
            Location::Stack(other_stack) => {
                if other_stack.size == Size::new(4) {

                    match other_stack.offset {
                        BasePointer::Sp(offset) => 
                            Ok(Asm{ statements: vec![AsmStatement::Instruction(RvInst::Lw { rd: *self, base: PhysReg::Sp, offset })]}),
                        BasePointer::Fp(offset) => 
                            Ok(Asm{ statements: vec![AsmStatement::Instruction(RvInst::Lw { rd: *self, base: PhysReg::Fp, offset })]}),
                    }
                } else if other_stack.size == Size::new(8) {

                    match other_stack.offset {
                        BasePointer::Sp(offset) => 
                            Ok(Asm{ statements: vec![AsmStatement::Instruction(RvInst::Ld { rd: *self, base: PhysReg::Sp, offset })]}),
                        BasePointer::Fp(offset) => 
                            Ok(Asm{ statements: vec![AsmStatement::Instruction(RvInst::Ld { rd: *self, base: PhysReg::Fp, offset })]}),
                    }
                } else {
                    Err(GenAsmErr::UnsupportedByteAlignment)
                }
            }
        }
    }

    pub fn gen_load_immediate(&self, const_val: &ConstValue) -> Asm {
        match const_val {
            ConstValue::I32(a) => 
                Asm { statements: vec![AsmStatement::Instruction(RvInst::Li { rd: *self, imm: *a as i64 })] },
            ConstValue::I64(a) => 
                Asm { statements: vec![AsmStatement::Instruction(RvInst::Li { rd: *self, imm: *a })] },
            ConstValue::U32(a) => 
                Asm { statements: vec![AsmStatement::Instruction(RvInst::Li { rd: *self, imm: *a as i64 })] },
            ConstValue::U64(a) => 
                Asm { statements: vec![AsmStatement::Instruction(RvInst::Li { rd: *self, imm: *a as i64 })] },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhysStack {
    offset: BasePointer,
    size: Byte       // 4 byte or 8 byte
}

impl PhysStack {
    pub fn new(offset: BasePointer, size:Byte) -> Self {
        Self { offset, size }
    }

    pub fn gen_load_asm(&self, location: &Location) -> Result<Asm, GenAsmErr> {
        let (self_base, self_offset) = self.offset.get_base_and_offset();

        match location {
            Location::Stack(other_stack) => {
                let (other_base, other_offset) = other_stack.offset.get_base_and_offset();

                if self.size == Size::new(4) {
                    Ok(
                        Asm { 
                            statements: vec![
                                AsmStatement::Instruction( RvInst::Lw { rd: PhysReg::T0, base: other_base, offset: other_offset },),
                                AsmStatement::Instruction( RvInst::Sw { rs2: PhysReg::T0, base: self_base, offset: self_offset },)
                            ]
                        }
                    )
                } else if self.size == Size::new(8) {
                    Ok(
                        Asm { 
                            statements: vec![
                                AsmStatement::Instruction( RvInst::Ld { rd: PhysReg::T0, base: other_base, offset: other_offset },),
                                AsmStatement::Instruction( RvInst::Sd { rs2: PhysReg::T0, base: self_base, offset: self_offset },)
                            ]
                        }
                    )
                } else {
                    return Err(GenAsmErr::UnsupportedByteAlignment)
                }
            }

            Location::Register(other_reg) => {
                if self.size == Size::new(4) {
                    Ok(
                        Asm { statements: vec![
                            AsmStatement::Instruction( RvInst::Sw { rs2: *other_reg, base: self_base, offset: self_offset })
                        ] }
                    )
                } else if self.size == Size::new(8) {
                    Ok(
                        Asm { statements: vec![
                            AsmStatement::Instruction( RvInst::Sd { rs2: *other_reg, base: self_base, offset: self_offset })
                        ] }
                    )
                } else {
                    return Err(GenAsmErr::UnsupportedByteAlignment)
                }
            }
        }
    }

    pub fn gen_load_immediate (&self, const_val: &ConstValue) -> Asm {
        let (self_base, self_offset) = self.offset.get_base_and_offset();
        match const_val {
            ConstValue::I32(a) => 
                Asm {
                    statements: vec![
                        AsmStatement::Instruction(RvInst::Li { rd: PhysReg::T0, imm: *a as i64 }),
                        AsmStatement::Instruction(RvInst::Sw { rs2: PhysReg::T0, base: self_base, offset: self_offset })
                    ]
                },
            ConstValue::I64(a) => 
                Asm {
                    statements: vec![
                        AsmStatement::Instruction(RvInst::Li { rd: PhysReg::T0, imm: *a as i64 }),
                        AsmStatement::Instruction(RvInst::Sd { rs2: PhysReg::T0, base: self_base, offset: self_offset })
                    ]
                },
            ConstValue::U32(a) => 
                Asm {
                    statements: vec![
                        AsmStatement::Instruction(RvInst::Li { rd: PhysReg::T0, imm: *a as i64 }),
                        AsmStatement::Instruction(RvInst::Sw { rs2: PhysReg::T0, base: self_base, offset: self_offset })
                    ]
                },
            ConstValue::U64(a) => 
                Asm {
                    statements: vec![
                        AsmStatement::Instruction(RvInst::Li { rd: PhysReg::T0, imm: *a as i64 }),
                        AsmStatement::Instruction(RvInst::Sd { rs2: PhysReg::T0, base: self_base, offset: self_offset })
                    ]
                },
        }
    }
}

fn aligned0x10 (size: u64) -> u64 {
    (size + 0xf) & !0xf
}

impl RegisterAllocator for NaiveAllocator {
    fn allocate(
            &mut self,
            _ir: &RvIR,
            arena: &mut VregArena,
            args: &[VReg],
        ) -> Byte {
            let mut current_offset = - 0x10 /* return addr and frame pointer */;
            let mut spillout_offset = 0;

            // arguments
            for (i, &arg_vreg) in args.iter().enumerate() {
                let arg_data = arena.regs.get_mut(arg_vreg.0).unwrap();

                // 引数をスタックメモリに移す
                if i < 8 {
                    current_offset -= 8 /* byte */; // 引数のサイズによらず固定
                    arg_data.location = Some(
                        Location::Stack(
                            PhysStack::new(
                                BasePointer::Fp(current_offset), 
                                arg_data.size.clone()
                            )
                        )
                    );
                } else {
                    // spill out arguments
                    // 呼ばれる側で、どのようにアクセスすればいいか？
                    arg_data.location = Some(
                        Location::Stack(
                            PhysStack::new(
                                BasePointer::Fp(spillout_offset), 
                                arg_data.size.clone()
                            )
                        )
                    );
                    spillout_offset += 8 /* byte */;
                }
            }

            // local valiables
            for vreg_data in arena.regs.iter_mut() {
                if vreg_data.location.is_none() {
                    current_offset -= vreg_data.size.value as i32;
                    vreg_data.location = Some(
                        Location::Stack(
                            PhysStack::new(
                                BasePointer::Fp(current_offset), 
                                vreg_data.size.clone()
                            )
                        )
                    )
                }
            }

            let raw_size = current_offset.abs() as u32;
            Byte::new(aligned0x10(raw_size as u64))
    }
}

///
pub fn gen_funcdef(
    func_def: &mut FuncDef,
    module_context: &Symbols,
    allocator: &mut dyn RegisterAllocator,
) -> Result<Asm, GenAsmErr> {
    let mut asm_statements:Vec<AsmStatement> = Vec::new();
    // prorogue

    // The offset must be in 16-byte units
    //                               |<-- this must be in 16 byte units -->|
    // let raw_size = func_def.arg_size + func_def.local_size + Byte::new(16);
    // let aligned_stack_size = aligned0x10(raw_size.value);
    let aligned_stack_size = allocator.allocate(
        &func_def.ir,
        &mut func_def.vreg_arena,
        &func_def.args).value;

    // set label name

    asm_statements.push(AsmStatement::Label(func_def.name.clone()));

    asm_statements.push(AsmStatement::Instruction(RvInst::Addi { rd: PhysReg::Sp, rs1: PhysReg::Sp, imm: aligned_stack_size  as i32 * -1 }),);        // スタックを伸ばす
    asm_statements.push(AsmStatement::Instruction(RvInst::Sd   { rs2: PhysReg::Fp, base: PhysReg::Sp, offset: (aligned_stack_size - 0x10) as i32 })); // 古いfpをスタックに保存する
    asm_statements.push(AsmStatement::Instruction(RvInst::Addi { rd: PhysReg::Fp, rs1: PhysReg::Sp, imm: aligned_stack_size as i32 }),);              // fpを更新する
    asm_statements.push(AsmStatement::Instruction(RvInst::Sd   { rs2: PhysReg::Ra, base: PhysReg::Fp, offset: -0x8 }));                               // retunr addrをスタックに記録する

    asm_statements.push(
        AsmStatement::Comment("--- save asm register ---".to_string())
    );

    for (i, arg_vreg) in func_def.args.iter().enumerate() {
        let location = &func_def
            .vreg_arena
            .get_vregdata(arg_vreg)
            .unwrap()
            .location
            .unwrap();

        asm_statements = [
            asm_statements,
            match i {
                0 => location.gen_load_asm(&Location::Register(PhysReg::A0)),
                1 => location.gen_load_asm(&Location::Register(PhysReg::A1)),
                2 => location.gen_load_asm(&Location::Register(PhysReg::A2)),
                3 => location.gen_load_asm(&Location::Register(PhysReg::A3)),
                4 => location.gen_load_asm(&Location::Register(PhysReg::A4)),
                5 => location.gen_load_asm(&Location::Register(PhysReg::A5)),
                6 => location.gen_load_asm(&Location::Register(PhysReg::A6)),
                7 => location.gen_load_asm(&Location::Register(PhysReg::A7)),
                8.. => { Ok(Asm{statements: vec![]}) } // スタックメモリに積まれている
            }?.statements
        ].concat();

    }

    asm_statements.push(
        AsmStatement::Comment("--- impl ---".to_string())
    );

    let instructions = &func_def.ir.0;
    for instruction in instructions {

        match instruction {

            Instruction::Call { dest, func, args } => {
                let func_call_asm = gen_funccall(
                    dest, func, args, func_def, &module_context
                )?.statements;
                asm_statements = [asm_statements, func_call_asm].concat();
            }

            Instruction::BinOp { op, dest, lhs, rhs } => {
                let a = gen_binop(op, dest, lhs, rhs, func_def)?.statements;
                asm_statements = [asm_statements, a].concat()
            }

            Instruction::Ret { val } => {
                asm_statements.push(
                    AsmStatement::Comment("--- return ---".to_string())
                );
                if let Some(a) = val {
                    match a {
                        Operand::Const(c) => {
                            let asm = PhysReg::A0.gen_load_immediate(c)
                                .statements;
                            
                            asm_statements = [asm_statements, asm].concat()
                        }
                        Operand::Reg(vreg) => {
                            let location = func_def
                                .vreg_arena
                                .get_vregdata(vreg)
                                .unwrap()
                                .location
                                .unwrap();
                            let asm = PhysReg::A0.gen_load_asm(&location)
                                .unwrap() 
                                .statements;
                            
                            asm_statements = [asm_statements, asm].concat()
                        }
                    }
                }

                asm_statements.push(AsmStatement::Instruction(RvInst::Ld   { rd: PhysReg::Ra, base: PhysReg::Fp, offset: -0x8 }),);                 // メモリからreturn addrを取り出してraにセット
                asm_statements.push(AsmStatement::Instruction(RvInst::Ld   { rd: PhysReg::Fp, base: PhysReg::Fp, offset: -0x10 }));                 // fpを戻す メモリから復元するため、fpは気にしなくても良さそう
                asm_statements.push(AsmStatement::Instruction(RvInst::Addi { rd: PhysReg::Sp, rs1: PhysReg::Sp, imm: aligned_stack_size as i32 }),);// スタックを戻す
                asm_statements.push(AsmStatement::Instruction(RvInst::Ret)); // as same as `jalr zero, 0(ra)`
            }

            Instruction::Branch { cond, true_label, false_label } => {
                let asm = match cond {
                    Operand::Const(c) => {
                        PhysReg::T0.gen_load_immediate(c)
                    }
                    Operand::Reg(vreg) => {
                        let location = func_def
                            .vreg_arena
                            .get_vregdata(vreg)
                            .unwrap()
                            .location
                            .unwrap();
                        PhysReg::T0.gen_load_asm(&location)?
                    }
                };
                asm_statements.push(AsmStatement::Instruction(RvInst::Bnez { rs1: PhysReg::T0, label: true_label.0.clone() }));
                asm_statements.push(AsmStatement::Instruction(RvInst::J { label: false_label.0.clone() }));
            }

            // TODO 未実装のInstruction
            _ => {
            }
        }
    }

    Ok(Asm { statements: asm_statements })
}

// frame pointerを入れないアセンブリを出力する
fn gen_funcdef_without_fp (
    func_def: &mut FuncDef,
    module_context: &Symbols,
    allocator: &mut dyn RegisterAllocator,
) -> Result<Asm, GenAsmErr> {
    todo!()
}

/// generate asm with frame pointer
pub fn gen_funccall (
    dest:&Option<VReg>, func: &Func, args:&Vec<Operand>,
    func_def: &FuncDef,
    module_context: &Symbols,
) -> Result<Asm, GenAsmErr> {
    let mut asm_statements:Vec<AsmStatement> = Vec::new();


    // スピルアウトサイズを計算16byteアラインメントに合わせる
    let spillout_size = if 8 < args.len() {
        let size = args.len() - 8 /* args */;
        aligned0x10(size as u64 * 8) /* byte */ // 引数のサイズが4byteだったとしても、64bit空間内に配置する
    } else {
        0
    };

    // 引数のセット
    // counts of args you want to store on the stack
    for (i, operand) in args.iter().enumerate() {
        asm_statements = [asm_statements, match operand {
            // 引数
            Operand::Const(const_val) => {
                // all args stored under the fp
                match i {
                    0 => PhysReg::A0.gen_load_immediate(const_val),
                    1 => PhysReg::A1.gen_load_immediate(const_val),
                    2 => PhysReg::A2.gen_load_immediate(const_val),
                    3 => PhysReg::A3.gen_load_immediate(const_val),
                    4 => PhysReg::A4.gen_load_immediate(const_val),
                    5 => PhysReg::A5.gen_load_immediate(const_val),
                    6 => PhysReg::A6.gen_load_immediate(const_val),
                    7 => PhysReg::A7.gen_load_immediate(const_val),
                    8.. => 
                        PhysStack::new(BasePointer::Sp(-(spillout_size as i32 - 8 /* byte */ * (i - 8 /*args*/) as i32)), const_val.get_size())
                            .gen_load_immediate(const_val)
                }
            }
            Operand::Reg(vreg) => {
                if let Some(vreg_data) = func_def.vreg_arena.get_vregdata(vreg) {

                    let Some (location) = &vreg_data.location else {return Err(GenAsmErr::VregNotLocated)};
                    // all args stored under the fp
                    match i {
                        0 => PhysReg::A0.gen_load_asm(location),
                        1 => PhysReg::A1.gen_load_asm(location),
                        2 => PhysReg::A2.gen_load_asm(location),
                        3 => PhysReg::A3.gen_load_asm(location),
                        4 => PhysReg::A4.gen_load_asm(location),
                        5 => PhysReg::A5.gen_load_asm(location),
                        6 => PhysReg::A6.gen_load_asm(location),
                        7 => PhysReg::A7.gen_load_asm(location),
                        8.. => 
                            PhysStack::new(BasePointer::Sp(-(spillout_size as i32 - 8 /* byte */ * (i - 8 /*args*/) as i32)), vreg_data.size)
                                .gen_load_asm(location)

                    }?
                } else {
                    return Err(GenAsmErr::VregNotLocated);
                }
            }
        }.statements].concat(); // asm_ins
        
    }

    asm_statements.push(
        AsmStatement::Instruction(RvInst::Addi { rd: PhysReg::Sp, rs1: PhysReg::Sp, imm: -1 * spillout_size as i32 })
    );

    // 関数の呼び出し
    asm_statements.push(
        AsmStatement::Instruction(
            RvInst::Call { symbol: 
            module_context
                .0
                .get(&func.0)
                .expect("name is not set")
                .clone()
        })
    );

    // spを元の状態に戻す
    asm_statements.push(
        AsmStatement::Instruction(RvInst::Addi { rd: PhysReg::Sp, rs1: PhysReg::Sp, imm: spillout_size as i32 })
    );

    // asm_statements
    // return value is in the `a0` register

    // 呼び出しから
    // 返った後の処理
    if let Some (vreg) = dest { // distは無くてもOk
        let vreg_data = func_def.vreg_arena.get_vregdata(vreg).unwrap();

        asm_statements = [
            asm_statements, 
            vreg_data
                .location
                .unwrap()
                .gen_load_asm(&Location::Register(PhysReg::A0))
                .unwrap()
                .statements 
        ]
        .concat();
    } 

    Ok(Asm{ statements: asm_statements })
}

pub fn gen_binop (
    op: &Operator, dest: &VReg, lhs: &Operand, rhs: &Operand,
    func_def: &FuncDef,
) -> Result<Asm, GenAsmErr> {
    let mut statements = Vec::new();

    // 1. LHS を汎用の一時レジスタ(t0)にロードする
    let lhs_asm = match lhs {
        Operand::Const(c) => PhysReg::T0.gen_load_immediate(c),
        Operand::Reg(v) => {
            let loc = func_def
                .vreg_arena
                .get_vregdata(v)
                .unwrap()
                .location
                .as_ref()
                .unwrap();
            PhysReg::T0.gen_load_asm(loc)?
        }
    };
    statements.extend(lhs_asm.statements);

    // 2. RHS を別の一時レジスタ(t1)にロードする
    let rhs_asm = match rhs {
        Operand::Const(c) => PhysReg::T1.gen_load_immediate(c),
        Operand::Reg(v) => {
            let loc = func_def
                .vreg_arena
                .get_vregdata(v)
                .unwrap()
                .location
                .as_ref()
                .unwrap();
            PhysReg::T1.gen_load_asm(loc)?
        }
    };
    statements.extend(rhs_asm.statements);

    // 3. 実際の演算を実行 (t0 = t0 + t1)
    match op {
        Operator::Add => {
            statements.push(AsmStatement::Instruction(RvInst::Add {
                rd: PhysReg::T0,
                rs1: PhysReg::T0,
                rs2: PhysReg::T1,
            }));
        }
        // 将来 Sub や Mul が増えても、ここを1行増やすだけで対応できます！
        _ => return Err(GenAsmErr::NotImplimentedYet),
    }

    // 4. 計算結果(t0)を Dest の Location に書き戻す
    let dest_vreg_data = func_def.vreg_arena.get_vregdata(dest).unwrap();
    let dest_location = dest_vreg_data.location.as_ref().unwrap();

    // 既存の gen_load_asm を「代入」として活用します。
    // (PhysStack 側の実装で、Register を渡すと sw/sd になるように作られているため完璧に動きます)
    let store_asm = match dest_location {
        Location::Stack(stack) => stack.gen_load_asm(&Location::Register(PhysReg::T0))?,
        Location::Register(reg) => reg.gen_load_asm(&Location::Register(PhysReg::T0))?,
    };
    statements.extend(store_asm.statements);

    Ok(Asm { statements })
}

// ============================================================================
// レジスタの生存期間判定
// ============================================================================



/// 
pub fn optimize_test_function (basic_block: &BasicBlock) -> Vec<VRegInfo> {
    let mut check_list = Vec::new();
    let mut vec_vreg_info = Vec::new();

    for (step, inst) in basic_block.insts.iter().enumerate() {
        match inst {
            Instruction::Assign { dest, src:_src } => {                
                if !check_list.contains(&dest) {
                    check_list.push(dest);
                    let life_span = LifeSpan(step, check_life_end(*dest, basic_block));
                    let vreg_info = VRegInfo {
                        v_reg: *dest,
                        crossed_call: check_func_called_in_lifesapn(life_span, basic_block),
                        life_span
                    };
                    vec_vreg_info.push(vreg_info);
                }
            }
            Instruction::Call { dest, func:_func, args:_args } => {
                if let Some(vreg) = dest {
                    if !check_list.contains(&vreg) {
                        check_list.push(vreg);
                        let life_span = LifeSpan(step, check_life_end(*vreg, basic_block));
                        let vreg_info = VRegInfo {
                            v_reg: *vreg,
                            crossed_call: check_func_called_in_lifesapn(life_span, basic_block),
                            life_span
                        };
                        vec_vreg_info.push(vreg_info);
                    }
                } 
            }
            Instruction::BinOp { op:_op, dest, lhs:_lhs, rhs:_rhs } => {
                if !check_list.contains(&dest) {
                    check_list.push(dest);
                    let life_span = LifeSpan(step, check_life_end(*dest, basic_block));
                    let vreg_info = VRegInfo {
                        v_reg: *dest,
                        crossed_call: check_func_called_in_lifesapn(life_span, basic_block),
                        life_span
                    };
                    vec_vreg_info.push(vreg_info);
                }
            }
            _ => {

            }
        }
    }
    vec_vreg_info
}

fn check_life_end (target_vreg: VReg, basic_block: &BasicBlock) -> usize {
    for (i, inst) in basic_block.insts.iter().rev().enumerate() {
        match inst {
            Instruction::Assign { dest: _, src } => {                
                if let Operand::Reg(vreg) = src {
                    if *vreg == target_vreg {
                        return basic_block.insts.len() - i;
                    }
                }
            }
            Instruction::Call { dest:_, func:_, args } => {
                for arg in args {
                    if let Operand::Reg(vreg) = arg {
                        if *vreg == target_vreg {
                            return basic_block.insts.len() - i;
                        }
                    }
                }
            }
            Instruction::BinOp { op:_, dest:_, lhs, rhs } => {
                if let Operand::Reg(vreg) = lhs {
                    if *vreg == target_vreg {
                        return basic_block.insts.len() - i;
                    }
                }
                if let Operand::Reg(vreg) = rhs {
                    if *vreg == target_vreg {
                        return basic_block.insts.len() - i;
                    }
                }
            }
            _ => {
                // skip
            }
        }
    }
    return 0;
}

fn check_func_called_in_lifesapn(life_span: LifeSpan, basic_block: &BasicBlock) -> bool {
    if life_span.0 < life_span.1 { // 普通の場合
        basic_block.insts[life_span.0..life_span.1]
            .iter()
            .any(
                |inst| 
                if let Instruction::Call { dest: _dest, func: _func, args:_args } = inst {
                    true
                } else {
                    false
                }) 
    } else { 
        // 循環している場合
        // 0..LifeSpan.1 LifeSpan.0..=
        basic_block.insts[0..life_span.1]
            .iter()
            .any(
                |inst| 
                if let Instruction::Call { dest: _dest, func: _func, args:_args } = inst {
                    true
                } else {
                    false
                }) ||
        basic_block.insts[life_span.0..]
            .iter()
            .any(
                |inst| 
                if let Instruction::Call { dest: _dest, func: _func, args:_args } = inst {
                    true
                } else {
                    false
                }) 
    }
}

// ================================================================================
//                                       test
// ================================================================================

#[cfg(test)]
mod rv64_codegen_test {
    use std::fmt::format;
    use std::{fs, vec};

    use crate::codegen::rv64::{gen_funcdef, optimize_test_function};
    use crate::ir::ir::{BasicBlock, ConstValue, Dest, Func, FuncDef, Instruction, Label, ModuleContext, Operand, Operator, RvIR};
    use crate::codegen::rv64::NaiveAllocator;
    use crate::unit::size::Byte;
    use crate::codegen::rv64_asm::{
        Asm,
        AsmStatement::Directive,
        Directive::Global
    };

    #[test]
    fn test00 () {
        let mut mod_ctx = ModuleContext::new();

        // other function definitions ...

        let mut args = Vec::new();

        let mut func = FuncDef::new(
            "test_func",
            Byte::new(16),
            Byte::new(16));

        let arg1_reg = func.vreg_arena.alloc(Byte::new(8), Some(String::from("arg1")));
        args.push(arg1_reg);
        let arg2_reg = func.vreg_arena.alloc(Byte::new(8), Some(String::from("arg2")));
        args.push(arg2_reg);

        let tmp_reg = func.vreg_arena.alloc(Byte::new(8), Some(String::from("tmp")));

        func.set_ir(RvIR(vec![
            Instruction::BinOp { 
                op: Operator::Add ,
                dest: tmp_reg,
                lhs: Operand::Reg(arg1_reg),
                rhs: Operand::Reg(arg2_reg),
            },
            Instruction::Ret { val: Some(Operand::Reg(tmp_reg)) }
        ]));

        let asm = gen_funcdef(
            &mut func,
            &mod_ctx.symbols,
            &mut NaiveAllocator
        );
        let id = mod_ctx.add_func(func);

    }

    #[test]
    fn test01 () {
        let mut mod_ctx = ModuleContext::new();

        let func_id = mod_ctx.create_func(
            "test_func",
            Byte::new(16),
            Byte::new(16));

        {
            let func = mod_ctx.get_func_mut(func_id).unwrap();

            let mut args = Vec::new();

            let arg1_reg = func.vreg_arena.alloc(Byte::new(8), Some(String::from("arg1")));
            args.push(arg1_reg);
            let arg2_reg = func.vreg_arena.alloc(Byte::new(8), Some(String::from("arg2")));
            args.push(arg2_reg);

            let tmp_reg = func.vreg_arena.alloc(Byte::new(8), Some(String::from("tmp")));

            func.set_args(args);
            func.set_ir(RvIR(vec![
                Instruction::BinOp { 
                    op: Operator::Add ,
                    dest: tmp_reg,
                    lhs: Operand::Reg(arg1_reg),
                    rhs: Operand::Reg(arg2_reg),
                },
                Instruction::Ret { val: Some(Operand::Reg(tmp_reg)) }
            ]));
        }

        let mut asm_statements = Vec::new();

        asm_statements.push(Directive(Global("test_func".to_string())));
        for i in mod_ctx.funcs.iter_mut() {
            asm_statements = [
                asm_statements,
                match gen_funcdef(
                    i,
                    &mod_ctx.symbols,
                    &mut NaiveAllocator
                ) {
                    Ok(asm) => {
                        asm.statements
                    }
                    Err(e) => {
                        println!("error occured!");
                        println!("{:?}", e);
                        return ;
                    }
                }
            ].concat();
        }

        let asm = Asm {
            statements: asm_statements
        };

        std::fs::write("test_program.S", &format!("{}", asm)).unwrap();
    }

    #[test]
    fn test02 () {
        let mut mod_ctx = ModuleContext::new();

        let func_id1 = mod_ctx.create_func(
            "arg10",
            Byte::new(16),
            Byte::new(16));

        let func_id2 = mod_ctx.create_func(
            "test_func",
            Byte::new(80),
            Byte::new(8));

        // 渡されたすべての引数を足す
        {
            let func = mod_ctx.get_func_mut(func_id1).unwrap();

            let mut args = Vec::new();

            let arg1_reg = func.vreg_arena.alloc(Byte::new(8), Some(String::from("arg1")));
            let arg2_reg = func.vreg_arena.alloc(Byte::new(8), Some(String::from("arg2")));
            let arg3_reg = func.vreg_arena.alloc(Byte::new(8), Some(String::from("arg3")));
            let arg4_reg = func.vreg_arena.alloc(Byte::new(8), Some(String::from("arg4")));
            let arg5_reg = func.vreg_arena.alloc(Byte::new(8), Some(String::from("arg5")));
            let arg6_reg = func.vreg_arena.alloc(Byte::new(8), Some(String::from("arg6")));
            let arg7_reg = func.vreg_arena.alloc(Byte::new(8), Some(String::from("arg7")));
            let arg8_reg = func.vreg_arena.alloc(Byte::new(8), Some(String::from("arg8")));
            let arg9_reg = func.vreg_arena.alloc(Byte::new(8), Some(String::from("arg9")));
            let arg10_reg = func.vreg_arena.alloc(Byte::new(8), Some(String::from("arg10")));
            args.push(arg1_reg);
            args.push(arg2_reg);
            args.push(arg3_reg);
            args.push(arg4_reg);
            args.push(arg5_reg);
            args.push(arg6_reg);
            args.push(arg7_reg);
            args.push(arg8_reg);
            args.push(arg9_reg);
            args.push(arg10_reg);

            let tmp_reg = func.vreg_arena.alloc(Byte::new(8), Some(String::from("tmp")));

            func.set_args(args);
            func.set_ir(RvIR(vec![
                Instruction::BinOp { op: Operator::Add , dest: tmp_reg, lhs: Operand::Const(crate::ir::ir::ConstValue::I64(0)), rhs: Operand::Reg(arg1_reg)},
                Instruction::BinOp { op: Operator::Add, dest: tmp_reg, lhs: Operand::Reg(tmp_reg), rhs: Operand::Reg(arg2_reg), },
                Instruction::BinOp { op: Operator::Add, dest: tmp_reg, lhs: Operand::Reg(tmp_reg), rhs: Operand::Reg(arg3_reg), },
                Instruction::BinOp { op: Operator::Add, dest: tmp_reg, lhs: Operand::Reg(tmp_reg), rhs: Operand::Reg(arg4_reg), },
                Instruction::BinOp { op: Operator::Add, dest: tmp_reg, lhs: Operand::Reg(tmp_reg), rhs: Operand::Reg(arg5_reg), },
                Instruction::BinOp { op: Operator::Add, dest: tmp_reg, lhs: Operand::Reg(tmp_reg), rhs: Operand::Reg(arg6_reg), },
                Instruction::BinOp { op: Operator::Add, dest: tmp_reg, lhs: Operand::Reg(tmp_reg), rhs: Operand::Reg(arg7_reg), },
                Instruction::BinOp { op: Operator::Add, dest: tmp_reg, lhs: Operand::Reg(tmp_reg), rhs: Operand::Reg(arg8_reg), },
                Instruction::BinOp { op: Operator::Add, dest: tmp_reg, lhs: Operand::Reg(tmp_reg), rhs: Operand::Reg(arg9_reg), },
                Instruction::BinOp { op: Operator::Add, dest: tmp_reg, lhs: Operand::Reg(tmp_reg), rhs: Operand::Reg(arg10_reg), },

                Instruction::Ret { val: Some(Operand::Reg(tmp_reg)) }
            ]));
        }

        {
            let func = mod_ctx.get_func_mut(func_id2).unwrap();

            let args = Vec::new();

            let tmp_reg = func.vreg_arena.alloc(Byte::new(8), Some(String::from("tmp")));

            func.set_args(args);
            func.set_ir(RvIR(vec![
                Instruction::Call { dest: Some(tmp_reg), func: Func(func_id1), args: vec![
                    Operand::Const(crate::ir::ir::ConstValue::I64(1)),
                    Operand::Const(crate::ir::ir::ConstValue::I64(2)),
                    Operand::Const(crate::ir::ir::ConstValue::I64(3)),
                    Operand::Const(crate::ir::ir::ConstValue::I64(4)),
                    Operand::Const(crate::ir::ir::ConstValue::I64(5)),
                    Operand::Const(crate::ir::ir::ConstValue::I64(6)),
                    Operand::Const(crate::ir::ir::ConstValue::I64(7)),
                    Operand::Const(crate::ir::ir::ConstValue::I64(8)),
                    Operand::Const(crate::ir::ir::ConstValue::I64(9)),
                    Operand::Const(crate::ir::ir::ConstValue::I64(10)),
                ] },
                Instruction::Ret { val: Some(Operand::Reg(tmp_reg)) }
            ]));
        }

        let mut asm_statements = Vec::new();

        asm_statements.push(Directive(Global("test_func".to_string())));

        for i in mod_ctx.funcs.iter_mut() {

            asm_statements = [
                asm_statements,
                match gen_funcdef(
                    i,
                    &mod_ctx.symbols,
                    &mut NaiveAllocator
                ) {
                    Ok(asm) => {
                        asm.statements
                    }
                    Err(e) => {
                        println!("error occured!");
                        println!("{:?}", e);
                        return ;
                    }
                }
            ].concat();
        }

        let asm = Asm {
            statements: asm_statements
        };

        std::fs::write("test_program.S", &format!("{}", asm)).unwrap();
        println!("DONE!");
    }

    #[test]
    fn test03() {
        let mut mod_ctx = ModuleContext::new();

        let func_id = mod_ctx.create_func(
            "test_func",
            Byte::new(16),
            Byte::new(16));

        {
            let func = mod_ctx.get_func_mut(func_id).unwrap();

            let mut args = Vec::new();

            let arg1_reg = func.vreg_arena.alloc(Byte::new(8), Some(String::from("arg1")));
            args.push(arg1_reg);
            let arg2_reg = func.vreg_arena.alloc(Byte::new(8), Some(String::from("arg2")));
            args.push(arg2_reg);

            let tmp_reg = func.vreg_arena.alloc(Byte::new(8), Some(String::from("tmp")));

            func.set_args(args);

            let basic_block_if_cond = BasicBlock {
                label: Label("hello".to_string()),
                insts: vec![
                    Instruction::Branch {
                        cond: Operand::Reg(tmp_reg),
                        true_label: Label("true_label".to_string()),
                        false_label: Label("false_label".to_string()),
                    },
                ]
            };

            let basic_block_true_label = BasicBlock {
                label: Label("true_label".to_string()),
                insts: vec![
                    Instruction::Ret { val: Some(Operand::Const(ConstValue::I64(0xdeadbeaf))) }
                ]
            };

            let basic_block_false_label = BasicBlock {
                label: Label("false_label".to_string()),
                insts: vec![
                    Instruction::Ret { val: Some(Operand::Const(ConstValue::I64(0xBADC0DE))) }
                ]
            };

            func.set_ir(RvIR(vec![
                Instruction::Branch {
                    cond: Operand::Reg(tmp_reg),
                    true_label: Label("true_label".to_string()),
                    false_label: Label("false_label".to_string()),
                },
                Instruction::Ret { val: Some(Operand::Reg(tmp_reg)) }
            ]));
        }

        let mut asm_statements = Vec::new();

        asm_statements.push(Directive(Global("test_func".to_string())));
        for i in mod_ctx.funcs.iter_mut() {
            asm_statements = [
                asm_statements,
                match gen_funcdef(
                    i,
                    &mod_ctx.symbols,
                    &mut NaiveAllocator
                ) {
                    Ok(asm) => {
                        asm.statements
                    }
                    Err(e) => {
                        println!("error occured!");
                        println!("{:?}", e);
                        return ;
                    }
                }
            ].concat();
        }

        let asm = Asm {
            statements: asm_statements
        };

        std::fs::write("test_program.S", &format!("{}", asm)).unwrap();
    }

    #[test]
    fn test04 () {
        let mut mod_ctx = ModuleContext::new();

        let func_id = mod_ctx.create_func(
            "test_func",
            Byte::new(16),
            Byte::new(16));

        {

            let func = mod_ctx.get_func_mut(func_id).unwrap();
            let tmp_reg_a = func.vreg_arena.alloc(Byte::new(8), Some(String::from("tmp_a")));
            let tmp_reg_b = func.vreg_arena.alloc(Byte::new(8), Some(String::from("tmp_b")));
            let tmp_reg_c = func.vreg_arena.alloc(Byte::new(8), Some(String::from("tmp_c")));
            let tmp_reg_d = func.vreg_arena.alloc(Byte::new(8), Some(String::from("tmp_d")));
            let tmp_reg_e = func.vreg_arena.alloc(Byte::new(8), Some(String::from("tmp_e")));

            // basic block
            let basic_block = BasicBlock{
                label: Label("inloop".to_string()),
                insts: vec![ 
                    Instruction::BinOp { op: Operator::Add, dest: tmp_reg_a, lhs: Operand::Reg(tmp_reg_d), rhs: Operand::Const(ConstValue::U64(2)) },
                    Instruction::BinOp { op: Operator::Add, dest: tmp_reg_b, lhs: Operand::Reg(tmp_reg_a), rhs: Operand::Reg(tmp_reg_e) },
                    Instruction::BinOp { op: Operator::Add, dest: tmp_reg_c, lhs: Operand::Reg(tmp_reg_a), rhs: Operand::Const(ConstValue::U64(3)) },
                    Instruction::BinOp { op: Operator::Add, dest: tmp_reg_d, lhs: Operand::Reg(tmp_reg_b), rhs: Operand::Reg(tmp_reg_c) },
                    Instruction::BinOp { op: Operator::Add, dest: tmp_reg_e, lhs: Operand::Reg(tmp_reg_d), rhs: Operand::Const(ConstValue::U64(5)) },
                ]
            };

            let basic_block_infomations = optimize_test_function(&basic_block);

            for i in basic_block_infomations {
                println!("{:?}", i);
            }
        }
    }
}
