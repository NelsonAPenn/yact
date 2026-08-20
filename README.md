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

For more information on how yact works, see [this page](docs/how_it_works.md).

## Requirements and installation

1. [Install Rust](https://www.rust-lang.org/tools/install)
2. `cargo install yact`
3. Set up desired code formatters: `yact` does not manage formatting tools. This
   entails that any desired external formatters must be installed and on the
   system path (or configured to be run from the workspace such as in the case
   of a virtual environment).

## Usage (on a per-repository basis)

1. Create a config file named `yactrc.toml` in the workspace root. The
   `docs/config_samples` folder contains example config files that will apply to
   most people. Note that the configuration file is dependent on what languages
   and formatters your team prefers, so these may require modification.

- [Rust sample](docs/config_samples/rust.toml)
- [C and C++ sample](docs/config_samples/c_cpp.toml)
- [Web development sample](docs/config_samples/web.toml)
- [Python sample](docs/config_samples/python.toml)
- [Go sample](docs/config_samples/go.toml)

2. Update your pre-commit git hook to run yact. In the simplest case (where no
   pre-commit hooks have been configured yet), run

```sh
yact init
```

This simply symlinks or hardlinks the yact executable into
`[repository root]/.git/hooks/pre-commit`. If you already have pre-commit
scripts set up that you'd like to keep, add yact to your script manually. Due to
fallability of performing such an update automatically, these cases are left up
to the user.

**The configuration is now complete. Committed changes will be formatted
transparently.**

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
transformers = [ { External = { System = { command = "rustfmt", env = {}, args = ["--emit", "stdout"] }}}]
```

## Considerations

`yact` will never bork your git history. However, `yact` will take liberty in
modifying your working tree as it sees fit. This is done in a fairly safe
manner, merging formatting changes back into your worktree but keeping the
worktree's version in case of conflicts.
