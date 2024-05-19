use std::{collections::HashSet, str::FromStr};

use city_common::data::u8bytes::U8Bytes;
use city_crypto::hash::base_types::hash256::Hash256;
use reqwest::blocking::ClientBuilder;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::{
    errors::data_resolver::BTCDataResolverError, introspection::transaction::BTCTransaction,
};

use super::{
    data::{BTCAddress160, BTCTransactionWithVout, BTCUTXO},
    tx::QBitcoinAPISync,
};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, PartialOrd, Eq)]
pub struct BTCLinkRPCConfig {
    pub network: String,
    pub url: String,
    pub user: String,
    pub password: String,
}
impl BTCLinkRPCConfig {
    pub fn new(rpc_url: &str) -> Self {
        let url = url::Url::from_str(rpc_url).unwrap();
        let path = url.path();
        let origin = url.origin().ascii_serialization();

        let rr = url
            .query_pairs()
            .find(|x| x.0.to_ascii_lowercase().eq("network"));

        let network = if rr.is_none() {
            "dogeRegtest".to_string()
        } else {
            rr.unwrap().1.to_string()
        };
        let final_url = format!("{}{}", origin, path);

        let user = url.username().to_string();
        let password = url.password().unwrap_or("").to_string();

        Self {
            url: final_url,
            user,
            password,
            network,
        }
    }
    pub fn has_basic_auth(&self) -> bool {
        !(self.user.is_empty() && self.password.is_empty())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, PartialOrd, Eq)]
pub struct BTCLinkRPCCommand<T> {
    pub jsonrpc: String,
    pub method: String,
    pub params: T,
    pub id: u32,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, PartialOrd, Eq)]
