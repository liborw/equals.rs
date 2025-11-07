# equals.nvim

A lightweight local Neovim plugin that runs [`equals.rs`](../..) on the current buffer and highlights inline result markers. It is meant to live inside this repository (e.g. for `lazy.nvim`'s `dir` option) rather than as a standalone plugin.

## Features
- Saves the current buffer to a temporary file, runs the `equals` CLI, and replaces the buffer with the evaluated output.
- Maps Neovim `filetype`s to `equals` languages (Python/Numbat by default) and toggles `--markdown` automatically for Markdown buffers.
- Provides a `:Equals` user command with an optional language override (`:Equals numbat`).
- Adds default highlights for the `#=` marker and the resulting value (`EqualsMarker` / `EqualsResult` highlight groups).

## Requirements
- Neovim 0.8+ (uses `vim.system` when available; falls back to `vim.fn.system`).
- A built `equals` binary on your `$PATH` or a custom path configured via `cmd`:
  ```bash
  cargo build --release
  # or cargo install --path .
  ```

## Installation

### lazy.nvim

```lua
{
  dir = "~/projects/equals/editors/neovim",
  config = function()
    require("equals").setup({
      cmd = "/home/user/.cargo/bin/equals", -- optional override
    })
  end,
}
```

### packer.nvim

```lua
use({
  "~/projects/equals/editors/neovim",
  config = function()
    require("equals").setup()
  end,
})
```

> Replace the `dir` with the absolute path to this repository on your machine.

## Usage

- Run `:Equals` inside any buffer that contains `#=` markers to evaluate it via `equals.rs`.
- Use `:Equals numbat` (or any other language string) to override the detected language for a single invocation.
- The command works with unsaved buffers—the plugin writes a temporary file under the hood and updates the buffer in place.

## Configuration

```lua
require("equals").setup({
  cmd = "equals",         -- executable to call
  extra_args = {},        -- additional CLI flags appended before file arguments
  language_map = {
    python = "python",
    numbat = "numbat",
  },
  pass_filetype = false,  -- set to true to forward unknown `filetype`s to --language
  markdown_filetypes = { markdown = true, md = true },
  highlight = {
    enable = true,
    filetypes = { "python", "numbat", "markdown", "md", "text" },
    marker_group = "EqualsMarker",
    result_group = "EqualsResult",
  },
})

-- Pass `highlight.filetypes = nil` to highlight every buffer that contains `#=`.
```

Tweak the highlight groups via `:highlight EqualsMarker` / `:highlight EqualsResult` or by linking them to other groups in your colorscheme.

## Notes

- Because the plugin relies on temporary files, the CLI cannot infer the language from the temp file extension. Make sure the `language_map` covers every `filetype` you want to auto-detect (Python and Numbat are provided out of the box).
- Failures are reported through `vim.notify` at the `ERROR` level so you can route them however you like; successful runs stay quiet.
