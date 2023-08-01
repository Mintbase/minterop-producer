import { Account, utils } from "near-api-js";
import { callContractMethod } from "../calls";
import { DEFAULT_STORE_CONTRACT, MARKET_CONTRACT, MAX_GAS } from "../constants";

const nearToYocto = utils.format.parseNearAmount;

export const simpleListAndSale = async (
  lister: Account,
  buyer: Account,
  tokenId: string,
) => {
  // make a storage deposit
  const despositCall = await callContractMethod(
    lister,
    MARKET_CONTRACT,
    "deposit_storage",
    {},
    MAX_GAS,
    nearToYocto("0.01"),
  );

  // call approve to list the token
  const approveCall = await callContractMethod(
    lister,
    DEFAULT_STORE_CONTRACT,
    "nft_approve",
    {
      account_id: MARKET_CONTRACT,
      token_id: tokenId,
      msg: JSON.stringify({ price: nearToYocto("0.5") }),
    },
    MAX_GAS,
    nearToYocto("0.008"),
  );

  const purchaseCall = await callContractMethod(
    buyer,
    MARKET_CONTRACT,
    "buy",
    {
      nft_contract_id: DEFAULT_STORE_CONTRACT,
      token_id: tokenId,
    },
    MAX_GAS,
    nearToYocto("0.6"),
  );

  return {
    despositCall,
    approveCall,
    purchaseCall,
  };
};
