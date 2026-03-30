# setup.gdb

# 1. アーキテクチャの設定
set architecture riscv:rv64

# 2. GDB終了時にQEMUも一緒に終了(kill)させるフック
# set confirm off で「本当に終了しますか？」の確認ダイアログを消す
define hook-quit
  set confirm off
  kill
end

# 3. QEMUへの接続
# （この時点では既に ~/.gdbinit からGEFが読み込まれているので、gef-remoteが使えます）
target remote localhost:1234

