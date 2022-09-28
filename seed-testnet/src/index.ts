import { Account, Contract } from "near-api-js";
import { loadAccount } from "./accounts";
import {
  StoreContract,
  loadStoreContract,
  callContractMethod,
  EventLog,
  mintTokensWithAccount,
  // deployStoreContract
} from "./calls";
import { DEFAULT_STORE_CONTRACT } from "./constants";
import { createSeries, mintParasToken } from "./paras";
import { fetchCurrentBlockHeight } from "./rpc";
import { EnvWriter } from "./utils/envWriter"
import { parasListAndSale } from "./workflows/parasListAndSale";
import { simpleBurn } from "./workflows/simpleBurn";
import { simpleListAndSale } from "./workflows/simpleListAndSale";
import { simpleTransfer } from "./workflows/simpleTransfer";



async function main() {
  // call RPC to get current block height
  const startBlockHeight = await fetchCurrentBlockHeight();

  // load the seeding account (store factory) root (mintspace2.testnet)
  // NOTE, this is not needed unless attempting to create new accounts
  // await loadAccount();

  const alice: Account = await loadAccount("mb_alice.testnet");
  const bob: Account = await loadAccount("mb_bob.testnet");
  const carol: Account = await loadAccount("mb_carol.testnet");

  // deploy store
  // NOTE: to save gas / token we are reuse mb_store.mintspace2.testnet
  // and fetching token ids to work with from minting call.
  // const store = await deployStoreContract(alice, 'mb_store');

  // load a store contract
  // NOTE: No longer using the new Contract instance warpper from NAJ,
  // as method calls do not return a response (see calls.ts)
  // const store = await loadStoreContract(alice, 'mb_store.mintspace2.testnet') as StoreContract;

  // begin by minting some tokens
  // NOTE: As more use cases are required, increment the number of tokens to mint
  const [tokenToTransfer, tokenForSale, tokenToBurn] =
    await mintTokensWithAccount(alice, 3);

  // workflows

  // simple transfer
  const transferResult = await simpleTransfer(
    alice, // from
    carol, // to
    tokenToTransfer
  );

  // simple list and sale
  const saleResult = await simpleListAndSale(
    alice, // seller
    bob, // buyer
    tokenForSale
  );

  // simple burn of a token
  const tokenIds: string[] = [tokenToBurn];
  const burnResult = await simpleBurn(alice, tokenIds);

  //paras interop
  const parasSeries = await createSeries(alice);
  //mint a paras token, then list and sell it on mintbase
  const parasToken = await mintParasToken(alice, bob);
  const listAndPurchase = await parasListAndSale(bob, carol, parasToken[0]);

  const stopBlockHeight = await fetchCurrentBlockHeight();
  
  //write to env
  const envWriter = new EnvWriter("../.env")
  envWriter.setEnvValues({
    "START_BLOCK_HEIGHT": startBlockHeight,
    "STOP_BLOCK_HEIGHT": stopBlockHeight +5 //kinda hacky, addded 5 to block height to give some time to last transaction that wasnt getting processed in time
  })

  return {
    transferResult,
    saleResult,
    burnResult,
    startBlockHeight,
    stopBlockHeight,
  };
}

main()
  .catch((err) => {
    console.error("Failed to seed testnet accounts");
    console.error(err);
    process.exit(1);
  })
  .then((result: any) => {
    console.info("Seeding completed successfully");
    console.info(JSON.stringify(result, null, 2));
    process.exit(0);
  });
