use std::process::Command;

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
    }
}
