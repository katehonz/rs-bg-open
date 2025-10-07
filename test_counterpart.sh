#!/bin/bash

curl -X POST http://localhost:8080/graphql \
  -H "Content-Type: application/json" \
  -d '{
    "query": "mutation CreateCounterpart($input: CreateCounterpartInput!) { createCounterpart(input: $input) { id name eik vatNumber isVatRegistered } }",
    "variables": {
      "input": {
        "name": "Test Counterpart",
        "companyId": 1,
        "isVatRegistered": false
      }
    }
  }'