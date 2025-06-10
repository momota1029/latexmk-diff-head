mod cmd;

use bstr::io::BufReadExt as _;
use clap::Parser;
use std::{
    ffi::{OsStr, OsString},
    io::{self, BufReader, Write as _},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};
use thiserror::Error;

use crate::cmd::{
    latexdiff,
    latexdiff_vc::{self, LatexdiffVc},
    latexmk::{self, LaTeXMK},
};

const APPLYING_RULE_PAT: &[u8] = b"Latexmk: applying rule ";
const ALL_TARGETS_PAT: &[u8] = b"Latexmk: All targets ";

fn main() -> io::Result<()> {
    let param = Opts::parse().to_param()?;
    if param.diff_only {
        return match diffmk(&param) {
            Ok(()) => Ok(()),
            Err(CError::Io(err)) => {
                eprintln!("{}", err);
                std::process::exit(1);
            }
            Err(CError::Main(err)) => {
                io::stderr().write_all(&err)?;
                std::process::exit(1);
            }
        };
    }

    let latexmk = param.latexmk();
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
    if !enable_typeset || param.async_diff {
        if enable_typeset && param.async_diff {
            // asyncでdiffを取るやつは「diff-onlyな自分を無責任に呼ぶ」とする。
            Command::new(std::env::current_exe()?).arg("--diff-only").args(std::env::args_os().skip(1)).spawn()?;
        }
        std::io::copy(&mut latexmk_stdout, &mut stdio)?;
        let mk_status = latexmk_spawn.wait()?;
        if mk_status.success() {
            latexmk.rename_pdf()?;
        }
        if param.latexmk_opts.synctex {
            // この場合LaTeX WorkshopがSyncTeX位置反映を怠るので、擬似的に出力があったということにしておく
            println!("Output written on dummy.pdf (for LaTeX Workshop's SyncTeX refresh on {:?}).", param.docfile);
        }
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

    /// Path to latexmk executable [default: "latexmk"]
    #[clap(long, value_parser)]
    latexmk: Option<PathBuf>,

    /// Path to latexdiff-vc executable [default: "latexdiff-vc"]
    #[clap(long, value_parser)]
    latexdiff_vc: Option<PathBuf>,

    /// Only generate diff output
    #[clap(long, value_parser, hide = true)]
    diff_only: bool,

    /// Use async latexdiff-vc to generate diff
    #[clap(long, value_parser)]
    async_diff: bool,

    /// Name of subdirectory for diff output [default: "diff"]
    #[clap(long, short, value_parser)]
    diff_name: Option<String>,
    /// Suffix added to diff filename [default: "-diff"]
    #[clap(long, value_parser)]
    diff_postfix: Option<String>,

    #[clap(flatten)]
    latexmk_opts: latexmk::Opts,
    #[clap(flatten)]
    latexdiff_opts: latexdiff::Opts,
    #[clap(flatten)]
    latexdiffvc_ops: latexdiff_vc::Opts,
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
            latexmk: self.latexmk.unwrap_or_else(|| "latexmk".into()),
            latexdiff_vc: self.latexdiff_vc.unwrap_or_else(|| "latexdiff-vc".into()),
            async_diff: self.async_diff,
            diff_only: self.diff_only,
            latexmk_opts: self.latexmk_opts,
            latexdiff_opts: self.latexdiff_opts,
            latexdiffvc_opts: self.latexdiffvc_ops,
        })
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
    OsString::from_iter([path.as_ref(), OsStr::new(ext)])
}

struct Param {
    dir: PathBuf,      // docの親ディレクトリ
    docfile: OsString, // file_stemに相当。拡張子は含まないし、ディレクトリも含まない

    tmpdir: PathBuf, // ユーザが指定する。latexmkが.auxファイル類を置いておく`--outdir`。ただし下のoutdirと区別するためにおいてある。
    outdir: PathBuf, // latexmkが2024年にout2dir(pdfを出力するディレクトリ)を用意しだしたらしいけど、どうしようもないのでこっちで代替するためのもの

    async_diff: bool,
    diff_only: bool,

    latexmk: PathBuf,
    latexdiff_vc: PathBuf,

    diff_dir_name: String,
    diff_postfix: String,

    latexdiff_opts: latexdiff::Opts,
    latexdiffvc_opts: latexdiff_vc::Opts,
    latexmk_opts: latexmk::Opts,
}
impl Param {
    fn latexmk(&self) -> LaTeXMK {
        LaTeXMK {
            latexmk: &self.latexmk,
            dir: &self.dir,
            docfile: &self.docfile,
            tmpdir: &self.tmpdir,
            outdir: &self.outdir,
            opts: &self.latexmk_opts,
        }
    }
    fn latexdiff_vc<'a>(&'a self, stem_diff: &'a OsStr) -> LatexdiffVc<'a> {
        LatexdiffVc {
            latexdiff_vc: &self.latexdiff_vc,
            dir: &self.dir,
            docfile: &self.docfile,
            diff_dir_name: &self.diff_dir_name,
            verbose: self.latexmk_opts.verbose,
            opts: &self.latexdiffvc_opts,
            latexdiff_opts: &self.latexdiff_opts,
            tmpdir: &self.tmpdir,
            stem_diff,
        }
    }
}

fn diffmk(param: &Param) -> Result<(), CError> {
    // doc.texであればdoc_diff.texとかになる。
    let stem_diff = osstr_join(&param.docfile, &param.diff_postfix);
    let latexdiff_vc = param.latexdiff_vc(&stem_diff);
    let start = std::time::Instant::now();
    let latexdiff_output = latexdiff_vc.command()?.stdout(Stdio::null()).stderr(Stdio::piped()).output()?;
    println!("latexdiff-vc took {} ms", start.elapsed().as_millis());
    if !latexdiff_output.status.success() {
        return Err(CError::Main(latexdiff_output.stderr));
    }
    latexdiff_vc.rename_tex()?;

    // ここでは一時的にparam.dir.join(DIFF_DIR_NAME)をちゃんと作成してそれを参照しているコードとして解釈されており、問題はない
    // 実際にダングリング参照になる場合はRustコンパイラが警告を出すが、今回はそうなっていない
    let latexmk = LaTeXMK {
        latexmk: &param.latexmk,
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
