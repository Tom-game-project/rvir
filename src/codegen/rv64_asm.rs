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
    Add { rd: PhysReg, rs1: PhysReg, rs2: PhysReg },
    Sub { rd: PhysReg, rs1: PhysReg, rs2: PhysReg },
    
    // --- I-Type (即値演算・ロード) ---
    Addi { rd: PhysReg, rs1: PhysReg, imm: i32 },
    Ld   { rd: PhysReg, base: PhysReg, offset: i32 }, // 64bit load
    Lw   { rd: PhysReg, base: PhysReg, offset: i32 }, // 32bit load
    Jalr { rd: PhysReg, base: PhysReg, offset: i32 },
    /// jal rd, label
    /// rd に戻り先アドレス(PC+4)を保存し、label の位置へジャンプする
    Jal { rd: PhysReg, label: String },
    
    // --- S-Type (ストア) ---
    Sd   { rs2: PhysReg, base: PhysReg, offset: i32 }, // 64bit store
    Sw   { rs2: PhysReg, base: PhysReg, offset: i32 }, // 32bit store

    // --- 疑似命令 (Pseudo Instructions) ---
    // ※将来ELFを出力する際は、ここで複数のネイティブ命令に展開（Lowering）します
    Li   { rd: PhysReg, imm: i64 },
    Mv   { rd: PhysReg, rs: PhysReg },
    Call { symbol: String },
    Ret,
}
