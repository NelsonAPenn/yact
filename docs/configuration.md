# Configuration file

The configuration file consists an optional version requirement on yact itself
followed by formatting configuration items.

An example version requirement can be found below:

```toml
requires_yact_version = ">=2.0.0, <3.0.0"
```

Each item specifies a glob specifying what files to format and a set of
transformers to apply to them:

```toml
[[items]]
glob = "**/*example/*glob*.extension"
transformers = []
```

## Builtin transformers

### Trailing whitespace trimmer

```toml
{Builtin = "TrailingWhitespace"}
```

## External transformers

If specific configuration is desired, configure with a `.rustfmt.toml` file.
More than likely the edition at least will need to be configured in this way.

### Rustfmt

```toml
{External = "Rustfmt"}
```

### Gofmt

```toml
{External = "Gofmt"}
```

### Clang format

If specific configuration is desired, configure with a `.clang-format` file.

```toml
{External = "ClangFormat"}
```

### Deno fmt

```toml
{External = "DenoFmt"}
```

### Prettier

```toml
{ External = { Prettier = { package_json_directory = "web/projects/example/", package_manager_type = "Bun" }}}
```

| Key                    | Required / Optional | Description                                                                                                                 |
| ---------------------- | ------------------- | --------------------------------------------------------------------------------------------------------------------------- |
| package_json_directory | optional            | Directory containing package.json, if different from repository root                                                        |
| package_manager_type   | optional            | Package manager to use to run workspace version of `prettier`; if not provided, searches for `prettier` on the system path. |

### Ruff lint

Configurable Ruff linter autofixes.

```toml
{ External = { RuffLint = { behavior = "", venv_path = ".venv", unsafe_fixes = true }}}
```

| Key          | Required / Optional | Description                                                                                                                          |
| ------------ | ------------------- | ------------------------------------------------------------------------------------------------------------------------------------ |
| behavior     | required            | "CheckOnly", "CheckAndFix", or "FixOnly"                                                                                             |
| venv_path    | optional            | Directory of virtual environment to use to run workspace version of `ruff`; if not provided, searches for `ruff` on the system path. |
| unsafe_fixes | optional            | Use `ruff`'s unsafe fixes option. Defaults to false                                                                                  |

### Ruff format

The configuration for `ruff` allows specifying a `venv_path` which allows
running the virtual environment's installed version of `ruff`.

```toml
{ External = { RuffFormat = { venv_path = ".venv" }}}
```

| Key       | Required / Optional | Description                                                                                                                          |
| --------- | ------------------- | ------------------------------------------------------------------------------------------------------------------------------------ |
| venv_path | optional            | Directory of virtual environment to use to run workspace version of `ruff`; if not provided, searches for `ruff` on the system path. |

### System formatter

A generic transformer that can be used to configure any formatter not already
supported.

```toml
{ External = { System = { command = "bash", env = {}, args = ["-c", "csharpier format"] }}}
```

| Key     | Required / Optional | Description                                   |
| ------- | ------------------- | --------------------------------------------- |
| command | required            | Command to run                                |
| env     | required            | Environment variables to run the command with |
| args    | required            | Command line arguments for the command        |
