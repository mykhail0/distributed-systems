use async_channel::Sender;
use executor::{Handler, Message, ModuleRef};
use log::error;
use std::future::Future;
use std::pin::Pin;
use uuid::Uuid;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Debug)]
pub(crate) enum ProductType {
    Electronics,
    Toys,
    Books,
}

#[derive(Clone)]
pub(crate) struct StoreMsg {
    sender: ModuleRef<CyberStore2047>,
    content: StoreMsgContent,
}

#[derive(Clone, Debug)]
pub(crate) enum StoreMsgContent {
    /// Transaction Manager initiates voting for the transaction.
    RequestVote(Transaction),
    /// If every process is ok with transaction, TM issues commit.
    Commit,
    /// System-wide abort.
    Abort,
}

impl From<TwoPhaseResult> for StoreMsgContent {
    fn from(res: TwoPhaseResult) -> Self {
        match res {
            TwoPhaseResult::Ok => Self::Commit,
            TwoPhaseResult::Abort => Self::Abort,
        }
    }
}

#[derive(Clone)]
pub(crate) struct NodeMsg {
    content: NodeMsgContent,
}

#[derive(Clone, Debug)]
pub(crate) enum NodeMsgContent {
    /// Process replies to TM whether it can/cannot commit the transaction.
    RequestVoteResponse(TwoPhaseResult),
    /// Process acknowledges to TM committing/aborting the transaction.
    FinalizationAck,
}

pub(crate) struct TransactionMessage {
    /// Request to change price.
    pub(crate) transaction: Transaction,

    /// Called after 2PC completes (i.e., the transaction was decided to be
    /// committed/aborted by CyberStore2047). This must be called after responses
    /// from all processes acknowledging commit or abort are collected.
    pub(crate) completed_callback:
        Box<dyn FnOnce(TwoPhaseResult) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(crate) enum TwoPhaseResult {
    Ok,
    Abort,
}

#[derive(Copy, Clone)]
pub(crate) struct Product {
    pub(crate) identifier: Uuid,
    pub(crate) pr_type: ProductType,
    pub(crate) price: u64,
}

#[derive(Copy, Clone, Debug)]
pub(crate) struct Transaction {
    pub(crate) pr_type: ProductType,
    pub(crate) shift: i32,
}

pub(crate) struct ProductPriceQuery {
    pub(crate) product_ident: Uuid,
    pub(crate) result_sender: Sender<ProductPrice>,
}

pub(crate) struct ProductPrice(pub(crate) Option<u64>);

/// Message which disables a node. Used for testing.
pub(crate) struct Disable;

// Transaction's state when there is one in progress.
struct TransactionState {
    ok: usize,
    callback: Box<dyn FnOnce(TwoPhaseResult) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send>,
    // This field is None during the first phase and Some during the second.
    result: Option<TwoPhaseResult>,
}

/// CyberStore2047.
/// This structure serves as TM.
pub(crate) struct CyberStore2047 {
    nodes: Vec<ModuleRef<Node>>,
    state: Option<TransactionState>,
}

impl CyberStore2047 {
    pub(crate) fn new(nodes: Vec<ModuleRef<Node>>) -> Self {
        Self { nodes, state: None }
    }

