import { Account, Contract } from "near-api-js";
import { CONTRACT_STORAGE_DEPOSIT, DEFAULT_STORE_CONTRACT, FACTORY_ACCOUNT, GAS, MAX_GAS, ONE_NEAR } from "./constants";


export type EventLog = {
  standard: string;
  version: string;
  event: string;
  data: {
    token_ids: [string];
  }[];
};

const parseLogsJson = (log: string) =>
  JSON.parse(log.replace("EVENT_JSON:", ""));

export const callContractMethod = async (
  account: Account,
  contractId: string,
  methodName: string,
  args: any,
  gas: string = MAX_GAS,
  attachedDeposit: string | null = ONE_NEAR
): Promise<EventLog[]> => {
  const result = await account.functionCall({
    contractId,
    methodName,
    args,
    gas,
    attachedDeposit,
  });

  // attempt to collect logs from the transaction and parse them
  try {
    // result[0]
    const [outcome] = result.receipts_outcome;
    const logs = outcome.outcome.logs;
    return logs.map(parseLogsJson);
  } catch (err: unknown) {
    console.info(
      `Contract call ${contractId}.${methodName} result produced no parsable logs`,
      err
    );
    return [];
  }
};

export const mintTokensWithAccount = async (
  account: Account,
  numTokens = 1,
  contractId: string = DEFAULT_STORE_CONTRACT,
  method = "nft_batch_mint"
): Promise<string[]> => {
  const result = await callContractMethod(account, contractId, method, {
    owner_id: account.accountId,
    metadata: {
      title: "Test NFT Mint", // this gets overridden by the reference anyway,
      // reference is an arweave hash from retreat demo for now
      reference: "4XKmOs3BhcqRGAFX3aZ2z44g9s6DySudzAOA4pVvRYY",
    },
    num_to_mint: numTokens,
  });
  const [{ data }] = result;
  const [{ token_ids }] = data;
  return token_ids;
};



// Good examples, not used regularly
export type StoreContract = {
  account: Account;
  contractId: string;
  nft_batch_mint: (args: any) => Promise<void>;
  // nft_transfer: [Function: nft_transfer],
  // nft_batch_transfer: [Function: nft_batch_transfer],
  // nft_batch_burn: [Function: nft_batch_burn]
} & Contract;

export const loadStoreContract = async (account: Account, contract: string) =>
  new Contract(account, contract, {
    changeMethods: [
      "nft_batch_mint",
      "nft_transfer",
      "nft_batch_transfer",
      "nft_batch_burn",
    ],
    viewMethods: [],
  });

export const deployStoreContract = async (owner: Account, name: string) => {
  const storeFactory = new Contract(owner, FACTORY_ACCOUNT, {
    // sender: alice,
    changeMethods: ["create_store"],
    viewMethods: [],
  }) as any;

  // deploy a store for account by calling factory method
  const args = {
    owner_id: owner.accountId,
    metadata: {
      spec: "nft-1.0.0",
      name,
      symbol: "TEST",
    },
  };
  const store = await storeFactory.create_store({
    args,
    gas: GAS,
    amount: CONTRACT_STORAGE_DEPOSIT,
  });
  return store;
};
