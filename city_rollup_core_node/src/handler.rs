use std::marker::PhantomData;
use std::net::SocketAddr;
use std::sync::Arc;

use bytes::Bytes;
use city_common::cli::args::RPCServerArgs;
use city_common_circuit::circuits::zk_signature::{
    verify_secp256k1_signature_proof, verify_standard_wrapped_zk_signature_proof,
};
use city_redis_store::{RedisStore, USER_KEYS};
use city_rollup_common::{
    actors::traits::OrchestratorRPCEventSenderSync,
    api::data::block::{
        requested_actions::{
            CityAddWithdrawalRequest, CityClaimDepositRequest, CityRegisterUserRequest,
            CityTokenTransferRequest,
        },
        rpc_request::*,
    },
    qworker::{job_id::QProvingJobDataID, proof_store::QProofStoreWriterSync},
};

use city_rollup_worker_dispatch::implementations::redis::QueueCmd;
use city_rollup_worker_dispatch::implementations::redis::RedisQueue;
use city_rollup_worker_dispatch::{
    implementations::redis::{
        Q_CMD, Q_RPC_ADD_WITHDRAWAL, Q_RPC_CLAIM_DEPOSIT, Q_RPC_REGISTER_USER, Q_RPC_TOKEN_TRANSFER,
    },
    traits::{proving_dispatcher::ProvingDispatcher, proving_worker::ProvingWorkerListener},
};
use city_store::config::C;
use http_body_util::BodyExt;
use http_body_util::Full;
use hyper::body::Incoming;
use hyper::header;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::Method;
use hyper::Request;
use hyper::Response;
use hyper::StatusCode;
use hyper_util::rt::TokioIo;
use jsonrpsee::core::client::ClientT;
use jsonrpsee::http_client::HttpClient;
use jsonrpsee::http_client::HttpClientBuilder;
use plonky2::hash::hash_types::RichField;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
use serde::de::DeserializeOwned;
use serde_json::{json, Value};
use tokio::sync::{mpsc, oneshot};
use tokio::{net::TcpListener, task::spawn_blocking};

use crate::ddos_preventer::{DDoSPreventer, DDOS_THRESHOLD};
use crate::rpc::ErrorCode;
use crate::rpc::ExternalRequestParams;
use crate::rpc::RequestParams;
use crate::rpc::ResponseResult;
use crate::rpc::RpcError;
use crate::rpc::RpcRequest;
use crate::rpc::RpcResponse;
use crate::rpc::Version;

type BoxBody = http_body_util::combinators::BoxBody<Bytes, hyper::Error>;

static NOTFOUND: &[u8] = b"Not Found";
static INDEX_HTML: &str = include_str!("../public/index.html");

#[derive(Clone)]
pub struct CityRollupRPCServerHandler<F: RichField> {
    pub args: RPCServerArgs,
    pub store: RedisStore,
    pub tx_queue: RedisQueue,
    pub api: Arc<HttpClient>,
    _marker: PhantomData<F>,
}

impl<F: RichField> CityRollupRPCServerHandler<F> {
    pub async fn new(args: RPCServerArgs, store: RedisStore) -> anyhow::Result<Self> {
        Ok(Self {
            tx_queue: RedisQueue::new(&args.redis_uri)?,
            api: Arc::new(HttpClientBuilder::default().build(&args.api_server_address)?),
            args,
            store,
            _marker: PhantomData,
        })
    }

    pub async fn handle(
        &mut self,
        req: Request<hyper::body::Incoming>,
    ) -> anyhow::Result<Response<BoxBody>> {
        match (req.method(), req.uri().path()) {
            (&Method::GET, "/editor") => Ok(editor()),
            (&Method::POST, "/") => self.rpc(req).await,
            (&Method::OPTIONS, "/") => self.preflight(req).await,
            _ => Ok(not_found()),
        }
    }

