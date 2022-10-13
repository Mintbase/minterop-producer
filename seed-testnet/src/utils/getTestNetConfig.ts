import * as near from "near-api-js";

export const getTestNetConfig = (
  keyStore: near.keyStores.UnencryptedFileSystemKeyStore
) => {
  const TESTNET_NEAR_CONFIG = {
    networkId: "testnet",
    nodeUrl: "https://rpc.testnet.near.org",
    walletUrl: "https://wallet.testnet.near.org",
    helperUrl: "https://helper.testnet.near.org",
    headers: {},
    keyStore,
  };
  return TESTNET_NEAR_CONFIG;
};
