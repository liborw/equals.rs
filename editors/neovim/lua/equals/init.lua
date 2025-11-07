local M = {}

local default_config = {
  cmd = "equals",
  extra_args = {},
  language_map = {
    python = "python",
    python3 = "python",
    numbat = "numbat",
    nb = "numbat",
    nbt = "numbat",
  },
  pass_filetype = false,
  markdown_filetypes = {
    markdown = true,
    md = true,
    rmd = true,
    quarto = true,
  },
  highlight = {
    enable = true,
    filetypes = { "python", "numbat", "markdown", "md", "text" },
    marker_group = "EqualsMarker",
    result_group = "EqualsResult",
  },
}

local config = vim.deepcopy(default_config)
local highlight_group = vim.api.nvim_create_augroup("EqualsHighlight", { clear = true })

local function trim(value)
  if type(value) ~= "string" then
    return ""
  end

  if vim.trim then
    return vim.trim(value)
  end

  return value:gsub("^%s+", ""):gsub("%s+$", "")
end

local function to_set(value)
  if value == nil then
    return nil
  end

  if vim.tbl_islist(value) then
    local set = {}
    for _, item in ipairs(value) do
      if type(item) == "string" then
        set[item] = true
      end
    end
    return set
  end

  return vim.deepcopy(value)
end

local function refresh_cached_tables()
  config._markdown_filetypes = to_set(config.markdown_filetypes)
  config.highlight._filetype_set = to_set(config.highlight.filetypes)
end

local function should_highlight(ft)
  local allowed = config.highlight._filetype_set
  if not allowed or vim.tbl_isempty(allowed) then
    return true
  end
  return allowed[ft] or false
end

local function is_markdown(ft, override)
  if type(override) == "boolean" then
    return override
  end
  local set = config._markdown_filetypes or {}
  return set[ft] or false
end

local function resolve_language(ft, override)
  if type(override) == "string" and override ~= "" then
    return override
  end

  if ft and ft ~= "" then
    if config.language_map[ft] then
      return config.language_map[ft]
    elseif config.pass_filetype then
      return ft
    end
  end
end

local function cleanup(paths)
  for _, path in ipairs(paths) do
    if path and #path > 0 then
      pcall(vim.fn.delete, path)
    end
  end
end

local function run_cli(cmd)
  if vim.system then
    local process = vim.system(cmd, { text = true }):wait()
    return {
      code = process.code or process.exit_code or 0,
      stdout = process.stdout or "",
      stderr = process.stderr or "",
    }
  end

  local stdout = vim.fn.system(cmd)
  local code = vim.v.shell_error
  return {
    code = code,
    stdout = stdout,
    stderr = "",
  }
end

