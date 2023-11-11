# yact

Yet another commit transformer, and the forbidden fruit of your development process.

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

