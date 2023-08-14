# JSPerf LSP

## Build

Simple build
```bash
cargo build
```

Watching
```bash
cargo install cargo-watch
cargo watch -x build
```

## Development

### General
The waxwing lsp can be launched either with `stderr` out or file out.  File out
for easier printf debugging can be achieved by args to the lsp itself.

```
waxwing-lsp -f /path/to/file --level OFF | TRACE | DEBUG | INFO | WARN | ERROR
```

### NeoVim
As of now,
[lsp-debug-tools](https://github.com/ThePrimeagen/lsp-debug-tools.nvim) is the
only debugging tool designed for in editor experience.

