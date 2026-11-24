use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use log::debug;
use postcard::to_allocvec;
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
    ident: Uuid,
    alive: HashSet<Uuid>,
    unsuspected: HashSet<Uuid>,
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
                ident,
                alive: all_idents.clone(),
                unsuspected: all_idents,
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
                    self.socket
                        .send_to(
                            to_allocvec(&DetectorOperation::HeartbeatResponse(self.ident))
                                .unwrap()
                                .as_slice(),
                            socket,
                        )
                        .await
                        .expect("cannot send");
                }

                DetectorOperation::HeartbeatResponse(id) => {
                    self.alive.insert(id);
                }

                DetectorOperation::AliveRequest => {
                    let vec = self.unsuspected.iter().copied().collect();
                    self.socket
                        .send_to(
                            to_allocvec(&DetectorOperation::AliveInfo(vec))
                                .unwrap()
                                .as_slice(),
                            socket,
                        )
                        .await
                        .expect("cannot send");
                }

                DetectorOperation::AliveInfo(_) => {}
            }
        }
    }
}

/// Called periodically to update alive processes.
#[async_trait::async_trait]
impl Handler<Tick> for FailureDetectorModule {
    async fn handle(&mut self, _self_ref: &ModuleRef<Self>, _msg: Tick) {
        if self.enabled {
            let request = to_allocvec(&DetectorOperation::HeartbeatRequest).unwrap();
            for (id, addr) in self.addresses.iter() {
                if self.alive.contains(id) {
                    self.unsuspected.insert(*id);
                } else {
                    self.unsuspected.remove(id);
                }
                // If cannot send then the receiver process failed, but not
                // changing this because of how `deserialize_and_forward` works.
                self.socket
                    .send_to(&request, addr)
                    .await
                    .expect("cannot send");
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
            match postcard::from_bytes(&buffer) {
                Ok(msg) => module_ref.send(DetectorOperationUdp(msg, sender)).await,
                Err(err) => {
                    debug!("Invalid format of detector operation ({})!", err);
                }
            }
        }
    }
}

struct DetectorOperationUdp(DetectorOperation, SocketAddr);

#[derive(Serialize, Deserialize, Debug)]
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
