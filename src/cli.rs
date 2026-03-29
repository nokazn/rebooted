use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "rebooted",
    about = "再起動後にコマンドを実行するCLIツール",
    long_about = "システムを再起動し、次回ログイン時に指定したコマンドを1度だけ実行します。\n\n例: rebooted -- echo 'hello after reboot'"
)]
pub struct Cli {
    /// 再起動後に実行するコマンドと引数
    #[arg(last = true, required = false)]
    pub command: Vec<String>,

    /// 再起動せずサービス登録のみ行う
    #[arg(long)]
    pub dry_run: bool,

    /// サービス識別ラベル（省略時はコマンドから自動生成）
    #[arg(long)]
    pub label: Option<String>,

    /// 内部実行モード: 再起動後にサービスからコマンドを実行して自己削除する
    #[arg(long, hide = true)]
    pub internal_exec: Option<String>,
}
