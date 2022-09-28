import fetch from "node-fetch";

export const fetchCurrentBlockHeight = async () => {
  const res = await fetch("https://rpc.testnet.near.org", {
    method: "POST",
    headers: {
      "content-type": "application/json",
    },
    body: JSON.stringify({
      jsonrpc: "2.0",
      method: "block",
      id: "none",
      params: {
        finality: "final",
      },
    }),
  });

  const { result } = await res.json();
  return result.header.height;
};
