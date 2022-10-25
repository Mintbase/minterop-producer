import { Account, utils } from "near-api-js";
import { callContractMethod } from "../calls";
import { DEFAULT_STORE_CONTRACT, MARKET_CONTRACT, MAX_GAS } from "../constants";

const nearToYocto = utils.format.parseNearAmount;

export const listThenTransferToInvalidate = async (
  lister: Account,
  transfer_to: Account,
  tokenId: string
) => {
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
    DEFAULT_STORE_CONTRACT,
    "nft_approve",
    {
      account_id: MARKET_CONTRACT,
      token_id: tokenId,
      msg: JSON.stringify({ price: nearToYocto("0.5") }),
    },
    MAX_GAS,
    nearToYocto("0.008")
  );

  const transferCallResult = await callContractMethod(
    lister,
    DEFAULT_STORE_CONTRACT,
    "nft_transfer",
    {
      token_id: tokenId,
      receiver_id: transfer_to.accountId,
    },
    MAX_GAS,
    "1"
  );

  return {
    despositCall,
    approveCall,
    transferCallResult,
  };
};
