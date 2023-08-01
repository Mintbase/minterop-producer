import * as near from "near-api-js";
import { connect, KeyPair, Account } from "near-api-js";
import { SecretManagerServiceClient } from "@google-cloud/secret-manager";
import {
  ACCOUNT_INITIAL_BALANCE,
  FACTORY_ACCOUNT,
  SECRETS_REPO_PATH,
} from "./constants";
import { getTestNetConfig } from "./utils/getTestNetConfig";

// These funds have to come from mintspace2.testnet,
// unfortunately it seems testnet accounts only also get funded 10N now vs 200N previously

// store keys in project for easy viewing
const keyStore = new near.keyStores.UnencryptedFileSystemKeyStore("./keys");

// near config testnet
export const TESTNET_NEAR_CONFIG = getTestNetConfig(keyStore);

// convert account to secret path.
export const getSecretResourceId = (accountName: string): string =>
  `${SECRETS_REPO_PATH}${accountName.replace(/\./g, "_")}/versions/1`;

export const loadAccount = async (
  accountName: string = FACTORY_ACCOUNT,
): Promise<Account> => {
  const near = await connect(TESTNET_NEAR_CONFIG);
  const client = new SecretManagerServiceClient();
  const [version] = await client.accessSecretVersion({
    name: getSecretResourceId(accountName),
  });

  if (version && version.payload && version.payload.data) {
    // Extract the payload as a string.
    const payload = version.payload.data.toString();
    const gcpPK = JSON.parse(payload).private_key;
    await keyStore.setKey(
      TESTNET_NEAR_CONFIG.networkId,
      accountName,
      KeyPair.fromString(gcpPK),
    );
  } else {
    throw new Error(`Unable to load account ${accountName} from GCP`);
  }

  return await near.account(accountName);
};

// The following is unused (for now) The original idea
// was to make each seed run generate ephemeral accounts
// generating new ones then deleting them post run and returning the funds.

// This didn't work perfectly awesome for a few reasons:
// 1. You current can't programtically generate new accounts without a parent account,
//     so, the balance must be supplied by parent account
// 2. Key management is doable, but more painful. Accidentally override a JSON key? Loose the tokens.

// short term, safer path is use longer lived accounts
// e.g. mb_alice.testnet, mb_bob.testnet
// keep them around as pets and store the keys (like with root) in GCP project (see above)

export const createAccount = async (accountName: string): Promise<Account> => {
  const near = await connect(TESTNET_NEAR_CONFIG);
  // create account under the factory root
  const parent = await near.account(FACTORY_ACCOUNT);
  // generate a key pair
  // THIS WILL OVERWRITE KEYS AND YOU WILL LOSE THE OLD ONES!
  // TODO: Check for existing key, throw overwrite error.
  const keyPair = KeyPair.fromRandom("ed25519");
  const publicKey = keyPair.getPublicKey().toString();
  await keyStore.setKey(TESTNET_NEAR_CONFIG.networkId, accountName, keyPair);

  await parent.createAccount(
    accountName, // new account name
    publicKey, // public key for new account
    ACCOUNT_INITIAL_BALANCE, // initial balance for new account in yoctoNEAR
  );

  return await near.account(accountName);
};

export const deleteAccount = async (accountName: string): Promise<void> => {
  const near = await connect(TESTNET_NEAR_CONFIG);
  const account = await near.account(accountName);
  await account.deleteAccount(FACTORY_ACCOUNT);
};
