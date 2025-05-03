#![allow(unexpected_cfgs)]

pub use anchor_lang::system_program::ID;
pub use anchor_lang;

use anchor_yaml_accounts::*;
use anchor_lang::prelude::*;


// so $MYVIMRC


yaml_contexts!({
// YAML

cell_quote_reserve:
  depends: 
    - cell
    - quote_mint
    - sys
  seeds: "[b\"cell_quote_reserve\", cell.key().as_ref(), quote_mint.key().as_ref()]"
  struct: TokenAccount
  token::mint: quote_mint
  token::authority: sys
  noinit:
    mut: true
    constraints:
      - "quote_mint.key() == cell.get_quote_asset().expect(\"cell not tradeable\")"

sys:
  type: "Signer<'info>"
  struct: Wat
  init:
    struct: Who


admin_init:
  struct: Wat

context Thingy:
  cell_quote_reserve: []

// END
});
