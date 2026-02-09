# Blueshift Escrow Program - AI Agent Instructions

## Project Overview
A Solana smart contract implementing a token escrow/swap mechanism. Users can create escrows by depositing Token A in exchange for Token B, which other users can fulfill by providing Token B in return.

## Architecture

### Core Components
- **State** ([state.rs](src/state.rs)): `Escrow` struct holds swap terms (seed, maker, mint_a, mint_b, receive amount, PDA bump)
- **Instructions** ([instructions/](src/instructions/)): Three main operations via discriminators:
  - `Make` (0x0): Create escrow, deposit Token A to vault
  - `Take` (0x1): Fulfill escrow by providing Token B, receive Token A
  - `Refund` (0x2): Cancel escrow and reclaim Token A
- **Helpers** ([helper.rs](src/instructions/helper.rs)): Reusable account validation traits and initialization utilities

### Data Flow
1. **Make**: Maker → creates PDA-derived Escrow account + ATA vault → deposits tokens
2. **Take**: Taker → validates escrow terms → provides Token B → receives Token A from vault
3. **Refund**: Maker → closes vault & escrow PDA → recovers lamports

### PDA Derivation Pattern
All escrows use PDA with seeds: `["escrow", maker_address, seed_u64, bump_byte]`
- The `seed` parameter (u64) enables multiple escrows per maker
- Bump is calculated during `Make` instruction and stored in Escrow state

## Critical Patterns

### Account Validation Framework
Uses trait-based validation (`AccountCheck` trait in [helper.rs](src/instructions/helper.rs)):
- `SignerAccount`: Ensures account has signed the transaction
- `MintInterface`: Validates SPL Token mint ownership (Token/Token-2022 program)
- `AssociatedTokenAccount`: Validates ATAs (init and init_if_needed methods)
- `ProgramAccount`: Manages PDA initialization and closure with proper seeding

**Pattern**: Parse accounts first via `TryFrom<&[AccountView]>`, validate, then extract instruction data.

### Instruction Structure Pattern
Each instruction follows this template:
1. Define `*Accounts<'a>` struct with `TryFrom<&[AccountView]>` for account parsing & validation
2. Define `*InstructionData` struct with `TryFrom<&[u8]>` for instruction data parsing
3. Define main `*<'a>` struct combining accounts + data
4. Implement `process()` method with core logic
5. Set `DISCRIMINATOR: &'a u8` constant for entrypoint routing

See [make.rs](src/instructions/make.rs) for complete example.

### Token Operations
- Uses `pinocchio_token::instructions::Transfer` for token movements
- Vault pattern: Program holds tokens in escrow via PDA-owned ATA
- Token Program CPI invocations are **unchecked** (no explicit invoke_signed needed for transfers)

## Key Dependencies & API
- **pinocchio v0.10.1**: Solana program framework (no_std, CPI support)
  - `AccountView`: Account reference with `try_borrow_mut()`, `is_signer()`, `owned_by()`
  - `Address::find_program_address()`: PDA derivation with `Seed` types
  - CPI: `Signer` from `Seed` for PDA-signed operations
- **pinocchio_token v0.5.0**: SPL Token instruction builders
- **pinocchio_system v0.5.0**: System program instructions (currently stubbed)

## Development Workflow

### Build & Test
```bash
cargo build --target wasm32-unknown-unknown  # Production wasm
cargo build --target wasm32-unknown-unknown --release  # Optimized
cargo test  # Unit tests (if configured)
```

### Common Tasks
1. **Adding new instruction**: Create `src/instructions/newstuff.rs`, add to mod.rs exports, add discriminator in lib.rs entrypoint
2. **Modifying Escrow state**: Update [state.rs](src/state.rs) - recalculate `Escrow::LEN`, update setters
3. **Account validation**: Add new trait in [helper.rs](src/instructions/helper.rs) inheriting `AccountCheck`
4. **Error handling**: Add variants to `EscrowError` enum in [errors.rs](src/errors.rs)

## Project-Specific Conventions

### Type Conversions
- Use `try_from()` for fallible parsing (accounts, instruction data)
- Use `TryFrom` trait implementation pattern consistently
- Always check data length before byte parsing (see MakeInstructionData)

### Memory & Layout
- `#[repr(C)]` for state struct (raw memory layout)
- Use `unsafe` transmute for pointer casts (see `Escrow::load*` methods)
- Escrow state is fixed-size (104 bytes): no dynamic serialization
- Account data validation: `Escrow::LEN` must match actual buffer size

### Error Handling
Multilingual error messages (Chinese + English) with custom error codes (0-4).
Always return `ProgramError` from validation functions.

### Code Organization
- Binary-size optimized: use `#[inline(always)]` on trivial getters/setters
- All account checks happen during struct construction (fail-fast pattern)
- Comments explain "why" (rent exemption, PDA seed requirements, Solana constraints)

## Entry Point & Routing
([lib.rs](src/lib.rs) lines 20-26): Matches first byte of instruction data to discriminator, routes to `process()` method.
Program ID hardcoded as placeholder: `address!("22222...")`
