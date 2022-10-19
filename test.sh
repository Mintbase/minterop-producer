#!/usr/bin/env bash

curl -X POST \
  -H 'Content-type: application/json' \
  -d '{"kind": "contract","payload":{"contract_id":"7"}}' \
  http://localhost:3000/publish