    async fn broadcast<T>(&mut self, msg: T)
    where
        Node: Handler<T>,
        T: Message + Clone,
    {
        for node in self.nodes.iter() {
            node.send(msg.clone()).await;
        }
    }
}

/// Node of CyberStore2047.
/// This structure serves as a process of the distributed system.
pub(crate) struct Node {
    products: Vec<Product>,
    pending_transaction: Option<Transaction>,
    enabled: bool,
}

impl Node {
    pub(crate) fn new(products: Vec<Product>) -> Self {
        Self {
            products,
            pending_transaction: None,
            enabled: true,
        }
    }
}

#[async_trait::async_trait]
impl Handler<NodeMsg> for CyberStore2047 {
    async fn handle(&mut self, self_ref: &ModuleRef<Self>, msg: NodeMsg) {
        if let Some(state) = self.state.as_mut() {
            match msg.content {
                NodeMsgContent::RequestVoteResponse(res) if state.result.is_none() => {
                    let start_second_phase = if let TwoPhaseResult::Ok = res {
                        state.ok += 1;
                        state.ok == self.nodes.len()
                    } else {
                        true
                    };

                    if start_second_phase {
                        state.ok = 0;
                        state.result = Some(res);
                        self.broadcast(StoreMsg {
                            sender: self_ref.clone(),
                            content: res.into(),
                        })
                        .await;
                    }
                }

                NodeMsgContent::FinalizationAck if let Some(result) = state.result => {
                    state.ok += 1;
                    if state.ok == self.nodes.len() {
                        let state = self.state.take().unwrap();
                        (state.callback)(result).await;
                    }
                }
                NodeMsgContent::RequestVoteResponse(_) => {
                    error!("Receiving `RequestVoteResponse` in the second phase");
                }
                NodeMsgContent::FinalizationAck => {
                    error!("Receiving `FinalizationAck` in the first phase");
                }
            }
        } else {
            error!("Receiving `NodeMsg` with no transaction in progress");
        }
    }
}

#[async_trait::async_trait]
impl Handler<StoreMsg> for Node {
    async fn handle(&mut self, _self_ref: &ModuleRef<Self>, msg: StoreMsg) {
        if self.enabled {
            msg.sender
                .send(NodeMsg {
                    content: match msg.content {
                        StoreMsgContent::RequestVote(trans) => {
                            // Assuming perfect links.
                            assert!(self.pending_transaction.is_none());
                            let Transaction { pr_type, shift } = trans;
                            let result = if self.products.iter().any(|p| {
                                p.pr_type == pr_type
                                    && p.price
                                        .checked_add_signed(i64::from(shift))
                                        .is_none_or(|price| price == 0)
                            }) {
                                TwoPhaseResult::Abort
                            } else {
                                TwoPhaseResult::Ok
                            };
                            if result == TwoPhaseResult::Ok {
                                self.pending_transaction = Some(trans);
                            }
                            NodeMsgContent::RequestVoteResponse(result)
                        }

                        StoreMsgContent::Commit => {
                            let transaction = self
                                .pending_transaction
                                .take()
                                .expect("Cannot commit without a transaction");
                            let shift = i64::from(transaction.shift);
                            for p in self.products.iter_mut() {
                                if p.pr_type == transaction.pr_type {
                                    p.price = p
                                        .price
                                        .checked_add_signed(shift)
                                        .expect("Validated in `RequestVote`");
                                }
                            }
                            NodeMsgContent::FinalizationAck
                        }

                        StoreMsgContent::Abort => {
                            self.pending_transaction.take();
                            NodeMsgContent::FinalizationAck
                        }
                    },
                })
                .await;
        }
    }
}

#[async_trait::async_trait]
impl Handler<ProductPriceQuery> for Node {
    async fn handle(&mut self, _self_ref: &ModuleRef<Self>, msg: ProductPriceQuery) {
        if self.enabled {
            let _ = msg
                .result_sender
                .send(ProductPrice(
                    self.products
                        .iter()
                        .find(|p| p.identifier == msg.product_ident)
                        .map(|p| p.price),
                ))
                .await;
        }
    }
}

#[async_trait::async_trait]
impl Handler<Disable> for Node {
    async fn handle(&mut self, _self_ref: &ModuleRef<Self>, _msg: Disable) {
        self.enabled = false;
    }
}

#[async_trait::async_trait]
impl Handler<TransactionMessage> for CyberStore2047 {
    async fn handle(&mut self, self_ref: &ModuleRef<Self>, msg: TransactionMessage) {
        // There shall be only one transaction at a time.
        assert!(self.state.is_none());
        let TransactionMessage {
            transaction,
            completed_callback,
        } = msg;
        if self.nodes.is_empty() {
            completed_callback(TwoPhaseResult::Ok).await;
        } else {
            self.state = Some(TransactionState {
                ok: 0,
                callback: completed_callback,
                result: None,
            });
            self.broadcast(StoreMsg {
                sender: self_ref.clone(),
                content: StoreMsgContent::RequestVote(transaction),
            })
            .await;
        }
    }
}
