#[cfg(test)]
mod ir_tests {
    use rvir::unit::size::Byte;
    use rvir::ir::ir::{BasicBlockList, ConstValue, Instruction, Label, ModuleContext, Operand, Operator};

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
                    println!("    basic_block id: {:?}", basic_block.id);
                    println!("    reach {:?}", basic_block.vreg_state_set.get_reach_statement_id_list());
                    println!("    live {:?}", basic_block.vreg_state_set.get_live_statement_id_list());
                }
            } else {
                println!("failed to setting basic_block_list");
            }
        }

    }

    #[test]
    fn test01() {
        let mut mod_ctx = ModuleContext::new();

        let func_id = mod_ctx.create_func(
            "test_func",
            Byte::new(0),
            Byte::new(0x10 * 6));

        {
            let func = mod_ctx.get_func_mut(func_id).unwrap();
            let tmp_reg_a = func.vreg_arena.alloc(Byte::new(8), Some(String::from("a")));
            let tmp_reg_b = func.vreg_arena.alloc(Byte::new(8), Some(String::from("b")));
            let tmp_reg_c = func.vreg_arena.alloc(Byte::new(8), Some(String::from("c")));
            let tmp_reg_d = func.vreg_arena.alloc(Byte::new(8), Some(String::from("d")));
            let tmp_reg_e = func.vreg_arena.alloc(Byte::new(8), Some(String::from("e")));

            let mut basic_block_list = BasicBlockList::new();

            // irのユーザーは事前に基本ブロックを構成する必要がある
            let block_id_0 = basic_block_list.alloc(Label("block0".to_string()));
            let block_id_1 = basic_block_list.alloc(Label("block1".to_string()));

            let basic_block_start = basic_block_list.alloc(Label("block_start".to_string()));

            basic_block_list.set_inst(
                basic_block_start,
                vec![
                Instruction::Assign { dest: tmp_reg_d, src: Operand::Const(ConstValue::I64(0)) },
                Instruction::Jump { target: Label("block0".to_string()) }
            ]);

            basic_block_list.set_inst(
                block_id_0,
                vec![
                Instruction::BinOp { op: Operator::Mul, dest: tmp_reg_a, lhs: Operand::Reg(tmp_reg_d), rhs: Operand::Const(ConstValue::I64(2)) },
                Instruction::BinOp { op: Operator::Add, dest: tmp_reg_b, lhs: Operand::Reg(tmp_reg_a), rhs: Operand::Reg(tmp_reg_e) },
                Instruction::BinOp { op: Operator::Mul, dest: tmp_reg_c, lhs: Operand::Reg(tmp_reg_a), rhs: Operand::Const(ConstValue::I64(3)) },
                Instruction::BinOp { op: Operator::Add, dest: tmp_reg_d, lhs: Operand::Reg(tmp_reg_b), rhs: Operand::Reg(tmp_reg_c) },
                Instruction::BinOp { op: Operator::Sub, dest: tmp_reg_e, lhs: Operand::Reg(tmp_reg_d), rhs: Operand::Const(ConstValue::I64(5)) },
                Instruction::Jump { target: Label("block1".to_string()) }
            ]);

            basic_block_list.set_inst(
                block_id_1,
                vec![
                Instruction::Jump { target: Label("block0".to_string()) }
            ]);

            let basic_block_list = basic_block_list.finish_ir_setting();
            if let Ok(basic_block_list ) = basic_block_list.set_pred_and_succ() {
                for basic_block in &basic_block_list.list {
                    println!("{}:", basic_block.label.0);
                    println!("    basic_block id: {:?}", basic_block.id);
                    println!("    reach {:?}", basic_block.vreg_state_set.get_reach_statement_id_list());
                    println!("    live {:?}", basic_block.vreg_state_set.get_live_statement_id_list());
                }
            } else {
                println!("failed to setting basic_block_list");
            }
        }
    }

    /// DOMが導出できているかどうかを確認する
    #[test]
    fn test02() {
        let mut mod_ctx = ModuleContext::new();

        let func_id = mod_ctx.create_func(
            "test_func",
            Byte::new(0),
            Byte::new(0x10 * 6));

        {
            let func = mod_ctx.get_func_mut(func_id).unwrap();

            let mut basic_block_list = BasicBlockList::new();

            // irのユーザーは事前に基本ブロックを構成する必要がある
            let block_id_0 = basic_block_list.alloc(Label("block0".to_string()));
            let block_id_1 = basic_block_list.alloc(Label("block1".to_string()));
            let block_id_2 = basic_block_list.alloc(Label("block2".to_string()));
            let block_id_3 = basic_block_list.alloc(Label("block3".to_string()));

            let block_id_exit = basic_block_list.alloc(Label("exit".to_string()));

            basic_block_list.set_inst(
                block_id_0,
                vec![
                // Instruction::Jump { target: Label("block1".to_string()) }
                Instruction::Branch { 
                    cond: Operand::Const(ConstValue::I64(0)), 
                    true_label: Label("exit".to_string()),
                    false_label: Label("block1".to_string()) ,
                }
            ]);

            basic_block_list.set_inst(
                block_id_1,
                vec![
                // Instruction::Jump { target: Label("block1".to_string()) }
                Instruction::Branch { 
                    cond: Operand::Const(ConstValue::I64(0)),
                    true_label: Label("block2".to_string()),
                    false_label: Label("block3".to_string()),
                }
            ]);

            basic_block_list.set_inst(
                block_id_2,
                vec![
                Instruction::Jump { target: Label("block0".to_string()) }
            ]);

            basic_block_list.set_inst(
                block_id_3,
                vec![
                Instruction::Jump { target: Label("block0".to_string()) }
            ]);

            basic_block_list.set_inst(
                block_id_exit, 
                vec![]
            );

            basic_block_list.set_entry_block(block_id_0);

            let basic_block_list = basic_block_list.finish_ir_setting();
            if let Ok(basic_block_list ) = basic_block_list.set_pred_and_succ() {
                for basic_block in &basic_block_list.list {
                    println!("{}:", basic_block.label.0);
                    println!("    basic_block id: {:?}", basic_block.id);
                    println!("    dom {:?}", basic_block.get_dom_basic_block_ids());
                }
            } else {
                println!("failed to setting basic_block_list");
            }
        }
    }
}
