import { Account } from "near-api-js";
import { callContractMethod} from "../calls";
import { DEFAULT_STORE_CONTRACT, MAX_GAS } from "../constants";

export const simpleTransfer = async (
  from: Account,
  to: Account,
  tokenId: string
) => {
  const transferCallResult = await callContractMethod(
    from,
    DEFAULT_STORE_CONTRACT,
    "nft_transfer",
    {
      token_id: tokenId,
      receiver_id: to.accountId,
    },
    MAX_GAS,
    "1"
  );

  return {
    transferCallResult,
  };
};
