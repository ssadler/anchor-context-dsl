#![allow(unexpected_cfgs)]

pub use anchor_lang::system_program::ID;
pub use anchor_spl::token::{TokenAccount, Mint};

use anchor_yaml_accounts::*;
use anchor_lang::prelude::*;


#[anchor_context_dsl({

payer:
  type: Signer<'info>

quote_mint:
  type: Mint
  constraints:
    - quote_mint.freeze_authority.is_none()
  if init:
    space: 128

})]
mod contexts {
    use super::*;

    #[context]
    pub struct Thingy<'info> {
        #[account(mut)]
        pub payer: Signer<'info>,
        #[account(init)]
        pub quote_mint: Account<'info, Mint>,
    }
}
use contexts::*;

#[account]
struct Cell { id: u32 }

#[account]
struct CellSystem { a: bool }

