use crate::codegen::rv64::{PhysReg};

/// 最終的なアセンブリプログラム全体を表現する構造体
#[derive(Debug, Clone)]
pub struct Asm {
    pub statements: Vec<AsmStatement>,
}

/// アセンブリの1行（要素）を表現する列挙型
#[derive(Debug, Clone)]
pub enum AsmStatement {
    Instruction(RvInst),
    Label(String),
    Directive(Directive),
    Comment(String),
}

/// アセンブラへの指示（将来ELFのセクション分けに直結します）
#[derive(Debug, Clone)]
pub enum Directive {
    Text,               // .text (コード領域の開始)
    Data,               // .data (データ領域の開始)
    Global(String),     // .global name (外部公開シンボル)
    Asciz(String),      // .asciz "..." (ヌル終端文字列)
    Align(u32),         // .align N
}

/// RISC-Vの実際の命令群（コンパイラが使うものだけ定義すればOK）
#[derive(Debug, Clone)]
pub enum RvInst {
    // --- R-Type (レジスタ間演算) ---
    /// レジスタrdに加算の結果を書き込む
    Add { rd: PhysReg, rs1: PhysReg, rs2: PhysReg },
    /// レジスタrdに加算の結果を書き込む
    Sub { rd: PhysReg, rs1: PhysReg, rs2: PhysReg },

    // --- I-Type (即値演算・ロード) ---
    /// レジスタrdに加算の結果を書き込む
    Addi { rd: PhysReg, rs1: PhysReg, imm: i32 },
    /// メモリの内容をレジスタに書き込む　
    Ld   { rd: PhysReg, base: PhysReg, offset: i32 }, // 64bit load
    /// メモリの内容をレジスタに書き込む　
    Lw   { rd: PhysReg, base: PhysReg, offset: i32 }, // 32bit load

    /// レジスタ指すアドレスにjumpする命令
    Jalr { rd: PhysReg, base: PhysReg, offset: i32 },
    /// jal rd, label
    /// rd に戻り先アドレス(PC+4)を保存し、label の位置へジャンプする
    Jal { rd: PhysReg, label: String },
    
    // --- S-Type (ストア) ---
    /// レジスタの内容をメモリに書き込む
    Sd   { rs2: PhysReg, base: PhysReg, offset: i32 }, // 64bit store
    /// レジスタの内容をメモリに書き込む
    Sw   { rs2: PhysReg, base: PhysReg, offset: i32 }, // 32bit store

    // --- 疑似命令 (Pseudo Instructions) ---
    // ※将来ELFを出力する際は、ここで複数のネイティブ命令に展開（Lowering）します
    /// 定数をレジスタにセットする
    Li   { rd: PhysReg, imm: i64 },
    Mv   { rd: PhysReg, rs: PhysReg },
    Call { symbol: String },
    Ret,
}

use std::fmt;
// use crate::codegen::rv64::PhysReg; // (PhysRegのインポートが必要です)

// ==========================================
// Asm全体に対する実装
// ==========================================
impl fmt::Display for Asm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for stmt in &self.statements {
            // 各ステートメントを改行区切りで出力する
            writeln!(f, "{}", stmt)?;
        }
        Ok(())
    }
}

// ==========================================
// 1行の要素(AsmStatement)に対する実装
// ==========================================
impl fmt::Display for AsmStatement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // 命令とコメントは読みやすくインデント（タブまたはスペース4つ）を入れる
            AsmStatement::Instruction(inst) => write!(f, "    {}", inst),
            AsmStatement::Comment(text)     => write!(f, "    # {}", text),
            // ラベルやディレクティブは左詰めで出力
            AsmStatement::Label(name)       => write!(f, "{}:", name),
            AsmStatement::Directive(dir)    => write!(f, "{}", dir),
        }
    }
}

// ==========================================
// アセンブラ・ディレクティブに対する実装
// ==========================================
impl fmt::Display for Directive {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Directive::Text => write!(f, ".text"),
            Directive::Data => write!(f, ".data"),
            Directive::Global(name) => write!(f, ".global {}", name),
            Directive::Asciz(s) => write!(f, ".asciz \"{}\"", s),
            Directive::Align(n) => write!(f, ".align {}", n),
        }
    }
}

// ==========================================
// 命令群(RvInst)に対する実装
// ==========================================
impl fmt::Display for RvInst {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // ※PhysReg には前回定義した `get_reg_name()` が実装されている前提です
        match self {
            // --- R-Type ---
            RvInst::Add { rd, rs1, rs2 } => write!(f, "add {}, {}, {}", rd.get_reg_name(), rs1.get_reg_name(), rs2.get_reg_name()),
            RvInst::Sub { rd, rs1, rs2 } => write!(f, "sub {}, {}, {}", rd.get_reg_name(), rs1.get_reg_name(), rs2.get_reg_name()),

            // --- I-Type ---
            RvInst::Addi { rd, rs1, imm } => write!(f, "addi {}, {}, {}", rd.get_reg_name(), rs1.get_reg_name(), imm),
            RvInst::Ld { rd, base, offset } => write!(f, "ld {}, {}({})", rd.get_reg_name(), offset, base.get_reg_name()),
            RvInst::Lw { rd, base, offset } => write!(f, "lw {}, {}({})", rd.get_reg_name(), offset, base.get_reg_name()),
            RvInst::Jalr { rd, base, offset } => write!(f, "jalr {}, {}({})", rd.get_reg_name(), offset, base.get_reg_name()),
            RvInst::Jal { rd, label } => write!(f, "jal {}, {}", rd.get_reg_name(), label),

            // --- S-Type ---
            RvInst::Sd { rs2, base, offset } => write!(f, "sd {}, {}({})", rs2.get_reg_name(), offset, base.get_reg_name()),
            RvInst::Sw { rs2, base, offset } => write!(f, "sw {}, {}({})", rs2.get_reg_name(), offset, base.get_reg_name()),

            // --- Pseudo Instructions ---
            RvInst::Li { rd, imm } => write!(f, "li {}, {}", rd.get_reg_name(), imm),
            RvInst::Mv { rd, rs } => write!(f, "mv {}, {}", rd.get_reg_name(), rs.get_reg_name()),
            RvInst::Call { symbol } => write!(f, "call {}", symbol),
            RvInst::Ret => write!(f, "ret"),
        }
    }
}
