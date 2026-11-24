use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use log::debug;
use serde::{Deserialize, Serialize};
use tokio::net::UdpSocket;
use uuid::Uuid;

use executor::{Handler, ModuleRef, System};

/// A message which disables a process. Used for testing
pub struct Disable;

#[derive(Clone)]
struct Tick;

pub struct FailureDetectorModule {
    enabled: bool,
    alive: HashSet<Uuid>,
    ident: Uuid,
    socket: Arc<UdpSocket>,
    addresses: HashMap<Uuid, SocketAddr>,
}

impl FailureDetectorModule {
    pub async fn new(
        system: &mut System,
        delay: Duration,
        addresses: &HashMap<Uuid, SocketAddr>,
        ident: Uuid,
        all_idents: HashSet<Uuid>,
    ) -> ModuleRef<Self> {
        let addr = addresses.get(&ident).unwrap();
        let socket = Arc::new(UdpSocket::bind(addr).await.unwrap());

        let module_ref = system
            .register_module(Self {
                enabled: true,
                alive: all_idents,
                ident,
                socket: Arc::clone(&socket),
                addresses: addresses.clone(),
            })
            .await;

        tokio::spawn(deserialize_and_forward(socket, module_ref.clone()));

        module_ref.request_tick(Tick, delay).await;

        module_ref
    }
}

/// New operation arrived on socket.
#[async_trait::async_trait]
impl Handler<DetectorOperationUdp> for FailureDetectorModule {
    async fn handle(&mut self, _self_ref: &ModuleRef<Self>, item: DetectorOperationUdp) {
        if self.enabled {
            let DetectorOperationUdp(msg, socket) = item;

            match msg {
                DetectorOperation::HeartbeatRequest => {
                    self.socket.send_to(
                        bincode::serialize(
                            &DetectorOperation::HeartbeatResponse(self.ident)
                        ).unwrap().as_slice()
                        , socket)
                            .await.expect("cannot send?");
                }

                DetectorOperation::HeartbeatResponse(id) => {
                    self.alive.insert(id);
                }

                DetectorOperation::AliveRequest => {
                    let vec = self.alive.iter().copied().collect();
                    self.socket.send_to(
                        bincode::serialize(
                            &DetectorOperation::AliveInfo(vec)
                        ).unwrap().as_slice()
                        , socket)
                            .await.expect("cannot send?");
                }

                DetectorOperation::AliveInfo(_) => {}
            }
        }
    }
}

/// Called periodically to check send broadcast and update alive processes.
#[async_trait::async_trait]
impl Handler<Tick> for FailureDetectorModule {
    async fn handle(&mut self, _self_ref: &ModuleRef<Self>, _msg: Tick) {
        if self.enabled {
            for (_, addr) in self.addresses.iter() {
                self.socket.send_to(
                    bincode::serialize(
                        &DetectorOperation::HeartbeatRequest
                    ).unwrap().as_slice()
                    , addr
                    ).await.expect("cannot send?");
            }
            self.alive.clear();
        }
    }
}

#[async_trait::async_trait]
impl Handler<Disable> for FailureDetectorModule {
    async fn handle(&mut self, _self_ref: &ModuleRef<Self>, _msg: Disable) {
        self.enabled = false;
    }
}

async fn deserialize_and_forward(
    socket: Arc<UdpSocket>,
    module_ref: ModuleRef<FailureDetectorModule>,
) {
    let mut buffer = vec![0];
    while let Ok((len, sender)) = socket.peek_from(&mut buffer).await {
        if len == buffer.len() {
            buffer.resize(2 * buffer.len(), 0);
        } else {
            socket.recv_from(&mut buffer).await.unwrap();
            match bincode::deserialize(&buffer) {
                Ok(msg) => module_ref.send(DetectorOperationUdp(msg, sender)).await,
                Err(err) => {
                    debug!("Invalid format of detector operation ({})!", err);
                }
            }
        }
    }
}

struct DetectorOperationUdp(DetectorOperation, SocketAddr);

#[derive(Serialize, Deserialize)]
pub enum DetectorOperation {
    /// Request to receive a heartbeat.
    HeartbeatRequest,
    /// Response to heartbeat, contains uuid of the receiver of HeartbeatRequest.
    HeartbeatResponse(Uuid),
    /// Request to receive information about working processes.
    AliveRequest,
    /// Vector of processes which are alive according to AliveRequest receiver.
    AliveInfo(Vec<Uuid>),
}
