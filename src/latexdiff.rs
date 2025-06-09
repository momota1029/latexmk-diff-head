use clap::Args;
use std::process::Command;

/// latexdiff コマンドラインオプション
#[derive(Args, Debug)]
pub struct LatexdiffOpts {
    /// \DIFadd/\DIFdel のスタイルを指定
    #[arg(short = 't', long = "type", value_name = "markupstyle")]
    pub markup_style: Option<String>,

    /// ブロック開始・終了コマンドのスタイル
    #[arg(short = 's', long = "subtype", value_name = "markstyle")]
    pub sub_style: Option<String>,

    /// 浮動環境内のマークアップスタイル
    #[arg(short = 'f', long = "floattype", value_name = "markstyle")]
    pub float_style: Option<String>,

    /// ファイルの文字エンコーディング
    #[arg(short = 'e', long = "encoding", value_name = "enc")]
    pub encoding: Option<String>,

    /// 独自プリアンブルファイル
    #[arg(short = 'p', long = "preamble", value_name = "file")]
    pub preamble: Option<String>,

    /// 使用パッケージ一覧 (カンマ区切り)
    #[arg(long = "packages", value_name = "pkg1,pkg2,...")]
    pub packages: Option<String>,

    /// プリアンブル表示
    #[arg(long = "show-preamble", action)]
    pub show_preamble: bool,

    /// 安全コマンドから除外 (正規表現)
    #[arg(short = 'A', long = "exclude-safecmd", value_name = "pattern")]
    pub exclude_safe_cmd: Option<String>,

    /// 安全コマンドに追加
    #[arg(short = 'a', long = "append-safecmd", value_name = "pattern")]
    pub append_safe_cmd: Option<String>,

    /// 安全コマンドを置換
    #[arg(long = "replace-safecmd", value_name = "pattern")]
    pub replace_safe_cmd: Option<String>,

    /// テキストコマンドから除外
    #[arg(short = 'X', long = "exclude-textcmd", value_name = "pattern")]
    pub exclude_text_cmd: Option<String>,

    /// テキストコマンドに追加
    #[arg(short = 'x', long = "append-textcmd", value_name = "pattern")]
    pub append_text_cmd: Option<String>,

    /// テキストコマンドを置換
    #[arg(long = "replace-textcmd", value_name = "pattern")]
    pub replace_text_cmd: Option<String>,

    /// 文脈依存テキストコマンドに追加
    #[arg(long = "append-context1cmd", value_name = "pattern")]
    pub append_context1_cmd: Option<String>,

    /// 文脈依存テキストコマンドを置換
    #[arg(long = "replace-context1cmd", value_name = "pattern")]
    pub replace_context1_cmd: Option<String>,

    /// 文脈2依存テキストコマンドに追加
    #[arg(long = "append-context2cmd", value_name = "pattern")]
    pub append_context2_cmd: Option<String>,

    /// 文脈2依存テキストコマンドを置換
    #[arg(long = "replace-context2cmd", value_name = "pattern")]
    pub replace_context2_cmd: Option<String>,

    /// mbox 保護対象コマンドから除外
    #[arg(long = "exclude-mboxsafecmd", value_name = "pattern")]
    pub exclude_mbox_safe_cmd: Option<String>,

    /// mbox 保護対象コマンドに追加
    #[arg(long = "append-mboxsafecmd", value_name = "pattern")]
    pub append_mbox_safe_cmd: Option<String>,

    /// 設定変数一括設定 (var=val,...)
    #[arg(short = 'c', long = "config", value_name = "var1=val1,...")]
    pub config: Option<String>,

    /// 正規表現変数に追加
    #[arg(long = "add-to-config", value_name = "var=pattern1;...")]
    pub add_to_config: Option<String>,

    /// 安全コマンドリストを表示
    #[arg(long = "show-safecmd", action)]
    pub show_safe_cmd: bool,

    /// テキストコマンドリストを表示
    #[arg(long = "show-textcmd", action)]
    pub show_text_cmd: bool,

    /// 設定変数を表示
    #[arg(long = "show-config", action)]
    pub show_config: bool,

    /// すべての表示オプションをまとめて実行
    #[arg(long = "show-all", action)]
    pub show_all: bool,

    /// 数式差分の粒度
    #[arg(long = "math-markup", value_name = "level")]
    pub math_markup: Option<String>,

    /// グラフィックス差分の方法
    #[arg(long = "graphics-markup", value_name = "mode")]
    pub graphics_markup: Option<String>,

    /// 引用マークアップ無効化
    #[arg(long = "disable-citation-markup", action)]
    pub disable_citation_markup: bool,

    /// auto-mbox 無効化
    #[arg(long = "disable-auto-mbox", action)]
    pub disable_auto_mbox: bool,

    /// 引用マークアップ有効化
    #[arg(long = "enable-citation-markup", action)]
    pub enable_citation_markup: bool,

    /// auto-mbox 強制化
    #[arg(long = "enforce-auto-mbox", action)]
    pub enforce_auto_mbox: bool,

