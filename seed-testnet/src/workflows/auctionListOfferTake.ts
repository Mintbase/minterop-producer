import { Account, utils } from "near-api-js";
import { callContractMethod } from "../calls";
import {
  AUCTION_MARKET_CONTRACT,
  DEFAULT_STORE_CONTRACT,
  MAX_GAS,
} from "../constants";

const nearToYocto = utils.format.parseNearAmount;

export const auctionListOfferTake = async (
  lister: Account,
  buyer: Account,
  firstOfferAccount: Account,
  tokenId: string
) => {
  // call approve to list the token as auction
  const approveCall = await callContractMethod(
    lister,
    DEFAULT_STORE_CONTRACT,
    "nft_approve",
    {
      account_id: AUCTION_MARKET_CONTRACT,
      token_id: tokenId,
      msg: JSON.stringify({ price: nearToYocto("0.1"), autotransfer: false }),
    },
    MAX_GAS,
    nearToYocto("0.008")
  );

  const firstOfferCall = await callContractMethod(
    firstOfferAccount,
    AUCTION_MARKET_CONTRACT,
    "make_offer",
    {
      nft_contract_id: DEFAULT_STORE_CONTRACT,
      token_key: [`${tokenId}:${DEFAULT_STORE_CONTRACT}`],
      price: [nearToYocto("0.11")],
      timeout: [{ Hours: 24 }],
    },
    MAX_GAS,
    nearToYocto("0.11")
  );

  const offerCall = await callContractMethod(
    buyer,
    AUCTION_MARKET_CONTRACT,
    "make_offer",
    {
      nft_contract_id: DEFAULT_STORE_CONTRACT,
      token_key: [`${tokenId}:${DEFAULT_STORE_CONTRACT}`],
      price: [nearToYocto("0.123")],
      timeout: [{ Hours: 24 }],
    },
    MAX_GAS,
    nearToYocto("0.123")
  );

  const takeCall = await callContractMethod(
    lister,
    AUCTION_MARKET_CONTRACT,
    "accept_and_transfer",
    {
      token_key: `${tokenId}:${DEFAULT_STORE_CONTRACT}`,
    },
    MAX_GAS,
    "1"
  );

  return {
    firstOfferCall,
    approveCall,
    offerCall,
    takeCall,
  };
};
