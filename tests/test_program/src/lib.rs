#![allow(unexpected_cfgs)]

pub use anchor_lang::system_program::ID;
pub use anchor_spl::token::{TokenAccount, Mint};

use anchor_yaml_accounts::*;
use anchor_lang::prelude::*;


// so $MYVIMRC

yaml_contexts!({
// YAML

quote_mint:
  type: Mint
  constraints:
    - quote_mint.freeze_authority.is_none()

cell_quote_reserve:
  depends:
    - sys
    - cell
    - quote_mint
  seeds: [b"cell_quote_reserve"] // , cell.key().as_ref(), quote_mint.key().as_ref()]
  type: TokenAccount
  token::mint: quote_mint
  token::authority: sys

sys:
  seeds: [b"system"]
  type: CellSystem

cell:
  type: Cell
  boxed: true

  if init:
    space: 10240
    seeds: [b"cell", cell_id.to_le_bytes().as_ref()]
  else:
    seeds: [b"cell", cell.id.to_le_bytes().as_ref()]

//  noinit:
//    mut: true
//    constraints:
//      - "quote_mint.key() == cell.get_quote_asset().expect(\"cell not tradeable\")"
//
context Thingy:
  cell_quote_reserve

// END
});

#[account]
struct Cell { id: u32 }

#[account]
struct CellSystem { a: bool }


