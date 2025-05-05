# Anchor Context DSL

`anchor-context-dsl` is an ergonomic Domain-Specific Language (DSL) for defining Anchor contexts in Solana programs. It simplifies the process of specifying accounts, their constraints, initialization logic, and dependencies in a concise and readable YAML-like syntax. This DSL is designed to streamline Solana program development by reducing boilerplate code and improving maintainability.

## Features

- **Concise Syntax**: Define complex Anchor contexts using a YAML-inspired format.
- **Account Dependencies**: Easily specify dependencies between accounts.
- **Constraints and Initialization**: Support for account constraints, initialization logic, and seed definitions.
- **Modularity**: Organize contexts into reusable definitions for different program functionalities.
- **Type Safety**: Leverages Anchor's type system for robust account validation.
- **Extensibility**: Supports custom account types, extensions, and zero-copy deserialization.

## Installation

Add `anchor-context-dsl` to your project by including it in your `Cargo.toml`:

```toml
[dependencies]
anchor-context-dsl = "0.1.0"  # Replace with the latest version
```

## Usage

The DSL allows you to define accounts and contexts in a single block using the yaml_contexts! macro. Below is an example showcasing various account definitions and context groupings.

## Example

```rust

use anchor_context_dsl::yaml_contexts;

yaml_contexts!({

// Account Definitions
quote_mint:
  type: Mint
  constraints:
    - quote_mint.freeze_authority.is_none()

cell_quote_reserve:
  depends: [cell, quote_mint, sys]
  seeds: [b"cell_quote_reserve", cell.key().as_ref(), quote_mint.key().as_ref()]
  type: TokenAccount
  token::mint: quote_mint
  token::authority: sys
  noinit:
    constraints:
      - quote_mint.key() == cell.get_quote_asset().expect("cell not tradeable")

cell:
  type: Cell
  boxed: true
  if init:
    space: 10240
    seeds: [b"cell", cell_id.to_le_bytes().as_ref()]
  else:
    seeds: [b"cell", cell.id.to_le_bytes().as_ref()]

// Context Definitions
context SwapNativeTokenCellContext:
  signer
  cell: [mut]
  cell_quote_reserve
  signer_quote_ata: [mut]
  signer_meme_ata
  quote_fees

context NewNativeTokenCellContext:
  instruction: (cell_id: u32)
  cell: [init]
  cell_quote_reserve: [init]
  quote_fees
  meme_mint: [init]
  extra_account_meta_list: [init]
});
```

## Key Components

1. **Account Definitions**:
   - Specify the `type` of the account (e.g., `Mint`, `TokenAccount`, `Signer`).
   - Use `depends` to declare dependencies on other accounts.
   - Define `seeds` for Program-Derived Addresses (PDAs).
   - Add `constraints` to enforce validation rules.
   - Use `if init` or `noinit` to handle initialization logic and space allocation.

2. **Context Definitions**:
   - Group accounts into contexts for specific program instructions.
   - Specify whether accounts are mutable (`[mut]`) or initialized (`[init]`).
   - Include instruction arguments using `instruction: (arg: type)`.

3. **Special Features**:
   - `boxed: true` for accounts requiring boxed storage.
   - `zero_copy: true` for zero-copy deserialization.
   - `init_if_needed: true` for optional initialization of accounts like Associated Token Accounts (ATAs).
   - Support for token extensions (e.g., `mint::token_program: token_program_2022`).

## Contributing

Contributions are welcome! To contribute:

1. Fork the repository.
2. Create a new branch (`git checkout -b feature/your-feature`).
3. Make your changes and commit (`git commit -m "Add your feature"`).
4. Push to the branch (`git push origin feature/your-feature`).
5. Open a Pull Request.

Please ensure your code follows the project's coding standards and includes tests.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## Contact

For questions or support, please open an issue on the GitHub repository or contact the maintainers at [your-contact-info].