local function apply_highlight(bufnr)
  if not config.highlight.enable then
    return
  end

  if not vim.api.nvim_buf_is_valid(bufnr) or not vim.api.nvim_buf_is_loaded(bufnr) then
    return
  end

  local ft = vim.bo[bufnr].filetype
  if not should_highlight(ft) or vim.b[bufnr].equals_highlighted then
    return
  end

  vim.api.nvim_buf_call(bufnr, function()
    vim.cmd(
      string.format(
        [[syntax match %s /\v#=/ containedin=ALL keepend]],
        config.highlight.marker_group
      )
    )
    vim.cmd(
      string.format(
        [[syntax match %s /\v#=\s*\zs[^#\r\n]*/ containedin=ALL keepend]],
        config.highlight.result_group
      )
    )
  end)

  vim.b[bufnr].equals_highlighted = true
end

local function ensure_highlight_groups()
  vim.api.nvim_set_hl(0, config.highlight.marker_group, { default = true, link = "Define" })
  vim.api.nvim_set_hl(0, config.highlight.result_group, { default = true, link = "String" })
end

local function configure_highlights()
  vim.api.nvim_clear_autocmds({ group = highlight_group })

  if not config.highlight.enable then
    return
  end

  ensure_highlight_groups()

  vim.api.nvim_create_autocmd({ "FileType", "BufWinEnter" }, {
    group = highlight_group,
    callback = function(args)
      apply_highlight(args.buf)
    end,
  })

  -- Apply immediately to the current buffer.
  local ok, bufnr = pcall(vim.api.nvim_get_current_buf)
  if ok and bufnr then
    apply_highlight(bufnr)
  end
end

local function build_command(opts, input_path, output_path)
  local cmd = { config.cmd }

  for _, arg in ipairs(config.extra_args) do
    table.insert(cmd, arg)
  end

  if opts.language then
    table.insert(cmd, "--language")
    table.insert(cmd, opts.language)
  end

  if opts.markdown then
    table.insert(cmd, "--markdown")
  end

  table.insert(cmd, "--input")
  table.insert(cmd, input_path)
  table.insert(cmd, "--output")
  table.insert(cmd, output_path)

  return cmd
end

local function read_buffer(bufnr)
  return vim.api.nvim_buf_get_lines(bufnr, 0, -1, false)
end

local function capture_view(bufnr)
  local winid = vim.fn.bufwinid(bufnr)
  if winid == -1 then
    return nil
  end

  local view
  vim.api.nvim_win_call(winid, function()
    view = vim.fn.winsaveview()
  end)

  return { winid = winid, view = view }
end

local function restore_view(snapshot)
  if not snapshot then
    return
  end

  local winid = snapshot.winid
  if not vim.api.nvim_win_is_valid(winid) then
    return
  end

  vim.api.nvim_win_call(winid, function()
    vim.fn.winrestview(snapshot.view)
  end)
end

local function write_buffer(bufnr, lines)
  if #lines == 0 then
    lines = { "" }
  end

  local snapshot = capture_view(bufnr)
  vim.api.nvim_buf_set_lines(bufnr, 0, -1, false, lines)
  restore_view(snapshot)
end

function M.run(opts)
  opts = opts or {}
  local bufnr = opts.bufnr or vim.api.nvim_get_current_buf()

  if not vim.api.nvim_buf_is_valid(bufnr) or not vim.api.nvim_buf_is_loaded(bufnr) then
    vim.notify("equals.nvim: buffer is not loaded", vim.log.levels.ERROR)
    return
  end

  local ft = opts.filetype or vim.bo[bufnr].filetype or ""
  local language = resolve_language(ft, opts.language)
  local markdown = is_markdown(ft, opts.markdown)

  local buf_lines = read_buffer(bufnr)
  local input_path = vim.fn.tempname()
  local output_path = vim.fn.tempname()

  local ok, write_err = pcall(vim.fn.writefile, buf_lines, input_path)
  if not ok then
    cleanup({ input_path, output_path })
    vim.notify(("equals.nvim: unable to write temp file: %s"):format(write_err), vim.log.levels.ERROR)
    return
  end

  local cmd = build_command({
    language = language,
    markdown = markdown,
  }, input_path, output_path)

  local result = run_cli(cmd)
  local success = result.code == 0

  if success then
    local ok_read, new_lines = pcall(vim.fn.readfile, output_path)
    if not ok_read then
      cleanup({ input_path, output_path })
      vim.notify(
        ("equals.nvim: failed to read equals output: %s"):format(new_lines),
        vim.log.levels.ERROR
      )
      return
    end
    write_buffer(bufnr, new_lines)
  else
    local message = trim(result.stderr)
    if message == "" then
      message = trim(result.stdout)
    end
    if message == "" then
      message = ("equals.nvim: command failed with exit code %d"):format(result.code)
    end
    vim.notify(message, vim.log.levels.ERROR)
  end

  cleanup({ input_path, output_path })
end

function M.setup(opts)
  config = vim.tbl_deep_extend("force", config, opts or {})
  refresh_cached_tables()
  configure_highlights()
end

refresh_cached_tables()
configure_highlights()

return M
