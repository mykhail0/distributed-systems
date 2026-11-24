# Distributed commit

Imagine working for a storage company which has recently released a distributed
storage system, named *CyberStore2047*. However, it was not released in a
polished state, and it still requires some quick fixes you have to implement.

Your task is to implement lacking functionalities of the distributed storage
system based on 2PC (the two-phase commit protocol). The system stores products
(the `Product` struct), and allows changing their prices by specifying product
type, for example, *increase price of all computers by 10*. The information
about prices of the products is stored in distributed shards, following some
partition key. *CyberStore2047* shall use 2PC to atomically perform updates on
all shards. There will only be at most one transaction in progress at any given
time.

Every process must respond to `ProductPriceQuery` messages and return the
current price of the queried product. `None` shall be returned if the product is
not in the store.

Processes of the system shall deny transactions (by voting to abort them) that
would result in nonpositive prices of products (i.e., the prices shall be always
greater than `0`).

In this assignment you should assume a crash-stop model — you should not store
anything in stable storage or support recoveries. You also should not implement
any timeouts (we accept that a crash can prevent the system from making
progress).

To run the system you should use the
[executor system](https://github.com/mykhail0/module-system).
