# latexmk-diff-head

A latexmk wrapper that automatically generates diff PDFs against the previous Git commit alongside regular compilation. Designed for LaTeX Workshop integration.

[日本語のREADMEはこちら](README_JA.md)

## What it does

1. Compiles your LaTeX document normally with latexmk
2. Concurrently generates a diff LaTeX file using latexdiff-vc against HEAD
3. Typesets the diff file to create a diff PDF with changes highlighted
4. Outputs both regular and diff PDFs

**Primary use case**: Academic paper writing where Git commits represent submission milestones, enabling clear visualization of changes for revision tracking.

## Installation

### Prerequisites

- Rust (cargo)
- latexmk
- latexdiff (included in most LaTeX distributions)
- Git
- VS Code + LaTeX Workshop (for integration)

### Install

```bash
cargo install latexmk-diff-head
```

Or build from source:

```bash
git clone https://github.com/momota1029/latexmk-diff-head.git
cd latexmk-diff-head
cargo install --path .
```

## Usage

### Command Line

```bash
# Basic usage (recommended)
latexmk-diff-head --synctex --flatten paper/main

# Output files:
# paper/main.pdf (regular)
# paper/diff/main-diff.pdf (diff with additions in blue, deletions in red)
```

### VS Code + LaTeX Workshop

Add to your `settings.json`:

```json
{
  "latex-workshop.latex.recipes": [
    {
      "name": "latexmk-diff-head",
      "tools": ["latexmk-diff-head"]
    }
  ],
  "latex-workshop.latex.tools": [
    {
      "name": "latexmk-diff-head",
      "command": "latexmk-diff-head",
      "args": ["--synctex", "--flatten", "%DOC%"]
    }
  ]
}
```

## Options
Run `latexmk-diff-head -h` for complete option list. Key options:

```bash
--synctex          # Generate SyncTeX (required for LaTeX Workshop)
--flatten          # Expand \input/\include (recommended for complex projects)
--xelatex          # Use XeLaTeX
--lualatex         # Use LuaLaTeX
--bibtex           # Use BibTeX
--biber            # Use Biber (BibLaTeX)
--revision REV     # Compare against specific revision [default: HEAD]
--tmpdir DIR       # Temporary files directory
--outdir DIR       # PDF output directory
```

## Git Workflow

Initialize your project:
```bash
git init
git add .
git commit -m "Initial draft"
```

After making changes and building, you'll see diffs against this commit. Commit periodically to update the comparison baseline.

## License
MIT