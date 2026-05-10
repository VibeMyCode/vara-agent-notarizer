# Vara Agent Notarizer

On-chain notarization and attestation service for AI agents on Vara Network.

## Service methods

### Paid methods:
- **Notarize(hash: [u8; 32], metadata: str)** → receipt_id (1 TVARA)
  Store a hash + metadata with block timestamp. Returns unique receipt_id.
- **Attest(subject: actor_id, claim: str)** → attestation_id (0.5 TVARA)
  One agent attests something about another.

### Fee management (owner only):
- **SetFeeNotarize(new_fee: u128)** - change notarize fee
- **SetFeeAttest(new_fee: u128)** - change attest fee
- **WithdrawFees()** - withdraw accumulated fees to owner wallet

### Free queries:
- **Verify(receipt_id, hash)** → bool - check receipt exists with that hash
- **GetReceipt(receipt_id)** → Option<Receipt>
- **GetReceiptsByAuthor(author, cursor, limit)** → ReceiptPage
- **GetAttestations(subject, cursor, limit)** → AttestationPage
- **GetConfig()** → Config

## Integration example (paying agent)
```bash
# Notarize a hash
vara-wallet --account <wallet> --network testnet call \\
  0xbee4db329d44cafaeb4c531b3bef69e6dd7cb77197c6d70e172aa99e533ccf61 \\
  Notarizer/Notarize --args '["0x<hash32>", "metadata"]' --value 1 --idl <idl>

# Verify
vara-wallet --network testnet --json call \\
  0xbee4db329d44cafaeb4c531b3bef69e6dd7cb77197c6d70e172aa99e533ccf61 \\
  Notarizer/Verify --args '[1, "0x<hash32>"]' --idl <idl>
```

## Pricing
- Notarize: 1 TVARA (changeable via set_fee_notarize)
- Attest: 0.5 TVARA (changeable via set_fee_attest)
- Gas: vouchers accepted
- Overpayment stays as donation to program

## Program IDs
- V1 (initial deploy): 0x5321933319be33804e039c27aec7b25426f263a598da4d87de1346c108a73b92
- V2 (with withdraw_fees): 0xbee4db329d44cafaeb4c531b3bef69e6dd7cb77197c6d70e172aa99e533ccf61
- V3 ⭐ (with metadata + withdraw_fees): 0x46227198294bb3ede5a1caa3801c731734d5e83c4ac06d72f3df64c5d7195acf

## Building with metadata (for Gear IDE)
```bash
# 1. Build normally
cargo build --release

# 2. Inject IDL as wasm custom section
python3 scripts/inject_idl.py \
  target/wasm32-gear/release/vara_agent_notarizer.opt.wasm \
  target/wasm32-gear/release/vara_agent_notarizer.idl

# 3. Deploy with meta-wasm
vara-wallet --account <wallet> --network testnet program upload \
  target/wasm32-gear/release/vara_agent_notarizer.opt.wasm_with_meta.wasm \
  --init New --args '["<owner_hex>"]' \
  --idl target/wasm32-gear/release/vara_agent_notarizer.idl
```
