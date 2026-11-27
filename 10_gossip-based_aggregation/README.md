# Gossip-based aggregation

The task is to implement gossip-based aggregation using probabilistic counting
sketches as discussed hitherto. The implementation must be based on the supplied
template, which assumes a system consisting of `Nodes` and clients that operate
as follows.

A client can contact any node and request it to install an aggregation query,
which is done by sending to the node a `QueryInstallMsg`. The goal of the query
is estimating the number of nodes in the system that satisfy the associated
`predicate`. The query also specifies the number of probabilistic sketch
instances (`num_instances`) and bits in each instance (`bits_per_instance`) that
should be utilized when computing the aggregate.

Nodes use gossiping to propagate among each other the information necessary for
executing such queries. A node initiates gossiping whenever it receives a
`SyncTriggerMsg`. It is the system that controls when those messages are sent;
you must never send them in your implementation. Upon reception of such a
message, the node requests its associated peer sampling service (`pss`) to
obtain a random node with which it will gossip. Subsequently, it sends the
necessary information to that node within a single `SyncGossipMsg`. The other
node does not reply with any message, that is, the communication scheme is
push-based.

At any moment in time, a client can contact any node asking it to provide the
current estimate of the result for any query. The client does this by sending to
the node a `QueryResultPollMsg`. The message carries the identifier of the node
on which the query was originally installed (`initiator`) and a callback to be
executed by the receiving node with the query result estimate as the parameter
(`callback`). If the node is not aware of the query, it executes the callback
with value `None`; otherwise, it executes the callback with its current estimate
of the node count satisfying the predicate of the query.

To recap, multiple queries can be executing in the system at the same time.
However, a next query installed by a client at a node overrides the previous
query installed at that node by the same or another client: the execution of the
previous query should eventually cease. In other words, for every initiator,
each node executes only the most recent query of which it is aware. Finally, a
`SyncGossipMsg`, sent by a node to its peer, should contain information on all
queries executed by the node.

Your solution must implement probabilistic counting by means of the
`ProbabilisticCounter` structure. As the source of random values
(`RandomnessSource`) for selecting sketch bits, the dedicated `rs` object of the
node must be utilized. The object generates pseudo-random values from a uniform
distribution. To convert them into ones from the desired geometric distribution,
your solution must use the provided `uniform_u32_to_geometric` method. Moreover,
`ProbabilisticCounter` must ensure special handling of two corner cases. First,
`evaluate` on a counter in which no sketch instance has any bit set to `1`
should return `0` instead of the value implied by the previous formula. Second,
`evaluate` on a counter in which at least one instance has all bits set to `1`
should return infinity (`u64::MAX`) instead of the value from the formula.

To run the system this
[executor system](https://github.com/mykhail0/module-system) is used.
