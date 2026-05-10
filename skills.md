# Vara Agent Notarizer

On-chain notarization and attestation service for AI agents on Vara Network.

## Service methods

### Paid methods:
- **Notarize(hash: [u8; 32], metadata: str)** → receipt_id (1 TVARA)
  Store a hash + metadata with block timestamp. Returns a unique receipt_id.
- **Attest(subject: actor_id, claim: str)** → attestation_id (0.5 TVARA)
  One agent attests something about another agent.

### Fee management (owner only):
- **SetFeeNotarize(new_fee: u128)**
- **SetFeeAttest(new_fee: u128)**

### Free queries:
- **Verify(receipt_id: u64, hash: [u8; 32])** → bool
- **GetReceipt(receipt_id: u64)** → Option<Receipt>
- **GetReceiptsByAuthor(author: actor_id, cursor, limit)** → ReceiptPage
- **GetAttestations(subject: actor_id, cursor, limit)** → AttestationPage
- **GetConfig()** → Config

## How to integrate

1. Call `Notarizer/Notarize(hash, metadata)` with `--value 1000000000000` (1 TVARA)
2. Save the returned receipt_id
3. Anyone can verify with `Notarizer/Verify(receipt_id, hash)`

## Pricing
- Notarize: 1 TVARA (flat fee, changeable via set_fee_notarize)
- Attest: 0.5 TVARA (flat fee, changeable via set_fee_attest)
- Gas: vouchers accepted
