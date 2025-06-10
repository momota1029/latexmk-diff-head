use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};

use crate::{
    cmd::{
        latexdiff,
        latexdiff_vc::{self, LatexdiffVc},
        latexmk::{self, LaTeXMK},
    },
    error::{self, Error::CurrentDirFailed},
    osstr_join,
};

#[derive(clap::Parser, Debug)]
pub struct Opts {
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
pub struct Param {
    pub dir: PathBuf,      // docの親ディレクトリ
    pub docfile: OsString, // file_stemに相当。拡張子は含まないし、ディレクトリも含まない

    pub tmpdir: PathBuf, // ユーザが指定する。latexmkが.auxファイル類を置いておく`--outdir`。ただし下のoutdirと区別するためにおいてある。
    pub outdir: PathBuf, // latexmkが2024年にout2dir(pdfを出力するディレクトリ)を用意しだしたらしいけど、どうしようもないのでこっちで代替するためのもの

    pub async_diff: bool,
    pub diff_only: bool,

    pub latexmk: PathBuf,
    pub latexdiff_vc: PathBuf,

    pub diff_docfile: OsString,
    pub diff_dir_name: String,

    pub latexdiff_opts: latexdiff::Opts,
    pub latexdiffvc_opts: latexdiff_vc::Opts,
    pub latexmk_opts: latexmk::Opts,
}
impl TryFrom<Opts> for Param {
    type Error = error::Error;
    fn try_from(from: Opts) -> error::Result<Param> {
        // まずは与えられたdocを絶対パスに変換するところから。
        // dir/stem形式なのでパスの部分文字列であり、本当のパスではないことに注意
        let doc = if from.doc.is_absolute() {
            from.doc
        } else {
            // 相対パスならカレントディレクトリ込みの絶対パスに変換
            Path::join(&std::env::current_dir().map_err(CurrentDirFailed)?, &from.doc)
        };
        // その親ディレクトリと名前を改めて取得する。unwrapしても問題はない(絶対パスなので)
        let dir = doc.parent().unwrap().to_path_buf();
        // doc.texを間違って指定してしまった場合を除外しようとするとdoc.tex.texが実在した場合に至極面倒。
        let docfile = doc.file_name().unwrap().to_os_string();

        // diffの処理など。
        let diff_dir_name = from.diff_name.unwrap_or_else(|| "diff".to_string());
        let diff_postfix = from.diff_postfix.unwrap_or_else(|| format!("-{diff_dir_name}"));
        let diff_docfile = osstr_join(&docfile, &diff_postfix);

        let tmpdir = match from.tmpdir {
            None => dir.join(".temp"),
            Some(p) => {
                error::create_dir_all(&p)?;
                error::canonicalize(p)?
            }
        };
        let outdir = match from.outdir {
            None => dir.to_path_buf(),
            Some(p) => {
                error::create_dir_all(&p)?;
                error::canonicalize(p)?
            }
        };
        let latexmk = from.latexmk.unwrap_or_else(|| "latexmk".into());
        let latexdiff_vc = from.latexdiff_vc.unwrap_or_else(|| "latexdiff-vc".into());
        Ok(Param {
            dir,
            diff_docfile,
            docfile,
            tmpdir,
            outdir,
            diff_dir_name,
            latexmk,
            latexdiff_vc,
            async_diff: from.async_diff,
            diff_only: from.diff_only,
            latexmk_opts: from.latexmk_opts,
            latexdiff_opts: from.latexdiff_opts,
            latexdiffvc_opts: from.latexdiffvc_ops,
        })
    }
}
impl Param {
    pub fn latexmk(&self) -> LaTeXMK {
        let Param { latexmk, dir, docfile, tmpdir, outdir, latexmk_opts, .. } = self;
        LaTeXMK { latexmk, dir, docfile, tmpdir, outdir: outdir.into(), opts: latexmk_opts }
    }
    pub fn latexmk_for_diff<'a>(&'a self) -> LaTeXMK<'a> {
        let Param { latexmk, tmpdir, diff_docfile, latexmk_opts, .. } = self;
        let outdir = self.dir.join(&self.diff_dir_name).into();
        LaTeXMK { latexmk, dir: tmpdir, docfile: diff_docfile, tmpdir, outdir, opts: latexmk_opts }
    }
    pub fn latexdiff_vc<'a>(&'a self) -> LatexdiffVc<'a> {
        let Param { latexdiff_vc, dir, docfile, diff_dir_name, tmpdir, latexdiff_opts, latexdiffvc_opts: opts, diff_docfile, .. } = self;
        let verbose = self.latexmk_opts.verbose;
        LatexdiffVc { latexdiff_vc, dir, docfile, diff_dir_name, verbose, opts, latexdiff_opts, tmpdir, diff_docfile }
    }
}
