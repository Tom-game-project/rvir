pub mod unit;
pub mod ir;
pub mod codegen;

use crate::ir::ir::*;
use crate::unit::size::Byte;

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

        // let asm = codegen(&mod_ctx);
        // println!("{}", asm);
    }
}
