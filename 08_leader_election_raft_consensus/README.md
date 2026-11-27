# Raft leader election

Your task is to implement the leader election of Raft.

In contrast to real-world implementations, do not add any randomization to
timeouts, but use them as they are provided (this is necessary for testing). The
leader shall send 10 *Heartbeats* during a timeout.

To run the algorithm this
[module system](https://github.com/mykhail0/module-system) is used.
