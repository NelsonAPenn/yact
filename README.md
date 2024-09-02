# yact

Yet another commit transformer-- but this one is different than the rest!

Enter the forbidden fruit of your development process.

## Features

`yact` is focused on (in order):

1. Seamlessly applying formatters with minimal disturbance to your workflow or git history (better than other solutions).
    - Transparently transforms staged changes for commit
    - Merges the resulting formatting changes back into working tree
2. Using the first-party implementation (`libgit2`) whenever possible when working with git repositories.
3. Performance
4. Efficiency

`yact` provides both a method for configuring what transformers to run on which files in a project as well as a method for integrating with other pre-commit management tools (like `pre-commit`).

## Usage

1. Create a config file named `.yactrc.toml` in the workspace root.

```toml
# Example .yactrc.toml
items = [
    { pathspec = "**/*.rs", transformers = [ {RawCommand = "Rustfmt"} ]},
    { pathspec = "**/*.md", transformers = [ {Builtin = "TrailingWhitespace" }]},
    { pathspec = "*.md", transformers = [ {Builtin = "TrailingWhitespace"} ]}
]
```

2. Update your pre-commit git hook to run yact. This can be as simple as placing the script below at `.git/hooks/pre-commit`

```sh
#!/bin/sh

yact
```

## Transformers

`yact` defines a transformer as any process that applies some sort of formatting to a file. `yact` includes builtin transformers (written in native Rust) and transformers that invoke another process.

The standard interface for transformers that are a separate process are a command that reads a source file in from stdin and writes the formatter version to stdout, returning a nonzero exit code if the operation failed.

`yact` has the following builtin transformers:

- `TrailingWhitespace`: trims trailing whitespace and ensures the source file ends in a newline.

Additionally, `yact` has options for the following popular transformers (simply providing the correct command-line arguments to them):

- `Rustfmt`

## Why another tool?

There are many wonderful tools out there that help you quit spending time
aligning lines of code, remove common mistakes, and sometimes even automatically
make common simplifications to your code. There are also tools that help
integrate these tools with your `git` workflow. However, the tools of the latter
class usually exhibit a couple main classes of problems.

1. Making you readd your changes. This is just annoying.
2. Not playing nice in cases when some changes are staged and some are unstaged
   and still in progress. Some tools result in borkage; others make you
   `git add -p` twice, which is again, just annoying.

`yact` is a standalone binary which operates on low-level git objects directly,
formatting staged changes behind the scenes without ever pushing the onus back
on you, and updating your working tree in the most correct way possible.

## Considerations

`yact` will never bork your git history. However, `yact` will take liberty in modifying your working tree as it sees fit. This is done in a fairly safe manner, merging formatting changes back into your worktree but keeping the worktree's version in case of conflicts.

## How it works

`yact` dives into git plumbing to manage staged changes as perfectly as it can. It uses bindings to `libgit2` to do things right.

General flow:

1. (If used as pre-commit hook management replacement) iterate diff and find the right transformer for each file.
2. Create new blob as transformation of staged blob
3. Diff new blob and work tree.
4. Merge diff into worktree.
5. (If used as a pre-commit hook management replacement) create new tree and bump commit to point to new tree.

