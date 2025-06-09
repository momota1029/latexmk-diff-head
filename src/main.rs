mod latexdiff;

use bstr::io::BufReadExt as _;
use clap::{Args, Parser};
use std::{
    ffi::{OsStr, OsString},
    io::{self, BufReader, Write as _},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};
use thiserror::Error;

const APPLYING_RULE_PAT: &[u8] = b"Latexmk: applying rule ";
const ALL_TARGETS_PAT: &[u8] = b"Latexmk: All targets ";

fn main() -> io::Result<()> {
    let param = Opts::parse().to_param()?;
    let latexmk = param.to_latexmk();
    // 普通のlatexmk
    let mut latexmk_spawn = latexmk.command()?.stdout(Stdio::piped()).stderr(Stdio::inherit()).spawn()?;
    let Some(mut latexmk_stdout) = latexmk_spawn.stdout.take() else { return Ok(()) }; // 出力がないってことは無いだろ……
    let mut stdio = std::io::stdout();
    let mut enable_typeset = false;
    // typesetが行われたかどうかを判定しながら出力を素通りさせる
    BufReader::new(&mut latexmk_stdout).for_byte_line_with_terminator(|line| {
        stdio.write_all(line)?;
        Ok(if line.starts_with(APPLYING_RULE_PAT) {
            enable_typeset = true;
            false
        } else if line.starts_with(ALL_TARGETS_PAT) {
            false
        } else {
            true
        })
    })?;
    // タイプセットが行われていないときはdiffも取らず、とりあえず出力だけして切り上げる
    if !enable_typeset {
        std::io::copy(&mut latexmk_stdout, &mut stdio)?;
        let mk_status = latexmk_spawn.wait()?;
        if mk_status.success() {
            latexmk.rename_pdf()?;
        }
        // この場合LaTeX WorkshopがSyncTeX位置反映を怠るので、擬似的に出力があったということにしておく
        println!("Output written on dummy.pdf (for LaTeX Workshop's SyncTeX refresh on {:?}).", param.docfile);
        std::process::exit(mk_status.code().unwrap_or(1));
    }

    // そうでない場合、別スレッドで標準出力だけ横流しする。エラーの出ようがないので無視。
    let _ = std::thread::spawn(move || std::io::copy(&mut latexmk_stdout, &mut stdio));
    let diff_res = diffmk(&param); // latexdiff-vcからタイプセットまでを実行する
    // diffmkは`*_diff`について作業(`*_diff.aux`などを生成)し、`mk`は`*`について作業する(`*.aux`などを生成する)ため、生成ファイルやその処理が全く被らないことに注意(関係ないファイルを上書きすることはあるが、実行時のエラーになるわけではない)

    // メインのlatexmkが成功しなかったらその場で失敗する
    let mk_status = latexmk_spawn.wait()?;
    if !mk_status.success() {
        std::process::exit(mk_status.code().unwrap_or(1));
    }
    // そうでなければpdfを移動
    latexmk.rename_pdf()?;
    match diff_res {
        Ok(()) => {}
        Err(CError::Io(err)) => {
            eprintln!("{}", err);
            std::process::exit(1);
        }
        Err(CError::Main(err)) => {
            io::stderr().write_all(&err)?;
            std::process::exit(1);
        }
    }
    std::process::exit(mk_status.code().unwrap_or(1));
}

#[derive(Parser, Debug)]
struct Opts {
    /// LaTeX document path without extension (e.g., "paper/main" for "paper/main.tex")
    #[clap(value_parser)]
    doc: PathBuf,

    /// Directory for temporary files (.aux, .log, etc.) [default: <doc_dir>/.temp]
    #[clap(long, value_parser)]
    tmpdir: Option<PathBuf>,
    /// Output directory for final PDF [default: same as document directory]
    #[clap(long, short, value_parser)]
    outdir: Option<PathBuf>,

    /// Name of subdirectory for diff output [default: "diff"]
    #[clap(long, short, value_parser)]
    diff_name: Option<String>,
    /// Suffix added to diff filename [default: "-diff"]
    #[clap(long, value_parser)]
    diff_postfix: Option<String>,

