-- Optional project config: load deliberately; no plugins are downloaded.
vim.opt.expandtab = true
vim.opt.shiftwidth = 4
vim.opt.tabstop = 4
vim.opt.colorcolumn = "100"
vim.api.nvim_create_user_command("NightfallCheck", function()
    vim.cmd("terminal sh scripts/ci.sh")
end, {})
