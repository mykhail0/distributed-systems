# Distributed text editor

The task is to implement a distributed text editor. However, given that an
algorithm that guarantees truly eventual consistency is too involved for a Small
Assignment, you are to implement the algorithm presented hitherto but modified
so as to guarantee text convergence through process synchronization.

## The algorithm

Processes operate in implicit rounds. At the beginning of a new round, each
process issues its operation (with respect to its current text, that is, the
text resulting from the previous round), appends the operation to the log,
applies it to the text, and broadcasts to other processes. Then, it waits for
operations from other process and processes them (i.e., transforms, appends to
the log, and applies them to the text). The round ends when the process receives
operations from all other processes. When the process receives an operation from
the next round (every process manages the rounds independently, so some may
already start the next round and broadcast its next operation), the operation is
processed not until the next round. Operations within a round are always
processed in the order they are received.

When at the beginning of a round a process does not have an operation to be
issued, it waits for the next event. If the event is a user's request of a new
edit, then the process continues with this operation. If it is a reception of
some other process' operation, the process internally issues itself a `NOP`
operation, processes it, and then continues processing the received operation.
Effectively, an operation issued by the process itself is always the first
operation appended to its log in each round.

This way, in every round, each process issues exactly one operation. The
operation is newer than all operations issued in the previous rounds and
concurrent with all operations issued by other processes in the same round.
Therefore, there is no need to use vector clocks. Moreover, all concurrent
operations are issued with respect to the same text.

The modified algorithm is illustrated in a [diagram](example-sa.svg).

## Implementation specification

The system consists of processes, clients, and a broadcast module.

The processes manage the exchange and transformations of text operations. There
is a constant number of processes in the system, and they are assigned
consecutive ranks (starting with rank `0`). You can assume the processes do not
fail.

Operation transformations shall be implemented as follows:

```code
Transform insert(p1, c1, r1) wrt. insert(p2, c2, r2) :-
  if p1 < p2: insert(p1, c1, r1)
  if p1 = p2 and r1 < r2: insert(p1, c1, r1)
  else: insert(p1 + 1, c1, r1)

Transform delete(p1, r1) wrt. delete(p2, r2) :-
  if p1 < p2: delete(p1, r1)
  if p1 = p2: NOP (do not modify text)
  else: delete(p1 - 1, r1)

Transform insert(p1, c1, r1) wrt. delete(p2, r2) :-
  if p1 <= p2: insert(p1, c1, r1)
  else: insert(p1 - 1, c1, r1)

Transform delete(p1, r1) wrt. insert(p2, c2, r2) :-
  if p1 < p2: delete(p1, r1)
  else: delete(p1 + 1, r1)
```

In contrast to the text editor described above, the processes do not store texts
or issue commands themselves. In the assignment, these activities are done by
the clients.

Each client is coupled with exactly one process. To issue an operation, the
client sends an `EditRequest` message to its process. It can issue multiple edit
requests at once. You can assume that the requests are valid edits (with respect
to the text as of they are issued). The process shall process requests in the
order they are received, introducing them to the system one by one in the first
possible rounds. The process sends to its client an `Edit` message each time the
client should apply a new operation to its text (including operations issued by
the client itself and the `NOP` operation). Since the client communicates with
its process in an asynchronous way, in each edit request, the client sends also
the total number of operations it has applied to its text so far. This way, the
process can learn with respect to which state the operation was issued, and
transforms the request with respect to operations which were appended to the log
later. These transformations shall follow the same transformation rules as
concurrent operations, and the edit request shall be assigned a temporary
process rank larger than ranks of actual processes (e.g., `N+1` in a system
containing `N` processes). After the transformations, the requested operation is
appended to the log, broadcast, and sent back to the client (which applies it to
the text only then).

To broadcast an operation to other processes, the process shall send an
`Operation` message to the broadcast module. Each message will be eventually
delivered to every other process exactly once, and messages from a process will
be delivered in the order they were sent by this process. You are not allowed to
send messages between processes other than through the broadcast module.

You shall complete the implementation of the process and necessary data
structures provided in the template, following the interfaces and doc comments
defined there. Exemplary implementations of the client and the broadcast module
are already provided.

To run the system this
[executor system](https://github.com/mykhail0/module-system) is used.
