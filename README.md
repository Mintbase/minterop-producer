# minterop-producer

## Mintbase (v2) Indexer

This repo contains the latest version of the Mintbase Indexer.
The remaining services have been moved into other repositories following a event based pub/sub architecture which is explained bellow.

"Mintbase" + "Interoperability" (consuming other contract events) = **"Minterop"** 


## Local Development

Ask someone on the team to supply you with 
1. an `.env` file that contains all the configurations you need 
2. a backend setup script which contains dependencies  
3. aws credentials

To run locally you can use this script which spins up a docker container on your machine with a db instance while running the indexer

```
scripts/run-cargo.sh
```

This env can be useful for confirming its working
```
RUST_LOG='minterop=debug'
```


## integration-tests 

(**work in progress**)

Integration tests are run when pull requests are generated. Given any start and stop block height, assertions can be confidently made that indexer data will be consistent or at least change as excpected due to code changes, as the index is produced from an immutable stream (the blockchain). 

To run the local tests use 

```
scripts/run-tests.sh
```


## Event based architecture

The indexer processes the events and when it needs to resolve certain chunks of metadata it sends an http request to the **minterop-consumer** where it gets forwaded as a published gcp pub/sub message by the **event-dispatcher** which eventually gets pushed to the **metadata-resolver** where it gets handled and written to the db.