    #[clap(flatten)]
    latexmk_opts: LatexmkOpts,
    #[clap(flatten)]
    latexdiff_opts: latexdiff::LatexdiffOpts,
    #[clap(flatten)]
    latexdiffvc_ops: LatexdiffVcOpts,
    // /// Use --flatten option for latexdiff-vc (handles \input/\include)
    // #[clap(long)]
    // flatten: bool,
    // /// Enable SyncTeX (`<docfile>.synctex.gz` file) generation
    // #[clap(long)]
    // synctex: bool,
}
impl Opts {
    fn to_param(self) -> io::Result<Param> {
        // stemなので存在しないパスの部分文字列であることに注意
        let doc = if self.doc.is_absolute() {
            self.doc
        } else {
            // 相対パスならカレントディレクトリ込みの絶対パスに変換
            Path::join(&std::env::current_dir()?, &self.doc)
        };
        // ここは存在しない(None)ことはない(joinされてるので)
        let parent = doc.parent().unwrap();
        let tmpdir = match self.tmpdir {
            None => parent.join(".temp"),
            Some(p) => {
                std::fs::create_dir_all(&p)?;
                std::fs::canonicalize(p)?
            }
        };
        let outdir = match self.outdir {
            None => parent.to_path_buf(),
            Some(p) => {
                std::fs::create_dir_all(&p)?;
                std::fs::canonicalize(p)?
            }
        };
        let diff_dir_name = self.diff_name.unwrap_or_else(|| "diff".to_string());
        let diff_postfix = self.diff_postfix.unwrap_or_else(|| format!("-{diff_dir_name}"));
        Ok(Param {
            dir: parent.to_path_buf(),
            // ここも存在しないことはない(joinされてるので)。doc.texを間違って指定してしまった場合を除外しようとするとdoc.tex.texが実在した場合に至極面倒。
            docfile: doc.file_name().unwrap().to_os_string(),
            tmpdir,
            outdir,
            diff_dir_name,
            diff_postfix,
            latexmk_opts: self.latexmk_opts,
            latexdiff_opts: self.latexdiff_opts,
            latexdiffvc_opts: self.latexdiffvc_ops,
        })
    }
}

// latexmk用オプション（最小限）
#[derive(Args, Debug)]
struct LatexmkOpts {
    /// LaTeX処理系選択
    #[clap(long, group = "engine")]
    xelatex: bool,
    #[clap(long, group = "engine")]
    lualatex: bool,

    /// 参考文献処理
    #[clap(long, group = "bib")]
    bibtex: bool,
    #[clap(long, group = "bib")]
    biber: bool,
    #[clap(long, group = "bib")]
    nobibtex: bool,

    /// Enable SyncTeX (`<docfile>.synctex.gz` file) generation
    #[clap(long)]
    synctex: bool,

    /// 出力レベル制御（相互排他）
    #[clap(long, group = "output")]
    silent: bool,
    #[clap(long, group = "output")]
    quiet: bool,
    #[clap(long, group = "output")]
    verbose: bool,

    /// その他のデバッグ・情報表示
    #[clap(long)]
    commands: bool,
}
impl LatexmkOpts {
    fn args_to(&self, cmd: &mut Command) {
        cmd.args(["-halt-on-error", "-file-line-error"]);
        if self.xelatex {
            cmd.arg("-xelatex");
        } else if self.lualatex {
            cmd.arg("-lualatex");
        } // 参考文献処理
        if self.bibtex {
            cmd.arg("-bibtex");
        } else if self.biber {
            cmd.arg("-biber");
        } else if self.nobibtex {
            cmd.arg("-nobibtex");
        }

        // 出力制御
        if self.silent {
            cmd.arg("-silent");
        } else if self.quiet {
            cmd.arg("-quiet");
        } else if self.verbose {
            cmd.arg("-verbose");
        }

        // その他
        if self.commands {
            cmd.arg("-commands");
        }

        if self.synctex {
            cmd.arg("-synctex=1");
        }
    }
}

// latexdiff-vc用オプション
#[derive(Args, Debug)]
struct LatexdiffVcOpts {
    /// バージョン管理システム（指定されない場合はgit使用）
    #[clap(long, group = "vcs")]
    git: bool,
    #[clap(long, group = "vcs")]
    svn: bool,
    #[clap(long, group = "vcs")]
    hg: bool,
    #[clap(long, group = "vcs")]
    cvs: bool,
    #[clap(long, group = "vcs")]
    rcs: bool,

    /// リビジョン指定（未指定時はHEAD使用）
    #[clap(long, short)]
    revision: Vec<String>,

    /// flatten関連
    #[clap(long, group = "flat")]
    flatten: bool,
    #[clap(long, group = "flat")] // flattenが指定された時のみ有効
    flatten_keep_intermediate: bool, // --flatten=keep-intermediate相当
}
impl LatexdiffVcOpts {
    fn args_to(&self, cmd: &mut Command) {
        if self.git {
            cmd.arg("--git");
        } else if self.svn {
            cmd.arg("--svn");
        } else if self.hg {
            cmd.arg("--hg");
        } else if self.cvs {
            cmd.arg("--cvs");
        } else if self.rcs {
            cmd.arg("--rcs");
        }
        if !self.revision.is_empty() {
            cmd.arg("--revision");
        }
        for rev in &self.revision {
            cmd.args(["--revision", rev]);
        }
        if self.flatten {
            cmd.arg("--flatten");
        } else if self.flatten_keep_intermediate {
            cmd.arg("--flatten=keep-intermediate");
        }
    }
}

#[derive(Debug, Error)]
enum CError {
    #[error("{}", String::from_utf8_lossy(.0))]
    Main(Vec<u8>),
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
}
fn osstr_join(path: impl AsRef<OsStr>, ext: &str) -> OsString {
    let mut buf = path.as_ref().to_os_string();
    buf.push(ext);
    buf
}

