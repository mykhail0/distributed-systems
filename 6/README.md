# Health checks

The task is to implement a distributed system in which all process are
monitoring each other over UPD. The implementation shall follow the
*EventualFailureDetector* algorithm presented during the lectures, but with a
constant delay (i.e., no increment by ∆ upon incorrect detections).

The processes of the system, the `FailureDetectorModule` modules, send at
regular intervals (specified by `delay`) the `HeartbeatRequest` messages to all
other processes and collect their responses over UDP (the `HeartbeatResponse`
messages) until the end of the interval. Then, they calculate which processes
are suspected of being (not) alive. The information is queried by sending the
`AliveRequest` message. The `AliveInfo` response shall contain identifiers of
processes which, according to the queried process, were alive during the
previous time interval.

You shall complete the implementation of the system provided in the template.
You shall implement:

* `FailureDetectorModule::new()`, which creates and initializes the module,
* handler for the `DetectorOperationUdp` messages, which serve health checks and
querying the status,
* handler for the `Tick` message, which is sent periodically to the module (at
the `delay` interval).

To serialize requests and responses to raw bytes you shall use the `postcard`
crate. To run the system you should use the
[module system](https://github.com/mykhail0/module-system).
