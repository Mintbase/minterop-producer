## We are reindexing mainnet metadata

**What will happen?**

Due to developments in our indexing and to provider richer metadata, we need to
reindex a bunch of metadata. This will happen with Mintbase staying operational,
but will come with restrictions

**But why?**

Given a `<field>` in the reference blob that is a URL, the blob will be
augmented by `<field>_type` with a mime type for what the URL is pointing to and
`<field>_size`, giving the size in bytes of what the URL is pointing to.
Furthermore, the `nft_attributes` table is feature to enable searching NFTs by
their attributes. Both things are already being indexed for newly minted tokens,
but we need to reindex older tokens to make sure all MB tokens have this
information.

**What are the implications?**

This will cause quite a bit of load on the mainnet indexing system. You might
notice delays when minting or broken tokens on batch minting. It is adviseable
to limit batch minting to 20 NFTs per transaction during this process. The
process itself is already throttled to a compromise between duration of the
limitations and their impact and is estimated to take 1-2 weeks.

We expect to start this process soon and will drop a message here once it's
actually starting.

tldr: Please limit batch minting to 20 NFTs per transaction, expect some minor
delays on the indexer (magnitude of seconds)
