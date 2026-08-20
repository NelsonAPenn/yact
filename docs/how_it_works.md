## How `yact` works

`yact` dives into git plumbing to manage staged changes as perfectly as it can.
It uses bindings to `libgit2` in order to most closely align with git behavior.

General flow:

1. (If used as pre-commit hook management replacement) iterate diff and find the
   right transformer for each file.
2. Create new blob as transformation of staged blob
3. Diff new blob and work tree.
4. Merge diff into worktree.
5. (If used as a pre-commit hook management replacement) create new tree and
   bump commit to point to new tree.
