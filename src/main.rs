mod cmd;
pub mod error;
pub mod param;

use crate::{
    cmd::latexmk::LaTeXMK,
    error::{Error, Result},
    param::{Opts, Param},
};
use bstr::io::BufReadExt as _;
use clap::Parser;
use std::{
    ffi::{OsStr, OsString},
    io::{BufReader, Write as _},
    process::{Child, ChildStdout, Command, Stdio},
};

const APPLYING_RULE_PAT: &[u8] = b"Latexmk: applying rule ";
const ALL_TARGETS_PAT: &[u8] = b"Latexmk: All targets ";

fn main() {
    std::process::exit(main2().unwrap_or_else(Error::print_and_exit).unwrap_or(1))
}

fn main2() -> error::Result<Option<i32>> {
    let param = Param::try_from(Opts::parse())?;
    if param.diff_only {
        return diffmk(&param);
    }
    let latexmk = param.latexmk(); // 普通のlatexmk
    let (enable_typeset, mut latexmk_out, mut latexmk_spawn) = latexmk_and_sure_typeset(&latexmk)?;
    let mut stdout = std::io::stdout();

    // タイプセットが行われ、同期的にdiffを取る必要がある場合
    if enable_typeset && !param.async_diff {
        // そうでない場合、別スレッドで標準出力だけ横流しする。エラーの出ようがないので無視。
        let _ = std::thread::spawn(move || std::io::copy(&mut latexmk_out, &mut stdout));
        let diff_res = diffmk(&param); // latexdiff-vcからタイプセットまでを実行する
        // diffmkは`*_diff`について作業(`*_diff.aux`などを生成)し、`mk`は`*`について作業する(`*.aux`などを生成する)ため、生成ファイルやその処理が全く被らないことに注意(関係ないファイルを上書きすることはあるが、実行時のエラーになるわけではない)

        // メインのlatexmkが成功しなかったらその場で失敗する
        let mk_status = latexmk_spawn.wait().map_err(error::Error::CommandFailed)?;
        if !mk_status.success() {
            Err(Error::AlreadySaid)?;
        }
        latexmk.rename_pdf()?;
        diff_res?;
        return Ok(mk_status.code());
    }

    // 非同期でdiffを取る場合の対応
    if param.async_diff {
        // asyncでdiffを取るやつは「diff-onlyな自分を無責任に呼ぶ」とする。
        Command::new(std::env::current_exe().map_err(Error::EnvError)?)
            .arg("--diff-only")
            .args(std::env::args_os().skip(1))
            .spawn()
            .map_err(error::Error::CommandFailed)?;
    }

    // 出力を素通りさせる
    std::io::copy(&mut latexmk_out, &mut stdout).map_err(error::Error::StdIoError)?;
    let mk_status = latexmk_spawn.wait().map_err(error::Error::CommandFailed)?;
    if !mk_status.success() {
        Err(Error::AlreadySaid)?;
    }
    latexmk.rename_pdf()?;
    if param.latexmk_opts.synctex {
        // この場合LaTeX WorkshopがSyncTeX位置反映を怠るので、擬似的に出力があったということにしておく
        println!("Output written on dummy.pdf (for LaTeX Workshop's SyncTeX refresh on {:?}).", param.docfile);
    }
    Ok(mk_status.code())
}

fn osstr_join(path: impl AsRef<OsStr>, ext: &str) -> OsString {
    OsString::from_iter([path.as_ref(), OsStr::new(ext)])
}

fn latexmk_and_sure_typeset(latexmk: &LaTeXMK) -> Result<(bool, ChildStdout, Child)> {
    let mut latexmk_spawn =
        latexmk.command()?.stdout(Stdio::piped()).stderr(Stdio::inherit()).spawn().map_err(error::Error::CommandFailed)?;
    let mut latexmk_stdout = latexmk_spawn.stdout.take().expect("handle present"); // 出力がないってことは無いだろ……
    let mut stdio = std::io::stdout();
    let mut enable_typeset = false;
    // typesetが行われたかどうかを判定しながら出力を素通りさせる
    BufReader::new(&mut latexmk_stdout)
        .for_byte_line_with_terminator(|line| {
            stdio.write_all(line)?;
            Ok(if line.starts_with(APPLYING_RULE_PAT) {
                enable_typeset = true;
                false
            } else {
                !line.starts_with(ALL_TARGETS_PAT)
            })
        })
        .map_err(error::Error::StdIoError)?;
    Ok((enable_typeset, latexmk_stdout, latexmk_spawn))
}

fn diffmk(param: &Param) -> error::Result<Option<i32>> {
    // doc.texであればdoc_diff.texとかになる。
    let latexdiff_vc = param.latexdiff_vc();
    let latexdiff_vc_code = cmd_for_diff(latexdiff_vc.command(), param.diff_only)?;
    latexdiff_vc.rename_tex()?;
    if latexdiff_vc_code != Some(0) {
        return Ok(latexdiff_vc_code);
    }

    // ここでは一時的にparam.dir.join(DIFF_DIR_NAME)をちゃんと作成してそれを参照しているコードとして解釈されており、問題はない
    // 実際にダングリング参照になる場合はRustコンパイラが警告を出すが、今回はそうなっていない
    let latexmk = param.latexmk_for_diff();
    let latexmk_code = cmd_for_diff(latexmk.command()?, param.diff_only)?;
    latexmk.rename_pdf()?;
    Ok(latexmk_code)
}

fn cmd_for_diff(mut cmd: Command, diff_only: bool) -> Result<Option<i32>> {
    if diff_only {
        let output = cmd.output().map_err(Error::CommandFailed)?;
        if !output.status.success() {
            return Err(Error::AlreadySaid);
        }
        Ok(output.status.code())
    } else {
        let output = cmd.stdout(Stdio::null()).stderr(Stdio::piped()).output().map_err(Error::CommandFailed)?;
        if !output.status.success() {
            return Err(Error::StdErr(output.stderr));
        }
        Ok(output.status.code())
    }
}
