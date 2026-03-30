/* エントリーポイントを _start シンボルに指定 */
ENTRY(_start)

SECTIONS {
    /* QEMUのRISC-V virtマシンの標準的なRAM開始アドレスなど、
       OSをロードする物理アドレスを指定します */
    . = 0x80000000;

    .text : {
        /* _start を含むブートコードを確実に先頭に配置するテクニック */
        KEEP(*(.text.boot))
        *(.text .text.*)
    }

    /* この後に .data や .bss セクションの定義が続きます */
}
