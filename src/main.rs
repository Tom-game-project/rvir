pub mod unit;
pub mod ir;

use crate::ir::ir::*;

use crate::unit::size::Byte;


// register
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reg { T0, T1, T2, T3, T4, T5, T6 }

fn gen_funcdef(func_def: &FuncDef) -> String {
    let mut asm = String::new();

    // prorogue

    let stack_size = func_def.arg_size + func_def.local_size + Byte::new(16);

    // set stack pointer
    // set frame pointer
    let prologue: String = format!("
    addi sp, sp, -{}
    sd fp, 0(sp)
    addi fp, sp, {}
    sd ra, -8(fp)
", stack_size.value, stack_size.value);

    asm.push_str(&prologue);

    let instructions = &func_def.ir.0;
    for instruction in instructions {
        match instruction {
            _ => {

            }
        }
    }
    asm
}

/// 
fn codegen(mod_ctx: &ModuleContext) -> String {
    let mut asm = String::new();
    asm
}

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod test {
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

        let instruction = vec![
        ];

        mod_ctx.get_func_mut(func_id).set_ir(RvIR(instruction));

        let asm = codegen(&mod_ctx);
        println!("{}", asm);
    }
}
