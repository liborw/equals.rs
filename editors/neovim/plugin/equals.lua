local equals = require("equals")

vim.api.nvim_create_user_command("Equals", function(opts)
  local args = {}
  if opts.args ~= "" then
    args.language = opts.args
  end
  equals.run(args)
end, {
  nargs = "?",
  desc = "Evaluate the current buffer with equals.rs",
})
