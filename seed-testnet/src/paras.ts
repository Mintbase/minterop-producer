// paras utils,
// should only need to create a series
// and re-use that can be re-used for interop tests

// create series: https://explorer.testnet.near.org/transactions/97bDRjQmZkMT38zDsewonAbLPZZQZ7kjQUoFe85hDr5W
// set price: https://explorer.testnet.near.org/transactions/NQ9Me9nEhxhZu5HQbhKbEUaXYf3LP5rtBwzBgobHa4H
// calling nft_mint: https://explorer.testnet.near.org/transactions/FAPmdbYWHwrrHrZnY6iQHRtQqKeXn6RaSn8jncp2jxQo

import { Account, utils } from "near-api-js";
import { callContractMethod } from "./calls";
import {
  MAX_GAS,
  PARAS_TOKEN_CONTRACT,
  PARAS_TOKEN_TEST_SERIES_ID,
} from "./constants";

const nearToYocto = utils.format.parseNearAmount;

export const createSeries = async (owner: Account) => {
  const result = await callContractMethod(
    owner,
    PARAS_TOKEN_CONTRACT,
    "nft_create_series",
    {
      creator_id: owner.accountId,
      token_metadata: {
        title: "ParasInteropTestSeries",
        media: "bafkreiadpfvtdi6lzw3nvmlyeyfb6gupharljjkbqtzef3wei6n4rp4o3u",
        reference:
          "bafkreictzt62nt4ba3awdpl4ypyozoqrmspjnmuoindlzpocvnw2ibgugi",
        copies: 100,
      },
      price: null,
      royalty: {
        "mb_alice.testnet": 1000,
        "mb_bob.testnet": 1000,
      },
    },
    MAX_GAS,
    nearToYocto("5")
  );

  return result;
};

export const mintParasToken = async (
  minter: Account,
  owner: Account,
  seriesId: string = PARAS_TOKEN_TEST_SERIES_ID
) => {
  const result = await callContractMethod(
    minter,
    PARAS_TOKEN_CONTRACT,
    "nft_mint",
    {
      token_series_id: seriesId,
      receiver_id: owner.accountId,
    },
    MAX_GAS,
    nearToYocto("0.09")
  );

  const [{ data }] = result;
  const [{ token_ids }] = data;
  return token_ids;
};
