use std::vec;

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

    /// 
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

            // arguments
            for (i, &arg_vreg) in args.iter().enumerate() {
                if i < 8 {
                    let arg_data = arena.regs.get_mut(arg_vreg.0).unwrap();
                    current_offset -= arg_data.size.value as i32;
                    arg_data.location = Some(
                        Location::Stack(
                            PhysStack::new(
                                    BasePointer::Fp(current_offset), 
                                    arg_data.size.clone()
                                )
                            )
                        )
                } else {
                    // spill out arguments
                    todo!() // TODO まだスイルアウト分は実装していない
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

    asm_statements.push(AsmStatement::Instruction(RvInst::Addi { rd: PhysReg::Sp, rs1: PhysReg::Sp, imm: aligned_stack_size  as i32 * -1 }),);
    asm_statements.push(AsmStatement::Instruction(RvInst::Sd   { rs2: PhysReg::Fp, base: PhysReg::Sp, offset: (aligned_stack_size - 0x10) as i32 }));
    asm_statements.push(AsmStatement::Instruction(RvInst::Addi { rd: PhysReg::Fp, rs1: PhysReg::Sp, imm: aligned_stack_size as i32 }),);
    asm_statements.push(AsmStatement::Instruction(RvInst::Sd   { rs2: PhysReg::Ra, base: PhysReg::Fp, offset: -0x8 }));

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
                // TODO
            }
            _ => {

            }
        }
    }

    asm_statements.push(AsmStatement::Instruction(RvInst::Ld   { rd: PhysReg::Ra, base: PhysReg::Fp, offset: -0x8 }),);
    asm_statements.push(AsmStatement::Instruction(RvInst::Ld   { rd: PhysReg::Fp, base: PhysReg::Fp, offset: -0x10 }));
    asm_statements.push(AsmStatement::Instruction(RvInst::Addi { rd: PhysReg::Sp, rs1: PhysReg::Sp, imm: aligned_stack_size as i32 }),);
    asm_statements.push(AsmStatement::Instruction(RvInst::Ret  )); // as same as `jalr zero, 0(ra)`
    
    Ok(Asm { statements: asm_statements })
}

pub fn gen_funccall (
    dest:&Option<VReg>, func: &Func, args:&Vec<Operand>,
    func_def: &FuncDef,
    module_context: &Symbols,
) -> Result<Asm, GenAsmErr> {
    let mut asm_statements:Vec<AsmStatement> = Vec::new();

    // 引数のセット
    // counts of args you want to store on the stack
    for (i, operand) in args.iter().enumerate() {
        asm_statements = [asm_statements, match operand {
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
                        PhysStack::new(BasePointer::Sp(8 /* byte */ * i as i32), const_val.get_size())
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
                            PhysStack::new(BasePointer::Sp(8 /* byte */ * i as i32), vreg_data.size)
                                .gen_load_asm(location)
                        
                    }?
                } else {
                    return Err(GenAsmErr::VregNotLocated);
                }
            }
        }.statements].concat(); // asm_ins
        
    }

    // 関数の呼び出し
    asm_statements.push(
        AsmStatement::Instruction(RvInst::Call { symbol: 
            module_context
                .0
                .get(&func.0)
                .expect("name is not set")
                .clone()
                 // .get_func(func.0)
                 // .expect("name is not set")
                 // .name
                 // .clone()
        })
    );

    // asm_statements
    // return value is in the `a0` register

    if let Some (vreg) = dest {
        if let Some(vreg_data) = func_def.vreg_arena.get_vregdata(vreg) {
            
        } else {
            // ERROR
        }
        // asm.push_str(format!("addi a0, {}, 0", ));
    } else {
        // ERROR
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


#[cfg(test)]
mod rv64_codegen_test {
    use crate::codegen::rv64::gen_funcdef;
    use crate::ir::ir::{FuncDef, Instruction, ModuleContext, Operand, Operator, RvIR};
    use crate::codegen::rv64::NaiveAllocator;
    use crate::unit::size::Byte;


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

        // let asm = codegen(&mod_ctx);
        // println!("{}", asm);
    }

    #[test]
    fn test01 () {
        // 

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

        for i in mod_ctx.funcs.iter_mut() {
            match gen_funcdef(
                i,
                &mod_ctx.symbols,
                &mut NaiveAllocator
            ) {
                Ok(asm) => {
                    println!("asm generated!");
                    println!("{}", asm);
                }
                Err(e) => {
                    println!("error occured!");
                    println!("{:?}", e);
                }
            }
        }
    }
}
