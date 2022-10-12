import { Account, utils } from "near-api-js";
import { callContractMethod } from "../calls";
import { MARKET_CONTRACT, MAX_GAS, PARAS_TOKEN_CONTRACT } from "../constants";

const nearToYocto = utils.format.parseNearAmount;

export const parasListAndSale = async (
  lister: Account,
  buyer: Account,
  tokenId: string
) => {
  // in this workflow, the minting happens on paras (see paras.ts for utils there

  // make a storage deposit
  const despositCall = await callContractMethod(
    lister,
    MARKET_CONTRACT,
    "deposit_storage",
    {},
    MAX_GAS,
    nearToYocto("0.01")
  );

  // call approve to list the token
  const approveCall = await callContractMethod(
    lister,
    PARAS_TOKEN_CONTRACT,
    "nft_approve",
    {
      account_id: MARKET_CONTRACT,
      token_id: tokenId,
      msg: JSON.stringify({ price: nearToYocto("0.5") }),
    },
    MAX_GAS,
    nearToYocto("0.008")
  );

  const purchaseCall = await callContractMethod(
    buyer,
    MARKET_CONTRACT,
    "buy",
    {
      nft_contract_id: PARAS_TOKEN_CONTRACT,
      token_id: tokenId,
    },
    MAX_GAS,
    nearToYocto("0.6")
  );

  return {
    despositCall,
    approveCall,
    purchaseCall,
  };
};
