# latexmk-diff-head

LaTeX Workshop用のlatexmkラッパーツール。論文執筆時に前回のコミットとの差分を自動生成し、変更箇所を可視化します。

## 概要
このツールは以下の流れで動作します：
1. 通常のlatexmkでPDFを生成
2. 並行してlatexdiff-vcで前回のGitコミット（HEAD）との差分LaTeXファイルを生成
3. 差分ファイルをタイプセットして差分PDFを作成
4. 通常版と差分版の両方のPDFが得られる

**主な用途**：
- 論文執筆で「提出 = Gitコミット」として管理
- 査読コメントへの対応時に変更箇所を明確に示す
- LaTeX Workshopから透明に利用可能

## インストール

### 前提条件

以下がシステムにインストールされている必要があります：
- Rust (cargo)
- latexmk
- latexdiff（多くのLaTeX環境に含まれています）
- Git
- VS Code + LaTeX Workshop拡張

### インストール手順

```bash
# cargoから直接インストール
cargo install latexmk-diff-head
```

または、GitHubからソースをビルド：

```bash
git clone https://github.com/momota1029/latexmk-diff-head.git
cd latexmk-diff-head
cargo install --path .
```

## 日本語LaTeX環境設定

### latexmkrc設定（日本語使用時）

ホームディレクトリの`.latexmkrc`ファイル（`~/.latexmkrc`）を開いて以下を追加：

```perl
$pdf_mode = 3;
$max_repeat = 10;
$latex = 'uplatex %O %S';
$bibtex = 'upbibtex %O %B';
$dvipdf = 'dvipdfmx %O -o %D %S';
```

この設定により、uplatex + dvipdfmxで日本語PDFがタイプセットされます。基本的に動けば問題はないので、お好みの`.latexmkrc`がある場合はそちらを使ってください。

### LaTeX Workshop設定

VS Codeの`settings.json`に以下を追加：

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

これで「LaTeX Workshop: Build with recipe」から`latexmk-diff-head`を選択して使用できます。

## Git初期化&コミット（初心者向け）

VS Codeでプロジェクトフォルダを開いた後、ターミナルで：
```bash
# Gitリポジトリを初期化
git init

# ユーザー情報を設定（初回のみ）
git config user.name "あなたの名前"
git config user.email "your.email@example.com"

# 初回コミット
git add .
git commit -m "初稿"
```

これで差分の基準となるコミットが作成され、次回のビルド時に差分PDFが生成されます。

### 2回目以降のコミット(VSCode)
1. LaTeXファイルを編集
2. サイドバーの「ソース管理」タブを開く
3. 変更ファイルの「+」ボタンでステージング（または変更欄の「+」ボタンでまとめてステージング）
4. コミットメッセージを入力して「コミット」ボタン

これによって、ビルド時の差分生成がこのコミットを基準にして行われるようになります。間違ってガンガンコミットすると差分が見えなくなっちゃうので注意。変更ファイルをちゃんと選んでコミットすると吉。

## 基本的な使用方法・実用例

### 日常的な論文執筆

1. **論文執筆**：
   - LaTeXファイルを編集・保存
   - 自動ビルドまたは手動ビルドで通常版PDFと差分版PDFが生成される
   - 差分PDFで前回コミットからの変更箇所を確認

2. **節目でコミット**：
   - VS Codeの「ソース管理」タブでコミット
   - コミット後は、そのコミットが新しい差分の基準点となる

### 査読対応ワークフロー
1. **論文を査読に提出する際**：
   - 論文を提出した後に「査読提出版」としてコミット（重要！）

2. **査読コメント対応中**：
   - LaTeXファイルを修正・保存
   - ビルドすると「査読提出版」からの差分PDFが自動生成される
   - 査読者に「何を変更したか」を明確に示せる

3. **修正完了後**：
   - VS Codeで「査読コメント対応完了」としてコミット
   - またコメントが来た時はこの状態からの差分を扱える

このワークフローにより、査読者に渡した版からの差分を常に手元で確認できます。

### コマンドラインから実行する場合
`paper/main.tex`をビルドしたい場合は、`.tex`を取り除いて以下の様にコマンドを入れてください:
```bash

# 推奨設定（SyncTeX + flatten）
latexmk-diff-head --synctex --flatten paper/main
```
このときの出力ファイルは
- `paper/main.pdf`：通常版PDF
- `paper/diff/main-diff.pdf`：差分版PDF（追加部分が青、削除部分が赤で表示）
です。お好みで以下のオプションを指定するとよいでしょう。速い方がいいなら`--async-diff`もオススメです。

## オプション一覧
全てのオプションを見たければ`latexmk-diff-head -h`とコマンドを打てば(英語ですが)説明が出てきます。

### 独自オプション

```bash
--async-diff       # 差分タイプセット時のエラーを表示しない代わりに、完全非同期で生成(LaTeX Workshopが差分を待たないでいいので気持ち速く感じる)
--tmpdir DIR       # 一時ファイル用ディレクトリ [default: <doc_dir>/.temp]
--outdir DIR       # PDF出力ディレクトリ [default: 文書と同じディレクトリ]
--diff-name DIR    # 差分ディレクトリ名 [default: "diff"]
--diff-postfix SUF # 差分ファイルの接尾辞 [default: "-diff"]
```

### latexmkオプション

```bash
# LaTeX処理系選択
--xelatex          # XeLaTeX使用
--lualatex         # LuaLaTeX使用

# 参考文献処理
--bibtex           # BibTeX使用
--biber            # Biber使用（BibLaTeX用）
--nobibtex         # 参考文献処理を無効化

# その他
--synctex          # SyncTeX生成（LaTeX Workshopでの位置同期に必須）
--commands         # 実行コマンドを表示
```

### latexdiffオプション

```bash
# 基本機能
--flatten          # \input、\includeを展開（複雑なプロジェクトで推奨）
--revision REV     # 比較対象リビジョン指定 [default: HEAD]

# マークアップスタイル
--type STYLE       # 差分マークアップスタイル
--encoding ENC     # ファイルエンコーディング
--math-markup LVL  # 数式差分の粒度
--graphics-markup MODE # 画像差分の処理方法

# 表示制御
--show-preamble    # 使用プリアンブルを表示
--show-all         # 全設定情報を表示
```

## トラブルシューティング
### よくある問題
**Q: 差分PDFが生成されない**
- Gitリポジトリが初期化されていますか？（`git init`済み？）
  - 少なくとも1回はコミットしていますか？
- latexdiffがインストールされていますか？
  - `~/.cargo/bin/`にPATHが通っていますか？

**Q: LaTeX Workshopで位置同期が効かない**
- `--synctex`オプションが設定に含まれていますか？
- 設定例通りの`args`になっていますか？

**Q: \input/\includeファイルで差分が正しく取れない**
- `--flatten`オプションが設定に含まれていますか？

**Q: 日本語が正しく処理されない**
- `~/.latexmkrc`でuplatex設定を確認してください
- または`--xelatex`/`--lualatex`オプションの使用も検討してみてください

## ライセンス
MIT

## 貢献

バグ報告や機能要求はGitHubのIssueにお願いします。