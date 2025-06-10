use std::{
    borrow::Cow,
    ffi::{OsStr, OsString},
    path::Path,
    process::Command,
};

use crate::error;

// dir+docfileとtepmdirを指定してlatexmkを実行し、pdfとsynctexをoutdirに移動する。
pub struct LaTeXMK<'a> {
    pub latexmk: &'a Path,
    pub dir: &'a Path,
    pub docfile: &'a OsStr,
    pub tmpdir: &'a Path,
    pub outdir: Cow<'a, Path>,
    pub opts: &'a Opts,
}
impl LaTeXMK<'_> {
    pub fn command(&self) -> error::Result<Command> {
        error::create_dir_all(&self.tmpdir)?;
        let mut cmd = Command::new(self.latexmk);
        self.opts.args_to(&mut cmd);
        cmd.args(["-outdir=", "-auxdir="].map(|key| OsString::from_iter([OsStr::new(key), self.tmpdir.as_os_str()])));
        cmd.arg(self.dir.join(&self.docfile));
        Ok(cmd)
    }
    pub fn rename_pdf(self) -> error::Result<()> {
        let pdf_name = OsString::from_iter([self.docfile, OsStr::new(".pdf")]);
        let synctex_name = OsString::from_iter([self.docfile, OsStr::new(".synctex.gz")]);
        error::copy(self.tmpdir.join(&pdf_name), self.outdir.join(pdf_name))?;
        if self.opts.synctex {
            error::copy(self.tmpdir.join(&synctex_name), self.outdir.join(synctex_name))?;
        }
        Ok(())
    }
}

/// Configuration options for latexmk command
#[derive(clap::Args, Debug)]
pub struct Opts {
    /// Use XeLaTeX as the LaTeX engine
    #[clap(long, group = "engine")]
    pub xelatex: bool,

    /// Use LuaLaTeX as the LaTeX engine
    #[clap(long, group = "engine")]
    pub lualatex: bool,

    /// Use BibTeX for bibliography processing
    #[clap(long, group = "bib")]
    pub bibtex: bool,

    /// Use Biber for bibliography processing
    #[clap(long, group = "bib")]
    pub biber: bool,

    /// Disable bibliography processing entirely
    #[clap(long, group = "bib")]
    pub nobibtex: bool,

    /// Enable SyncTeX generation for editor synchronization
    #[clap(long)]
    pub synctex: bool,

    /// Suppress all output except errors
    #[clap(long, group = "output")]
    pub silent: bool,

    /// Reduce output verbosity
    #[clap(long, group = "output")]
    pub quiet: bool,

    /// Increase output verbosity with detailed information
    #[clap(long, group = "output")]
    pub verbose: bool,

    /// Display system commands being executed
    #[clap(long)]
    pub commands: bool,
}
impl Opts {
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
