use crate::ir::ir::*;
use crate::unit::size::*;

fn aligned0x10 (size: u64) -> u64 {
    (size + 0xf) & !0xf
}

///
pub fn gen_funcdef(
    func_def: &FuncDef,
    module_context: &ModuleContext,
) -> Result<String, GenAsmErr> {
    let mut asm = String::new();
    // prorogue

    // The offset must be in 16-byte units
    //                               |<-- this must be in 16 byte units -->|
    let raw_size = func_def.arg_size + func_def.local_size + Byte::new(16);
    let aligned_stack_size = aligned0x10(raw_size.value);

    // set label name
    asm.push_str(&format!("{}:\n", func_def.name)); // TODO:

    // set stack pointer
    // set frame pointer
    let prologue: String = format!("
    # --- prologue ---
    addi sp, sp, -{}
    sd fp, {}(sp)
    addi fp, sp, {}
    sd ra, -8(fp)
", aligned_stack_size, aligned_stack_size - 0x10, aligned_stack_size);
    asm.push_str(&prologue);

    let instructions = &func_def.ir.0;
    for instruction in instructions {

        match instruction {
            Instruction::Call { dest, func, args } => {

            }
            _ => {

            }
        }
    }

    let epilogue: String = format!("
    # --- epilogue ---
    ld ra, -8(fp)
    ld fp, -16(fp)

    addi sp, sp, {}      # スタックを片付ける (spをfpと同じ高さに戻す)
    ret                  # as same as `jalr zero, 0(ra)`
", aligned_stack_size);
    asm.push_str(&epilogue);
    Ok(asm)
}


fn gen_funccall (
    dest:&Option<VReg>, func: &Func, args:&Vec<Operand>,
    func_def: &FuncDef,
    module_context: &ModuleContext,
) -> Result<String, GenAsmErr> {
    let mut asm = String::new();

    // counts of args you want to store on the stack
    for (i, operand) in args.iter().enumerate() {
        let asm_ins = match operand {
            Operand::Const(const_val) => {
                    // all args stored under the fp
                Ok(match i {
                    0 => PhysReg::A0.gen_load_immediate(const_val),
                    1 => PhysReg::A1.gen_load_immediate(const_val),
                    2 => PhysReg::A2.gen_load_immediate(const_val),
                    3 => PhysReg::A3.gen_load_immediate(const_val),
                    4 => PhysReg::A4.gen_load_immediate(const_val),
                    5 => PhysReg::A5.gen_load_immediate(const_val),
                    6 => PhysReg::A6.gen_load_immediate(const_val),
                    7 => PhysReg::A7.gen_load_immediate(const_val),
                    8.. => 
                        PhysStack::new(BasePointer::Sp(8 * (i as i32 - 8)), const_val.get_size())
                            .gen_load_immediate(const_val)
                    
                })
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
                            PhysStack::new(BasePointer::Sp(8 * (i as i32 - 8)), vreg_data.size)
                                .gen_load_asm(location)
                        
                    }
                } else {
                    Err(GenAsmErr::VregNotLocated)
                }
            }
        }?;

    }

    asm.push_str(
        &format!("jal ra, {}", 
            module_context
                .get_func(func.0)
                .expect("name is not set") 
                .name));

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

    Ok(asm)
}


/// 
pub fn codegen(mod_ctx: &ModuleContext) -> String {
    let mut asm = String::new();
    asm
}

struct Codegen {

}


impl Codegen {
    fn load_operand(dist: &Operand, src: &Operand, v_reg_arena: &VregArena) -> String {
        todo!();
    }

    fn store_operand () {

    }
}

#[cfg(test)]
mod rv64_codegen_test {
    use crate::*;

    #[test]
    fn test00 (){
        let mut mod_ctx = ModuleContext::new();

        let func_id = mod_ctx.create_func(
            "func", 
            Byte::new(2),
            Byte::new(2)
        );

        // other function definitions ...

        // let instruction = vec![
        // ];

        // mod_ctx.get_func_mut(func_id).set_ir(RvIR(instruction));

        // let asm = codegen(&mod_ctx);
        // println!("{}", asm);
    }
}
