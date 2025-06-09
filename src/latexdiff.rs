use clap::Args;
use std::process::Command;

/// Configuration options for latexdiff command
#[derive(Args, Debug, Clone)]
pub struct LatexdiffOpts {
    /// Markup style for \DIFadd and \DIFdel commands
    #[arg(short = 't', long = "type", value_name = "markupstyle")]
    pub markup_style: Option<String>,

    /// Style for block start/end commands
    #[arg(short = 's', long = "subtype", value_name = "markstyle")]
    pub sub_style: Option<String>,

    /// Markup style within floating environments
    #[arg(short = 'f', long = "floattype", value_name = "markstyle")]
    pub float_style: Option<String>,

    /// Character encoding for input files [default: utf8]
    #[arg(short = 'e', long = "encoding", value_name = "enc")]
    pub encoding: Option<String>,

    /// Custom preamble file for diff output
    #[arg(short = 'p', long = "preamble", value_name = "file")]
    pub preamble: Option<String>,

    /// Comma-separated list of required packages
    #[arg(long = "packages", value_name = "pkg1,pkg2,...")]
    pub packages: Option<String>,

    /// Display the preamble being used
    #[arg(long = "show-preamble", action)]
    pub show_preamble: bool,

    /// Exclude commands from safe command list (regex pattern)
    #[arg(short = 'A', long = "exclude-safecmd", value_name = "pattern")]
    pub exclude_safe_cmd: Option<String>,

    /// Add commands to safe command list (regex pattern)
    #[arg(short = 'a', long = "append-safecmd", value_name = "pattern")]
    pub append_safe_cmd: Option<String>,

    /// Replace safe command list entirely (regex pattern)
    #[arg(long = "replace-safecmd", value_name = "pattern")]
    pub replace_safe_cmd: Option<String>,

    /// Exclude commands from text command list (regex pattern)
    #[arg(short = 'X', long = "exclude-textcmd", value_name = "pattern")]
    pub exclude_text_cmd: Option<String>,

    /// Add commands to text command list (regex pattern)
    #[arg(short = 'x', long = "append-textcmd", value_name = "pattern")]
    pub append_text_cmd: Option<String>,

    /// Replace text command list entirely (regex pattern)
    #[arg(long = "replace-textcmd", value_name = "pattern")]
    pub replace_text_cmd: Option<String>,

    /// Add commands to context1 command list (regex pattern)
    #[arg(long = "append-context1cmd", value_name = "pattern")]
    pub append_context1_cmd: Option<String>,

    /// Replace context1 command list entirely (regex pattern)
    #[arg(long = "replace-context1cmd", value_name = "pattern")]
    pub replace_context1_cmd: Option<String>,

    /// Add commands to context2 command list (regex pattern)
    #[arg(long = "append-context2cmd", value_name = "pattern")]
    pub append_context2_cmd: Option<String>,

    /// Replace context2 command list entirely (regex pattern)
    #[arg(long = "replace-context2cmd", value_name = "pattern")]
    pub replace_context2_cmd: Option<String>,

    /// Exclude commands from mbox-safe command list (regex pattern)
    #[arg(long = "exclude-mboxsafecmd", value_name = "pattern")]
    pub exclude_mbox_safe_cmd: Option<String>,

    /// Add commands to mbox-safe command list (regex pattern)
    #[arg(long = "append-mboxsafecmd", value_name = "pattern")]
    pub append_mbox_safe_cmd: Option<String>,

    /// Set configuration variables (var=val,var2=val2,...)
    #[arg(short = 'c', long = "config", value_name = "var1=val1,...")]
    pub config: Option<String>,

    /// Add patterns to regex variables (var=pattern1;pattern2;...)
    #[arg(long = "add-to-config", value_name = "var=pattern1;...")]
    pub add_to_config: Option<String>,

    /// Display current safe command list
    #[arg(long = "show-safecmd", action)]
    pub show_safe_cmd: bool,

    /// Display current text command list
    #[arg(long = "show-textcmd", action)]
    pub show_text_cmd: bool,

    /// Display all configuration variables
    #[arg(long = "show-config", action)]
    pub show_config: bool,

    /// Execute all --show-* options together
    #[arg(long = "show-all", action)]
    pub show_all: bool,

    /// Math markup granularity level (off, whole, coarse, fine)
    #[arg(long = "math-markup", value_name = "level")]
    pub math_markup: Option<String>,

    /// Graphics markup handling mode (off, new-only, both)
    #[arg(long = "graphics-markup", value_name = "mode")]
    pub graphics_markup: Option<String>,

    /// Disable citation markup processing
    #[arg(long = "disable-citation-markup", action)]
    pub disable_citation_markup: bool,

    /// Disable automatic mbox protection
    #[arg(long = "disable-auto-mbox", action)]
    pub disable_auto_mbox: bool,

    /// Enable citation markup processing
    #[arg(long = "enable-citation-markup", action)]
    pub enable_citation_markup: bool,

    /// Force automatic mbox protection
    #[arg(long = "enforce-auto-mbox", action)]
    pub enforce_auto_mbox: bool,

    /// Specify driver type for output format
    #[arg(long = "driver", value_name = "type")]
    pub driver: Option<String>,

    /// Suppress warning messages
    #[arg(long = "ignore-warnings", action)]
    pub ignore_warnings: bool,

    /// Set label for diff output identification
    #[arg(short = 'L', long = "label", value_name = "label")]
    pub label: Option<String>,

    /// Suppress label line in diff output
    #[arg(long = "no-label", action)]
    pub no_label: bool,

    /// Make labels visible in the output
    #[arg(long = "visible-label", action)]
    pub visible_label: bool,
}

impl LatexdiffOpts {
    /// struct の値を Command に反映する
    pub fn args_to(&self, verbose: bool, cmd: &mut Command) {
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
        if verbose {
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
    }
}
