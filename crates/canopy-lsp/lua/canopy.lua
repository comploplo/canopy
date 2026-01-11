-- Canopy LSP configuration for Neovim 0.11+/0.12
-- Uses the new vim.lsp.config() and vim.lsp.enable() APIs

local M = {}

--- Setup canopy-lsp with the given server path
---@param lsp_path string Path to the canopy-lsp binary
function M.setup(lsp_path)
  -- Define the canopy LSP server configuration
  vim.lsp.config('canopy', {
    cmd = { lsp_path },
    filetypes = { 'text', 'markdown' },
    root_markers = { '.git', '.canopy' },
    settings = {},
  })

  -- Enable the server
  vim.lsp.enable('canopy')

  -- Enable inlay hints globally
  vim.lsp.inlay_hint.enable(true)

  -- Set up keymaps for LSP features
  vim.api.nvim_create_autocmd('LspAttach', {
    callback = function(args)
      local client = vim.lsp.get_client_by_id(args.data.client_id)
      if client and client.name == 'canopy' then
        local opts = { buffer = args.buf }

        -- Hover: show semantic information
        vim.keymap.set('n', 'K', vim.lsp.buf.hover, opts)

        -- Code actions: quick fixes
        vim.keymap.set('n', '<leader>ca', vim.lsp.buf.code_action, opts)

        -- Toggle inlay hints
        vim.keymap.set('n', '<leader>ih', function()
          vim.lsp.inlay_hint.enable(not vim.lsp.inlay_hint.is_enabled())
        end, opts)

        -- Show diagnostics
        vim.keymap.set('n', '<leader>d', vim.diagnostic.open_float, opts)
        vim.keymap.set('n', '[d', vim.diagnostic.goto_prev, opts)
        vim.keymap.set('n', ']d', vim.diagnostic.goto_next, opts)

        print('Canopy LSP attached - K=hover, <leader>ca=actions, <leader>ih=toggle hints')
      end
    end,
  })
end

--- Auto-setup using CANOPY_LSP_PATH environment variable
function M.auto_setup()
  local lsp_path = os.getenv('CANOPY_LSP_PATH')
  if lsp_path then
    M.setup(lsp_path)
  else
    vim.notify('CANOPY_LSP_PATH not set', vim.log.levels.ERROR)
  end
end

return M