    /// verbose モード
    #[arg(short = 'V', long = "verbose", action)]
    pub verbose: bool,

    /// driver タイプ指定
    #[arg(long = "driver", value_name = "type")]
    pub driver: Option<String>,

    /// 警告抑制
    #[arg(long = "ignore-warnings", action)]
    pub ignore_warnings: bool,

    /// 差分ファイルのラベル設定
    #[arg(short = 'L', long = "label", value_name = "label")]
    pub label: Option<String>,

    /// ラベル行を抑制
    #[arg(long = "no-label", action)]
    pub no_label: bool,

    /// ラベルを可視化
    #[arg(long = "visible-label", action)]
    pub visible_label: bool,

    /// \\input/\\include を展開
    #[arg(long = "flatten", action)]
    pub flatten: bool,
}

impl LatexdiffOpts {
    /// struct の値を Command に反映する
    pub fn args_to(&self, cmd: &mut Command) {
        if let Some(v) = &self.markup_style {
            cmd.arg(format!("--type={}", v));
        }
        if let Some(v) = &self.sub_style {
            cmd.arg(format!("--subtype={}", v));
        }
        if let Some(v) = &self.float_style {
            cmd.arg(format!("--floattype={}", v));
        }
        if let Some(v) = &self.encoding {
            cmd.arg(format!("--encoding={}", v));
        } else {
            cmd.arg("--encoding=utf8");
        }

        if let Some(v) = &self.preamble {
            cmd.arg(format!("--preamble={}", v));
        }
        if let Some(v) = &self.packages {
            cmd.arg(format!("--packages={}", v));
        }
        if self.show_preamble {
            cmd.arg("--show-preamble");
        }
        if let Some(v) = &self.exclude_safe_cmd {
            cmd.arg(format!("--exclude-safecmd={}", v));
        }
        if let Some(v) = &self.append_safe_cmd {
            cmd.arg(format!("--append-safecmd={}", v));
        }
        if let Some(v) = &self.replace_safe_cmd {
            cmd.arg(format!("--replace-safecmd={}", v));
        }
        if let Some(v) = &self.exclude_text_cmd {
            cmd.arg(format!("--exclude-textcmd={}", v));
        }
        if let Some(v) = &self.append_text_cmd {
            cmd.arg(format!("--append-textcmd={}", v));
        }
        if let Some(v) = &self.replace_text_cmd {
            cmd.arg(format!("--replace-textcmd={}", v));
        }
        if let Some(v) = &self.append_context1_cmd {
            cmd.arg(format!("--append-context1cmd={}", v));
        }
        if let Some(v) = &self.replace_context1_cmd {
            cmd.arg(format!("--replace-context1cmd={}", v));
        }
        if let Some(v) = &self.append_context2_cmd {
            cmd.arg(format!("--append-context2cmd={}", v));
        }
        if let Some(v) = &self.replace_context2_cmd {
            cmd.arg(format!("--replace-context2cmd={}", v));
        }
        if let Some(v) = &self.exclude_mbox_safe_cmd {
            cmd.arg(format!("--exclude-mboxsafecmd={}", v));
        }
        if let Some(v) = &self.append_mbox_safe_cmd {
            cmd.arg(format!("--append-mboxsafecmd={}", v));
        }
        if let Some(v) = &self.config {
            cmd.arg(format!("--config={}", v));
        }
        if let Some(v) = &self.add_to_config {
            cmd.arg(format!("--add-to-config={}", v));
        }
        if self.show_safe_cmd {
            cmd.arg("--show-safecmd");
        }
        if self.show_text_cmd {
            cmd.arg("--show-textcmd");
        }
        if self.show_config {
            cmd.arg("--show-config");
        }
        if self.show_all {
            cmd.arg("--show-all");
        }
        if let Some(v) = &self.math_markup {
            cmd.arg(format!("--math-markup={}", v));
        }
        if let Some(v) = &self.graphics_markup {
            cmd.arg(format!("--graphics-markup={}", v));
        }
        if self.disable_citation_markup {
            cmd.arg("--disable-citation-markup");
        }
        if self.disable_auto_mbox {
            cmd.arg("--disable-auto-mbox");
        }
        if self.enable_citation_markup {
            cmd.arg("--enable-citation-markup");
        }
        if self.enforce_auto_mbox {
            cmd.arg("--enforce-auto-mbox");
        }
        if self.verbose {
            cmd.arg("--verbose");
        }
        if let Some(v) = &self.driver {
            cmd.arg(format!("--driver={}", v));
        }
        if self.ignore_warnings {
            cmd.arg("--ignore-warnings");
        }
        if let Some(v) = &self.label {
            cmd.arg(format!("--label={}", v));
        }
        if self.no_label {
            cmd.arg("--no-label");
        }
        if self.visible_label {
            cmd.arg("--visible-label");
        }
        if self.flatten {
            cmd.arg("--flatten");
        }
    }
}
