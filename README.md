# yact

Yet another commit transformer-- but this one is different than the rest!

Enter the forbidden fruit of your development process.

There are many wonderful tools out there that help you quit spending time
aligning lines of code, remove common mistakes, and sometimes even automatically
make common simplifications to your code. There are also tools that help
integrate these tools with your `git` workflow. However, the tools of the latter
class are not always so wonderful. They usually exhibit a couple main classes of
problems.

1. Making you readd your changes. This is just annoying.
2. Not playing nice in cases when some changes are staged and some are unstaged
   and still in progress. Some tools result in borkage; others make you
   `git add -p` twice, which is again, just annoying.

`yact` is a standalone binary which operates on low-level git objects directly,
formatting staged changes behind the scenes without ever pushing the onus back
on you, and updating your working tree in the most correct way possible.

## Features

- Provides a general method for seamlessly applying transformations to committed text files.
- Plays nice with files with some staged and some unstaged changes.
- Provides a one-file transform command that can be used in other pre-commit hook management tools.
- Provides a replacement for other pre-commit management tools.

## Considerations

`yact` is focused on (in order):

1. Doing a really good job linting automatically with minimal disturbance to your workflow (better than other solutions)
2. Correctness. Working with git objects right.
3. Effectiveness / efficiency

It is not focused on:

- Preserving your worktree exactly as it was.

`yact` could be considered dangerous by some, but would be considered convenient by most. `yact` will never bork your git history. However, `yact` will take liberty in modifying your working tree as it sees fit.

## How it works

`yact` dives into git plumbing to manage staged changes as perfectly as it can. It uses bindings to `libgit2` to do things right.

General flow:

1. (If used as pre-commit hook management replacement) iterate diff and find the right transformer for each file.
2. Create new blob as transformation of staged blob
3. Diff new blob and work tree.
4. Merge diff into worktree.
5. (If used as a pre-commit hook management replacement) create new tree and bump commit to point to new tree.

