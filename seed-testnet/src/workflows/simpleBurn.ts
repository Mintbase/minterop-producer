import { Account, utils } from "near-api-js";
import { callContractMethod } from "../calls";
import { DEFAULT_STORE_CONTRACT, MARKET_CONTRACT, MAX_GAS } from "../constants";

const nearToYocto = utils.format.parseNearAmount;

export const simpleBurn = async (
  account: Account,
  tokenIds: string[],
  method = "nft_batch_burn",
): Promise<any> => {
  const burnCall = await callContractMethod(
    account,
    DEFAULT_STORE_CONTRACT,
    method,
    {
      token_ids: tokenIds,
    },
    MAX_GAS,
    "1",
  );
  return {
    burnCall,
  };
};