struct Param {
    dir: PathBuf,      // docの親ディレクトリ
    docfile: OsString, // file_stemに相当。拡張子は含まないし、ディレクトリも含まない

    tmpdir: PathBuf, // ユーザが指定する。latexmkが.auxファイル類を置いておく`--outdir`。ただし下のoutdirと区別するためにおいてある。
    outdir: PathBuf, // latexmkが2024年にout2dir(pdfを出力するディレクトリ)を用意しだしたらしいけど、どうしようもないのでこっちで代替するためのもの

    diff_dir_name: String,
    diff_postfix: String,

    latexdiff_opts: latexdiff::LatexdiffOpts,
    latexdiffvc_opts: LatexdiffVcOpts,
    latexmk_opts: LatexmkOpts,
}
impl Param {
    fn to_latexmk(&self) -> LaTeXMK {
        LaTeXMK { dir: &self.dir, docfile: &self.docfile, tmpdir: &self.tmpdir, outdir: &self.outdir, opts: &self.latexmk_opts }
    }
}

// dir+docfileとtepmdirを指定してlatexmkを実行し、pdfとsynctexをoutdirに移動する。
struct LaTeXMK<'a> {
    dir: &'a Path,
    docfile: &'a OsStr,
    tmpdir: &'a Path,
    outdir: &'a Path,
    opts: &'a LatexmkOpts,
}
impl LaTeXMK<'_> {
    fn command(&self) -> io::Result<Command> {
        let mut outdir_arg = OsString::from("-outdir=");
        outdir_arg.push(&self.tmpdir);
        let mut auxdir_arg = OsString::from("-auxdir=");
        auxdir_arg.push(&self.tmpdir);

        std::fs::create_dir_all(&self.tmpdir)?;
        let mut cmd = Command::new("latexmk");
        self.opts.args_to(&mut cmd);
        cmd.arg(self.dir.join(&self.docfile));
        Ok(cmd)
    }
    fn rename_pdf(self) -> io::Result<()> {
        let tmpdoc = self.tmpdir.join(&self.docfile);
        let outdoc = self.outdir.join(self.docfile);
        let pdf = PathBuf::from(osstr_join(&outdoc, ".pdf"));
        std::fs::copy(osstr_join(&tmpdoc, ".pdf"), &pdf)?;
        if self.opts.synctex {
            std::fs::copy(osstr_join(&tmpdoc, ".synctex.gz"), osstr_join(&outdoc, ".synctex.gz"))?;
        }
        Ok(())
    }
}

fn diffmk(param: &Param) -> Result<(), CError> {
    let stem_diff = osstr_join(&param.docfile, &param.diff_postfix);
    let mut latexdiff = Command::new("latexdiff-vc");
    param.latexdiff_opts.args_to(&mut latexdiff);
    param.latexdiffvc_opts.args_to(&mut latexdiff);
    latexdiff.args(["-d", &param.diff_dir_name, "--force"]);

    latexdiff.arg(osstr_join(&param.docfile, ".tex"));
    // current_dirからの相対指定でないと失敗する(ここではファイル名のみでOK)
    let start = std::time::Instant::now();
    // current_dirを指定すると、そこにdiffを作成して、渡されたパス文字列をその中に忠実に再現する(すなわち絶対パスは死ぬ)
    let latexdiff_output = latexdiff.current_dir(&param.dir).stdout(Stdio::null()).stderr(Stdio::piped()).output()?;
    println!("latexdiff-vc took {} ms", start.elapsed().as_millis());
    if !latexdiff_output.status.success() {
        return Err(CError::Main(latexdiff_output.stderr));
    }

    // doc.texであればdoc_diffとかになる。
    std::fs::create_dir_all(&param.tmpdir)?;
    std::fs::rename(
        // DIFF_DIR_NAMEが存在していなかった場合も、latexdiff-vcが自動作成する
        param.dir.join(&param.diff_dir_name).join(osstr_join(&param.docfile, ".tex")),
        param.tmpdir.join(osstr_join(&stem_diff, ".tex")),
    )?; // とりあえずさっさと移動。

    // ここでは一時的にparam.dir.join(DIFF_DIR_NAME)をちゃんと作成してそれを参照しているコードとして解釈されており、問題はない
    // 実際にダングリング参照になる場合はRustコンパイラが警告を出すが、今回はそうなっていない
    let latexmk = LaTeXMK {
        dir: &param.tmpdir,
        docfile: &stem_diff,
        tmpdir: &param.tmpdir,
        outdir: &param.dir.join(&param.diff_dir_name),
        opts: &param.latexmk_opts,
    };
    let latexmk_output = latexmk.command()?.stdout(Stdio::null()).stderr(Stdio::piped()).output()?;
    if !latexmk_output.status.success() {
        return Err(CError::Main(latexmk_output.stderr));
    }
    latexmk.rename_pdf()?;
    Ok(())
}