pub struct BTCLinkRPCCommandResponse<R> {
    pub error: Option<String>,
    pub result: Option<R>,
    pub id: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BTCLinkAPI {
    pub rpc_config: BTCLinkRPCConfig,
    pub electrs_url: String,
    pub no_proxy: bool,
}

impl BTCLinkAPI {
    pub fn new(rpc_url: String, electrs_url: String) -> Self {
        Self {
            rpc_config: BTCLinkRPCConfig::new(&rpc_url),
            electrs_url,
            no_proxy: true,
        }
    }
    pub fn new_str(rpc_url: &str, electrs_url: &str) -> Self {
        Self {
            rpc_config: BTCLinkRPCConfig::new(rpc_url),
            electrs_url: electrs_url.to_string(),
            no_proxy: true,
        }
    }
    pub fn new_with_proxy(rpc_url: String, electrs_url: String) -> Self {
        Self {
            rpc_config: BTCLinkRPCConfig::new(&rpc_url),
            electrs_url,
            no_proxy: false,
        }
    }
    pub fn new_str_with_proxy(rpc_url: &str, electrs_url: &str) -> Self {
        Self {
            rpc_config: BTCLinkRPCConfig::new(rpc_url),
            electrs_url: electrs_url.to_string(),
            no_proxy: false,
        }
    }
    pub fn send_command<T: Serialize, R: DeserializeOwned>(
        &self,
        method: &str,
        version: &str,
        params: T,
    ) -> Result<R, BTCDataResolverError> {
        let cmd: BTCLinkRPCCommand<T> = BTCLinkRPCCommand {
            jsonrpc: version.to_string(),
            method: method.to_string(),
            params,
            id: 1,
        };
        let client = if self.no_proxy {
            ClientBuilder::new()
                .no_proxy()
                .build()
                .expect("Client::new()")
        } else {
            ClientBuilder::new().build().expect("Client::new()")
        };

        let base = if self.rpc_config.has_basic_auth() {
            client.post(&self.rpc_config.url).basic_auth(
                self.rpc_config.user.to_string(),
                Some(self.rpc_config.password.to_string()),
            )
        } else {
            client.post(&self.rpc_config.url)
        };
        let result = base
            .json(&cmd)
            .send()
            .map_err(|err| BTCDataResolverError::new(err.to_string()))?;
        let result_text = result
            .text()
            .map_err(|err| BTCDataResolverError::new(err.to_string()))?;
        let json_result = serde_json::from_str::<BTCLinkRPCCommandResponse<R>>(&result_text);
        if json_result.is_err() {
            Err(BTCDataResolverError {
                message: result_text,
            })
        } else {
            let res = json_result.unwrap();
            if res.result.is_some() {
                Ok(res.result.unwrap())
            } else {
                Err(BTCDataResolverError {
                    message: res.error.unwrap_or("error parsing response".to_string()),
                })
            }
        }
    }
    pub fn get_electrs<R: DeserializeOwned>(
        &self,
        path: String,
    ) -> Result<R, BTCDataResolverError> {
        let client = if self.no_proxy {
            ClientBuilder::new()
                .no_proxy()
                .build()
                .expect("Client::new()")
        } else {
            ClientBuilder::new().build().expect("Client::new()")
        };
        let resp = client
            .get(format!("{}/{}", self.electrs_url, path))
            .send()
            .map_err(|err| BTCDataResolverError {
                message: err.to_string(),
            })?
            .error_for_status()
            .map_err(|err| BTCDataResolverError {
                message: err.to_string(),
            })?;
        let text = resp.text().map_err(|err| BTCDataResolverError {
            message: err.to_string(),
        })?;
        Ok(
            serde_json::from_str::<R>(&text).map_err(|err| BTCDataResolverError {
                message: err.to_string(),
            })?,
        )
    }
    pub fn btc_get_raw_transaction(&self, txid: Hash256) -> Result<U8Bytes, BTCDataResolverError> {
        self.send_command("getrawtransaction", "1.0", (txid,))
    }
    pub fn btc_send_raw_transaction(&self, bytes: &[u8]) -> Result<Hash256, BTCDataResolverError> {
        self.send_command("sendrawtransaction", "1.0", (hex::encode(bytes),))
    }
    pub fn btc_get_utxos(&self, address: String) -> Result<Vec<BTCUTXO>, BTCDataResolverError> {
        self.get_electrs(format!("address/{}/utxo", address))
    }
}

impl QBitcoinAPISync for BTCLinkAPI {
    fn get_funding_transactions(
        &self,
        address: BTCAddress160,
    ) -> anyhow::Result<Vec<BTCTransaction>> {
        let utxos = self.btc_get_utxos(address.to_string())?;

        HashSet::<Hash256>::from_iter(utxos.iter().map(|x| x.txid))
            .into_iter()
            .map(|txid| {
                let raw = self.btc_get_raw_transaction(txid)?;
                BTCTransaction::from_bytes(&raw.0)
            })
            .collect::<anyhow::Result<Vec<BTCTransaction>>>()
    }

    fn get_utxos(&self, address: BTCAddress160) -> anyhow::Result<Vec<BTCUTXO>> {
        self.btc_get_utxos(address.to_string())
            .map_err(|e| anyhow::format_err!("{}", e.message))
    }

    fn get_funding_transactions_with_vout(
        &self,
        address: BTCAddress160,
    ) -> anyhow::Result<Vec<BTCTransactionWithVout>> {
        let utxos = self.btc_get_utxos(address.to_string())?;
        let transactions = utxos
            .iter()
            .map(|utxo| {
                let txid = utxo.txid;
                let tx = self.btc_get_raw_transaction(txid)?;
                Ok(BTCTransactionWithVout {
                    transaction: BTCTransaction::from_bytes(&tx.0)?,
                    vout: utxo.vout,
                })
            })
            .collect::<anyhow::Result<Vec<BTCTransactionWithVout>>>()?;
        Ok(transactions)
    }

    fn get_transaction(&self, txid: Hash256) -> anyhow::Result<BTCTransaction> {
        let raw = self.btc_get_raw_transaction(txid)?;
        BTCTransaction::from_bytes(&raw.0)
    }

    fn send_transaction(&self, tx: &BTCTransaction) -> anyhow::Result<Hash256> {
        let bytes = tx.to_bytes();
        let txid = self.btc_send_raw_transaction(&bytes)?;
        Ok(txid)
    }
}
