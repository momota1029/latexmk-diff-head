use std::{
    ffi::{OsStr, OsString},
    path::Path,
    process::Command,
};

use crate::{error, osstr_join};

pub struct LatexdiffVc<'a> {
    pub latexdiff_vc: &'a Path,
    pub dir: &'a Path,
    pub docfile: &'a OsStr,
    pub diff_dir_name: &'a str,
    pub verbose: bool,
    pub opts: &'a Opts,
    pub latexdiff_opts: &'a super::latexdiff::Opts,
    pub tmpdir: &'a Path,
    pub diff_docfile: &'a OsStr,
}
impl LatexdiffVc<'_> {
    pub fn command(&self) -> Command {
        let mut latexdiff = Command::new(&self.latexdiff_vc);
        self.latexdiff_opts.args_to(self.verbose, &mut latexdiff);
        self.opts.args_to(&mut latexdiff);
        latexdiff.args(["-d", &self.diff_dir_name, "--force"]);
        // current_dirからの相対指定でないとdiffフォルダに入れるのに失敗する(ここではファイル名のみでOK)
        latexdiff.arg(OsString::from_iter([self.docfile, OsStr::new(".tex")])).current_dir(&self.dir);
        latexdiff
    }
    pub fn rename_tex(self) -> error::Result<()> {
        error::create_dir_all(&self.tmpdir)?;
        error::rename(
            // DIFF_DIR_NAMEが存在していなかった場合も、latexdiff-vcが自動作成する
            self.dir.join(&self.diff_dir_name).join(osstr_join(&self.docfile, ".tex")),
            // doc.texであればdoc_diff.texとかになる。
            self.tmpdir.join(osstr_join(&self.diff_docfile, ".tex")),
        )?; // とりあえずさっさと移動。
        Ok(())
    }
}

/// Configuration options for latexdiff-vc command
#[derive(clap::Args, Debug)]
pub struct Opts {
    /// Use Git for version control operations
    #[clap(long, group = "vcs")]
    pub git: bool,

    /// Use Subversion (SVN) for version control operations
    #[clap(long, group = "vcs")]
    pub svn: bool,

    /// Use Mercurial (Hg) for version control operations
    #[clap(long, group = "vcs")]
    pub hg: bool,

    /// Use CVS for version control operations
    #[clap(long, group = "vcs")]
    pub cvs: bool,

    /// Use RCS for version control operations
    #[clap(long, group = "vcs")]
    pub rcs: bool,

    /// Specify revision(s) for comparison [default: HEAD vs working copy]
    #[clap(long, short)]
    pub revision: Vec<String>,

    /// Flatten document by expanding \input and \include commands
    #[clap(long, group = "flat")]
    pub flatten: bool,

    /// Flatten document and keep intermediate files for debugging
    #[clap(long, group = "flat")]
    pub flatten_keep_intermediate: bool,

    /// Use latexdiff-fast
    #[clap(long, group = "execution")]
    fast: bool,

    /// Use latexdiff-so
    #[clap(long, group = "execution")]
    so: bool,

    /// Only show pages with changes
    #[clap(long)]
    only_changes: bool,
}
impl Opts {
    pub fn args_to(&self, cmd: &mut Command) {
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
        if self.revision.is_empty() {
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
        if self.fast {
            cmd.arg("--fast");
        }
        if self.so {
            cmd.arg("--so");
        }
        if self.only_changes {
            cmd.arg("--only-changes");
        }
    }
}
