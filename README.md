# yact

Yet another commit transformer-- but this one will finally help you truly forget
about formatting!

`yact` is a tool that applies code formatters / prettifiers to your staged
changes, seamlessly updating what you commit. Additionally, it works well when
staging only some changes within a file-- it merges the formatting changes back
into your worktree (accepting the worktree version in case of a conflict). It's
like another programmer cleaning up your commits behind the scenes!

`yact` is designed to avoid the following issues that plague other popular
auto-formatters / auto-formatting strategies:

1. Putting formatting changes in your worktree, aborting the commit, and forcing
   you to add back the formatting updates to whatever you staged. This is just
   annoying.
2. Not playing nice in cases when some changes are staged and some are unstaged
   and still in progress. Some tools result in borkage; others make you
   `git add -p` twice, which is again, just annoying.
3. Format on write can sometimes be jarring. For example, when writing a new
   function and then saving, the function will be reduced to a minimal form (ex.
   `int main(){}`).

## Requirements and installation

- This crate depends on [libgit2](https://libgit2.org/), but this should be
  built automatically if it is not installed on the target system.
- For the time being, build and install `yact` from source:
  1. [Install Rust](https://www.rust-lang.org/tools/install)
  2. `cargo install --git https://github.com/NelsonAPenn/yact --bin yact`
- `yact` does not install formatting tools for you. This does require that any
  desired external formatters be installed and on the system path.

## Usage

1. Create a config file named `yactrc.toml` in the workspace root. Below are
   some example config files that will apply to most people. Note that the
   configuration file is dependent on what languages and formatters your team
   prefers, so these may require modification.

```toml
# Configuration file for yact. See https://github.com/NelsonAPenn/yact for more
# information.
#
# Created from the Rust sample config file.

# Note if the Rust edition you are using is later than 2015, you may need to
# create a .rustfmt.toml file in the workspace root specifying the edition.
[[items]]
glob = "**/*.rs"
transformers = [{ External = "Rustfmt" }]

[[items]]
glob = "**/*.md"
transformers = [{ Builtin = "TrailingWhitespace" }]
```

```toml
# Configuration file for yact. See https://github.com/NelsonAPenn/yact for more
# information.
#
# Created from the C / C++ sample config file.
items = [
    { glob = "**/*.cpp", transformers = [{External = "ClangFormat"}] },
    { glob = "**/*.hpp", transformers = [{External = "ClangFormat"}] },
    { glob = "**/*.h", transformers = [{External = "ClangFormat"}] },
    { glob = "**/*.c", transformers = [{External = "ClangFormat"}] },

    # clang-format also works for JSON. If it's already installed, may as well
    # use it rather than configuring a different tool.
    { glob = "**/*.json", transformers = [{External = "ClangFormat"}] },
   
    { glob = "**/*.md", transformers = [{ Builtin = "TrailingWhitespace" }]},
]
```

```toml
# Configuration file for yact. See https://github.com/NelsonAPenn/yact for more
# information.
#
# Created from the web development sample config file.

# "Npm" can be replaced with "Yarn", or the field can be removed entirely to
# use a global installation of prettier.
items = [
    { glob = "**/*.js", transformers = [{External = { Prettier = { package_manager_type = "Npm" } }}] },
    { glob = "**/*.ts", transformers = [{External = { Prettier = { package_manager_type = "Npm" }}}] },
    { glob = "**/*.jsx", transformers = [{External = { Prettier = { package_manager_type = "Npm" }}}] },
    { glob = "**/*.tsx", transformers = [{External = { Prettier = { package_manager_type = "Npm" }}}] },
    { glob = "**/*.html", transformers = [{External = { Prettier = { package_manager_type = "Npm" }}}] },
    { glob = "**/*.json", transformers = [{External = { Prettier = { package_manager_type = "Npm" }}}] },
    { glob = "**/*.md", transformers = [{External = { Prettier = { package_manager_type = "Npm" }}}] },
]
```

```toml
# Configuration file for yact. See https://github.com/NelsonAPenn/yact for more
# information.
#
# Created from the Python sample config file.

# To use a virtual environment, run `git commit` from the activated environment
# or provide the virtual environment path (recommended to be within repository
# root), for example `{ RuffFormat = { venv_path = ".venv" }}`
items = [
    { glob = "**/*.py", transformers = [{ External = { RuffFormat = {}}}] },
    { glob = "**/*.pyi", transformers = [{ External = { RuffFormat = {}}}] },

    { glob = "**/*.md", transformers = [{ Builtin = "TrailingWhitespace" }]},
]
```

```toml
# Configuration file for yact. See https://github.com/NelsonAPenn/yact for more
# information.
#
# Created from the Golang sample config file.
[[items]]
glob = "**/*.go"
transformers = [{ External = "Gofmt" }]

[[items]]
glob = "**/*.md"
transformers = [{ Builtin = "TrailingWhitespace" }]
```

2. Update your pre-commit git hook to run yact. For example, on Unix systems,
   run the following command from the root of the repository you'd like to use
   yact in.

```sh
ln -s "$(which yact)" .git/hooks/pre-commit
```

## Transformers

`yact` defines a transformer as any process that applies some sort of formatting
to a file. `yact` includes builtin transformers (written in native Rust) and
transformers that invoke another process.

The standard interface for transformers that are a separate process are a
command that reads a source file in from stdin and writes the formatter version
to stdout, returning a nonzero exit code if the operation failed.

`yact` has the following builtin transformers:

- `TrailingWhitespace`: trims trailing whitespace and ensures the source file
  ends in a newline.

Additionally, `yact` has options for the following popular transformers (simply
providing the correct command-line arguments to them):

- `Rustfmt`
- `ClangFormat`
- `DenoFmt`
- `Prettier`
- `RuffFormat`
- `Gofmt`

Finally, `yact` provides a catch-all `System` transformer where command, env,
and args can be configured. Example below.

```toml
[[items]]
glob = "**/*.rs"
transformers = [ { System = { command = "rustfmt", env = {}, args = ["--emit", "stdout"] }}]
```

## Considerations

`yact` will never bork your git history. However, `yact` will take liberty in
modifying your working tree as it sees fit. This is done in a fairly safe
manner, merging formatting changes back into your worktree but keeping the
worktree's version in case of conflicts.

## How it works

`yact` dives into git plumbing to manage staged changes as perfectly as it can.
It uses bindings to `libgit2` to do things right.

General flow:

1. (If used as pre-commit hook management replacement) iterate diff and find the
   right transformer for each file.
2. Create new blob as transformation of staged blob
3. Diff new blob and work tree.
4. Merge diff into worktree.
5. (If used as a pre-commit hook management replacement) create new tree and
   bump commit to point to new tree.