    pub async fn rpc(&mut self, req: Request<Incoming>) -> anyhow::Result<Response<BoxBody>> {
        let whole_body = req.collect().await?.to_bytes();
        let data = serde_json::from_slice::<RpcRequest<RequestParams<F>>>(&whole_body);
        use RequestParams::*;
        let res = match data {
            Ok(RpcRequest {
                request: TokenTransfer(req),
                ..
            }) => self.token_transfer(req).await.map(|r| json!(r)),
            Ok(RpcRequest {
                request: ClaimDeposit(req),
                ..
            }) => self.claim_deposit(req).await.map(|r| json!(r)),
            Ok(RpcRequest {
                request: AddWithdrawal(req),
                ..
            }) => self.add_withdrawal(req).await.map(|r| json!(r)),
            Ok(RpcRequest {
                request: RegisterUser(req),
                ..
            }) => self.register_user(req).map(|r| json!(r)),
            Ok(RpcRequest {
                request: ProduceBlock,
                ..
            }) => self.produce_block().map(|r| json!(r)),
            Ok(RpcRequest {
                request: GatherRegisterUser,
                ..
            }) => self.gather_register_user().map(|r| json!(r)),
            Ok(RpcRequest {
                request: GatherClaimDeposit(checkpoint_id),
                ..
            }) => self.gather_claim_deposit(checkpoint_id).map(|r| json!(r)),
            Ok(RpcRequest {
                request: GatherAddWithdrawal(checkpoint_id),
                ..
            }) => self.gather_add_withdrawal(checkpoint_id).map(|r| json!(r)),
            Ok(RpcRequest {
                request: GatherTokenTransfer(checkpoint_id),
                ..
            }) => self.gather_token_transfer(checkpoint_id).map(|r| json!(r)),
            Err(_) => {
                let request =
                    serde_json::from_slice::<RpcRequest<ExternalRequestParams>>(&whole_body)?
                        .request;
                self.api
                    .request(&request.method, request.params)
                    .await
                    .map_err(anyhow::Error::from)
                    .map(|r: serde_json::Value| json!(r))
            }
        }
        .map_or_else(
            |_| ResponseResult::<Value>::Error(RpcError::from(ErrorCode::InternalError)),
            |r| ResponseResult::<Value>::Success(r),
        );

        let response = RpcResponse {
            jsonrpc: Version::V2,
            id: None,
            result: res,
        };

        let code = match response.result {
            ResponseResult::Success(_) => StatusCode::OK,
            ResponseResult::Error(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        Ok(Response::builder()
            .status(code)
            .header(header::CONTENT_TYPE, "application/json")
            .body(full(serde_json::to_vec(&response)?))?)
    }
    pub async fn preflight(&self, req: Request<Incoming>) -> anyhow::Result<Response<BoxBody>> {
        let _whole_body = req.collect().await?;
        let response = Response::builder()
            .status(StatusCode::OK)
            .header("Access-Control-Allow-Origin", "*")
            .header("Access-Control-Allow-Headers", "*")
            .header("Access-Control-Allow-Methods", "POST, GET, OPTIONS")
            .body(BoxBody::default())?;
        Ok(response)
    }
    fn register_user(&mut self, req: CityRegisterUserRPCRequest<F>) -> Result<(), anyhow::Error> {
        let user_pub_key = req.public_key.to_string();
        let mut connection = self.store.get_connection()?;
        // use HEXISTS command to check if the field exists in the hash table
        let exists: bool = redis::cmd("HEXISTS")
            .arg(USER_KEYS)
            .arg(user_pub_key.as_str())
            .query(&mut connection)?;
        if exists {
            // if the public key already exists in the hash table, return an error
            return Err(anyhow::anyhow!("Public key already registered"));
        }
        self.notify_rpc_register_user(&req)?;
        Ok(())
    }
    pub fn flush_rpc_requests<T: DeserializeOwned>(
        &mut self,
        topic: &'static str,
    ) -> anyhow::Result<Vec<T>> {
        Ok(self
            .tx_queue
            .pop_all(topic)?
            .into_iter()
            .map(|v| Ok(serde_json::from_slice(&v)?))
            .collect::<anyhow::Result<Vec<_>>>()?)
    }
    fn gather_register_user(&mut self) -> Result<Vec<CityRegisterUserRequest<F>>, anyhow::Error> {
        let ret =
            match self.flush_rpc_requests::<CityRegisterUserRPCRequest<F>>(Q_RPC_REGISTER_USER) {
                Ok(v) => Ok(v
                    .par_iter()
                    .map(|req| CityRegisterUserRequest::<F>::new(req.public_key))
                    .collect::<Vec<_>>()),
                Err(e) => Err(e),
            };
        ret
    }
    fn gather_claim_deposit(
        &mut self,
        checkpoint_id: u64,
    ) -> Result<Vec<CityClaimDepositRequest>, anyhow::Error> {
        //todo: have rpc_node id from somewhere
        let rpc_node_id = 0;
        let reqs = self.flush_rpc_requests::<CityClaimDepositRPCRequest>(Q_RPC_CLAIM_DEPOSIT)?;
        let deposit = reqs
            .par_iter()
            .enumerate()
            .map(|(i, req)| {
                let count = i as u32;
                let signature_proof_id = QProvingJobDataID::claim_deposit_l1_signature_proof(
                    rpc_node_id,
                    checkpoint_id,
                    count, //it's index indeed
                );
                let mut ps = self.store.clone();
                ps.set_bytes_by_id(signature_proof_id, &req.signature_proof)?;

                Ok(CityClaimDepositRequest::new(
                    req.user_id,
                    req.deposit_id,
                    req.value,
                    req.txid,
                    req.public_key,
                    signature_proof_id,
                ))
            })
            .collect::<anyhow::Result<Vec<CityClaimDepositRequest>>>()?;
        Ok(deposit)
    }
    fn gather_add_withdrawal(
        &mut self,
        checkpoint_id: u64,
    ) -> Result<Vec<CityAddWithdrawalRequest>, anyhow::Error> {
        let rpc_node_id = 0;
        let reqs = self.flush_rpc_requests::<CityAddWithdrawalRPCRequest>(Q_RPC_ADD_WITHDRAWAL)?;
        let withdraw = reqs
            .par_iter()
            .enumerate()
            .map(|(i, req)| {
                let count = i as u32;
                let signature_proof_id = QProvingJobDataID::withdrawal_signature_proof(
                    rpc_node_id,
                    checkpoint_id,
                    count, //it's index indeed
                );
                let mut ps = self.store.clone();
                ps.set_bytes_by_id(signature_proof_id, &req.signature_proof)?;

                Ok(CityAddWithdrawalRequest::new(
                    req.user_id,
                    req.value,
                    req.nonce,
                    req.destination_type,
                    req.destination,
                    signature_proof_id,
                ))
            })
            .collect::<anyhow::Result<Vec<CityAddWithdrawalRequest>>>()?;
        Ok(withdraw)
    }
    fn gather_token_transfer(
        &mut self,
        checkpoint_id: u64,
    ) -> Result<Vec<CityTokenTransferRequest>, anyhow::Error> {
        let rpc_node_id = 0;
        let reqs = self.flush_rpc_requests::<CityTokenTransferRPCRequest>(Q_RPC_TOKEN_TRANSFER)?;
        let transfer = reqs
            .par_iter()
            .enumerate()
            .map(|(i, req)| {
                let count = i as u32;
                let signature_proof_id = QProvingJobDataID::transfer_signature_proof(
                    rpc_node_id,
                    checkpoint_id,
                    count, //it's index indeed
                );
                let mut ps = self.store.clone();
                ps.set_bytes_by_id(signature_proof_id, &req.signature_proof)?;

                Ok(CityTokenTransferRequest::new(
                    req.user_id,
                    req.to,
                    req.value,
                    req.nonce,
                    signature_proof_id,
                ))
            })
            .collect::<anyhow::Result<Vec<CityTokenTransferRequest>>>()?;
        Ok(transfer)
    }
    fn produce_block(&mut self) -> Result<(), anyhow::Error> {
        Ok(self.notify_rpc_produce_block()?)
    }

    async fn add_withdrawal(
        &mut self,
        req: CityAddWithdrawalRPCRequest,
    ) -> Result<(), anyhow::Error> {
        self.verify_signature_proof(req.user_id, req.signature_proof.clone())
            .await?;
        self.notify_rpc_add_withdrawal(&req)?;
        Ok(())
    }

    async fn claim_deposit(
        &mut self,
        req: CityClaimDepositRPCRequest,
    ) -> Result<(), anyhow::Error> {
        self.verify_signature_proof_secp256k1(req.user_id, req.signature_proof.clone())
            .await?;
        self.notify_rpc_claim_deposit(&req)?;
        Ok(())
    }

    async fn token_transfer(
        &mut self,
        req: CityTokenTransferRPCRequest,
    ) -> Result<(), anyhow::Error> {
        self.verify_signature_proof(req.user_id, req.signature_proof.clone())
            .await?;
        self.notify_rpc_token_transfer(&req)?;
        Ok(())
    }

    async fn verify_signature_proof_secp256k1(
        &self,
        _user_id: u64,
        signature_proof: Vec<u8>,
    ) -> anyhow::Result<()> {
        spawn_blocking(move || {
            verify_secp256k1_signature_proof::<C, { city_store::config::D }>(
                Default::default(),
                signature_proof,
            )?;
            Ok::<_, anyhow::Error>(())
        })
        .await??;
        Ok(())
    }
    async fn verify_signature_proof(
        &self,
        _user_id: u64,
        signature_proof: Vec<u8>,
    ) -> anyhow::Result<()> {
        spawn_blocking(move || {
            verify_standard_wrapped_zk_signature_proof::<C, { city_store::config::D }>(
                Default::default(),
                signature_proof,
            )?;
            Ok::<_, anyhow::Error>(())
        })
        .await??;
        Ok(())
    }
}

impl<F: RichField> OrchestratorRPCEventSenderSync<F> for CityRollupRPCServerHandler<F> {
    fn notify_rpc_claim_deposit(
        &mut self,
        event: &CityClaimDepositRPCRequest,
    ) -> anyhow::Result<()> {
        self.tx_queue.dispatch(Q_RPC_CLAIM_DEPOSIT, event.clone())?;
        Ok(())
    }

    fn notify_rpc_register_user(
        &mut self,
        event: &CityRegisterUserRPCRequest<F>,
    ) -> anyhow::Result<()> {
        self.tx_queue.dispatch(Q_RPC_REGISTER_USER, event.clone())?;
        Ok(())
    }

    fn notify_rpc_add_withdrawal(
        &mut self,
        event: &CityAddWithdrawalRPCRequest,
    ) -> anyhow::Result<()> {
        self.tx_queue
            .dispatch(Q_RPC_ADD_WITHDRAWAL, event.clone())?;
        Ok(())
    }

    fn notify_rpc_token_transfer(
        &mut self,
        event: &CityTokenTransferRPCRequest,
    ) -> anyhow::Result<()> {
        self.tx_queue
            .dispatch(Q_RPC_TOKEN_TRANSFER, event.clone())?;
        Ok(())
    }

    fn notify_rpc_produce_block(&mut self) -> anyhow::Result<()> {
        self.tx_queue.dispatch(Q_CMD, QueueCmd::ProduceBlock)?;
        Ok(())
    }
}

pub async fn run<F: RichField>(args: RPCServerArgs) -> anyhow::Result<()> {
    let addr: SocketAddr = args.rollup_rpc_address.parse()?;

    let listener = TcpListener::bind(addr).await?;
    tracing::info!("Listening on http://{}", addr);
    let store = RedisStore::new(&args.redis_uri)?;
    let handler = CityRollupRPCServerHandler::<F>::new(args, store).await?;
    let (tx, mut rx) = mpsc::channel::<(SocketAddr, oneshot::Sender<bool>)>(DDOS_THRESHOLD);

    // start the receiver thread
    tokio::spawn(async move {
        // create a DDoS preventer in the receiver thread so that it can keep track of the addresses
        let mut ddos_preventer = DDoSPreventer::new();

        while let Some((addr, responder)) = rx.recv().await {
            let allow = ddos_preventer.should_allow(addr);
            // send back the result
            let _ = responder.send(allow);
        }
    });

    loop {
        let (stream, addr) = listener.accept().await?;
        let io = TokioIo::new(stream);
        let handler = handler.clone();

        // create a channel to communicate with the receiver thread
        let (resp_tx, resp_rx) = oneshot::channel();

        // send the addr to the receiver thread
        if let Err(e) = tx.send((addr, resp_tx)).await {
            tracing::error!("Failed to send address to DDoS prevention thread: {:?}", e);
            return Err(anyhow::anyhow!("Failed to send address"));
        }

        // receive the result from the receiver thread
        if let Ok(allow) = resp_rx.await {
            if !allow {
                tracing::warn!("Connection from {} blocked due to DDoS protection", addr);
                continue;
            }
        } else {
            tracing::error!("Failed to receive response from DDoS prevention thread");
            return Err(anyhow::anyhow!("Failed to receive response"));
        }

        tokio::task::spawn(async move {
            // TODO: should remove the extra clone
            let service = service_fn(|req| async { handler.clone().handle(req).await });

            if let Err(err) = http1::Builder::new().serve_connection(io, service).await {
                tracing::info!("Failed to serve connection: {:?}", err);
            }
        });
    }
}

fn not_found() -> Response<BoxBody> {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Full::new(NOTFOUND.into()).map_err(|e| match e {}).boxed())
        .unwrap()
}

fn editor() -> Response<BoxBody> {
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/html")
        .body(Full::new(INDEX_HTML.into()).map_err(|e| match e {}).boxed())
        .unwrap()
}

fn full<T: Into<Bytes>>(chunk: T) -> BoxBody {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}
