
import * as borsh from "@coral-xyz/borsh";
import { PublicKey } from '@solana/web3.js';

// Helper function for PublicKey serialization/deserialization
export const publicKeySchema = {
  serialize: (publicKey) => publicKey.toBytes(),
  deserialize: (buffer) => new PublicKey(buffer),
};
// Schéma pour extraire la clé de type de compte
export const keySchema = borsh.struct([
  borsh.u64("value"),
  borsh.u8("account_type"),
]);

// ContractAccount Schema
export const contractAccountSchema = borsh.struct([
  borsh.u64("value"),
  borsh.u8("account_type"),
  borsh.publicKey("admin")
]);
