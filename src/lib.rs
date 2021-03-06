extern crate exonum_jsonrpc;
#[macro_use]
extern crate serde_derive;
extern crate serde;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate log;
extern crate failure;
#[macro_use] 
extern crate failure_derive;
#[macro_use] 
extern crate display_derive;

use std::collections::BTreeMap;
use std::io;

use serde::{Serialize, Deserialize};
use serde_json::value::Value;

use exonum_jsonrpc::client::Client as RpcClient;
pub use exonum_jsonrpc::error::Error as RpcError;

#[derive(Fail, Debug, Display)]
pub enum Error {
    #[display(fmt = "No information. {}", _0)]
    NoInformation(String),
    #[display(fmt = "Memory pool error. {}", _0)]
    Memory(String),
    #[display(fmt = "Transaction is incorrect. {}", _0)]
    TransactionIncorrect(String),
    #[display(fmt = "Transaction rejected. {}", _0)]
    TransactionRejected(String),
    #[display(fmt = "Insufficient funds.")]
    InsufficientFunds,
    #[display(fmt = "Transaction already in chain.")]
    TransactionAlreadyInChain,
    #[display(fmt = "{}", _0)]
    Rpc(RpcError),
    #[display(fmt = "{}", _0)]
    Other(io::Error)
}

pub type Result<T> = ::std::result::Result<T, Error>;
pub type Params = Vec<Value>;

impl Error {
    pub fn incorrect_transaction<S: Into<String>>(s: S) -> Error {
        Error::TransactionIncorrect(s.into())
    }
}

impl From<RpcError> for Error {
    fn from(e: RpcError) -> Error {
        match e {
            exonum_jsonrpc::Error::Rpc(value) => {
                if let Some(code) = value.pointer("/code").and_then(Value::as_i64) {
                    let msg = value
                        .pointer("/message")
                        .and_then(Value::as_str)
                        .unwrap_or_else(|| "")
                        .into();

                    match code {
                        -5 => return Error::NoInformation(msg),
                        -6 => return Error::InsufficientFunds,
                        -7 => return Error::Memory(msg),
                        -25 => return Error::TransactionIncorrect(msg),
                        -26 => return Error::TransactionRejected(msg),
                        -27 => return Error::TransactionAlreadyInChain,
                        _ => {}
                    }
                }
                Error::Rpc(RpcError::Rpc(value))
            }
            e => Error::Rpc(e),
        }
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::Other(e)
    }
}

pub struct Client {
    inner: RpcClient,
}

impl std::fmt::Debug for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("BitcoinRpcClient").finish()
    }
}

#[derive(Clone, Deserialize, Debug)]
pub struct Info {
    pub version: u32,
    pub protocolversion: u32,
    pub walletversion: u32,
    pub balance: f64,
    pub blocks: u64,
    pub timeoffset: u64,
    pub connections: u32,
    pub proxy: String,
    pub difficulty: f64,
    pub testnet: bool,
    pub keypoololdest: u64,
    pub keypoolsize: u64,
    pub paytxfee: f64,
    pub relayfee: f64,
    pub errors: String,
}

#[derive(Clone, Deserialize, Debug)]
pub struct AddressInfo {
    pub isvalid: bool,
    pub address: String,
    #[serde(rename = "scriptPubKey")]
    pub script_pubkey: String,
    pub ismine: bool,
    pub iswatchonly: bool,
    pub isscript: bool,
    pub pubkey: String,
    pub iscompressed: bool,
    pub account: Option<String>,
    pub hdkeypath: String,
    pub hdmasterkeyid: String,
}

#[derive(Clone, Deserialize, Debug, PartialEq)]
pub struct MultiSig {
    pub address: String,
    #[serde(rename = "redeemScript")]
    pub redeem_script: String,
}

#[derive(Clone, Deserialize, Debug, PartialEq)]
pub struct ScriptSig {
    pub asm: String,
    pub hex: String,
}

#[derive(Clone, Deserialize, Debug, PartialEq)]
pub struct ScriptPubKey {
    pub asm: String,
    pub hex: String,
    #[serde(rename = "reqSigs")]
    pub req_sigs: Option<u64>,
    #[serde(rename = "type")]
    pub key_type: String,
    pub addresses: Option<Vec<String>>,
}

// TODO use TxIn from bitcoin crate
#[derive(Clone, Deserialize, Debug)]
pub struct TxIn {
    pub txid: String,
    pub vout: u32,
    #[serde(rename = "scriptSig")]
    pub script_sig: ScriptSig,
    pub sequence: u64,
    pub txinwitness: Option<Vec<String>>,
}

#[derive(Clone, Deserialize, Debug)]
pub struct TxOut {
    pub value: f64,
    pub n: u32,
    #[serde(rename = "scriptPubKey")]
    pub script_pubkey: ScriptPubKey,
}

#[derive(Clone, Deserialize, Debug)]
pub struct RawTransactionInfo {
    pub hex: Option<String>,
    pub txid: String,
    pub hash: String,
    pub size: u64,
    pub vsize: u64,
    pub version: u32,
    pub locktime: u32,
    pub vin: Vec<TxIn>,
    pub vout: Vec<TxOut>,
    pub confirmations: Option<u64>,
}

#[derive(Clone, Deserialize, Debug)]
pub struct UnspentTransactionInfo {
    pub txid: String,
    pub vout: u32,
    pub address: String,
    pub account: String,
    #[serde(rename = "scriptPubKey")]
    pub script_pubkey: String,
    #[serde(rename = "redeemScript")]
    pub redeem_script: Option<String>,
    pub amount: f64,
    pub confirmations: u64,
    pub spendable: bool,
    pub solvable: bool,
}

#[derive(Clone, Serialize, Debug)]
pub struct DependentOutput {
    pub txid: String,
    pub vout: u32,
    #[serde(rename = "scriptPubKey")]
    pub script_pubkey: String,
    #[serde(rename = "redeemScript")]
    pub redeem_script: String,
}

#[derive(Clone, Deserialize, Debug)]
pub struct SignTxOutput {
    pub hex: String,
    pub complete: bool,
}

#[derive(Clone, Serialize, Debug)]
pub struct TransactionInput {
    pub txid: String,
    pub vout: u32,
    pub sequence: Option<u64>,
}

#[derive(Clone, Deserialize, Debug)]
pub struct TransactionOutput {
    pub address: String,
    pub value: String,
}

#[derive(Clone, Deserialize, Debug)]
pub struct TransactionInfo {
    pub address: Option<String>,
    pub vout: u32,
    pub confirmations: u64,
    pub txid: String,
    pub abandoned: Option<bool>,
    pub time: u64,
}

#[derive(Debug)]
struct RpcRequest {
    method: String,
    params: Params,
    response: Result<Value>,
}

impl Client {
    pub fn new<S>(url: S, user: Option<String>, password: Option<String>) -> Client
    where
        S: Into<String>,
    {
        Client { inner: RpcClient::new(url.into(), user.map(Into::into), password.map(Into::into)) }
    }

    pub fn url(&self) -> &str {
        self.inner.url()
    }
    pub fn password(&self) -> &Option<String> {
        self.inner.password()
    }
    pub fn username(&self) -> &Option<String> {
        self.inner.username()
    }

    fn request<T>(&self, method: &str, params: Params) -> Result<T>
    where
        for<'de> T: Deserialize<'de>,
    {
        let request = self.inner.build_request(method.into(), params);
        let response = self.inner.send_request(&request)?;
        trace!(
            "{:#?}",
            RpcRequest {
                method: request.method.clone(),
                params: request.params.clone(),
                response: response.clone().into_result::<Value>().map_err(Error::from),
            }
        );
        response.into_result::<T>().map_err(Error::from)
    }
}

// public api part
impl Client {
    pub fn getinfo(&self) -> Result<Info> {
        self.request("getinfo", Vec::new())
    }

    pub fn getnewaddress(&self) -> Result<String> {
        self.request("getnewaddress", vec![])
    }

    pub fn validateaddress(&self, addr: &str) -> Result<AddressInfo> {
        self.request("validateaddress", vec![Value::String(addr.to_owned())])
    }

    pub fn createmultisig<V: AsRef<[String]>>(&self, signs: u8, addrs: V) -> Result<MultiSig> {
        let n = serde_json::to_value(signs).unwrap();
        let addrs = serde_json::to_value(addrs.as_ref()).unwrap();
        self.request("createmultisig", vec![n, addrs])
    }

    pub fn sendtoaddress(&self, addr: &str, amount: &str) -> Result<String> {
        let params = vec![
            serde_json::to_value(addr).unwrap(),
            serde_json::to_value(amount).unwrap(),
        ];
        self.request("sendtoaddress", params)
    }

    pub fn getrawtransaction(&self, txid: &str) -> Result<String> {
        let params = json!([txid, 0]).as_array().cloned().unwrap();
        self.request("getrawtransaction", params)
    }

    pub fn getrawtransaction_verbose(&self, txid: &str) -> Result<RawTransactionInfo> {
        let params = json!([txid, 1]).as_array().cloned().unwrap();
        self.request("getrawtransaction", params)
    }

    pub fn createrawtransaction<T, O>(
        &self,
        transactions: T,
        outputs: O,
        data: Option<String>,
    ) -> Result<String>
    where
        T: AsRef<[TransactionInput]>,
        O: AsRef<[TransactionOutput]>,
    {
        let mut map = BTreeMap::new();
        map.extend(outputs.as_ref().iter().map(|x| {
            (x.address.clone(), x.value.clone())
        }));
        if let Some(data) = data {
            map.insert("data".into(), data);
        }

        let params = json!([transactions.as_ref(), map])
            .as_array()
            .cloned()
            .unwrap();
        self.request("createrawtransaction", params)
    }

    pub fn dumpprivkey(&self, pub_key: &str) -> Result<String> {
        let params = json!([pub_key]).as_array().cloned().unwrap();
        self.request("dumpprivkey", params)
    }

    pub fn signrawtransaction<O, K>(
        &self,
        txhex: &str,
        outputs: O,
        priv_keys: K,
    ) -> Result<SignTxOutput>
    where
        O: AsRef<[DependentOutput]>,
        K: AsRef<[String]>,
    {
        let params = json!([txhex, outputs.as_ref(), priv_keys.as_ref()])
            .as_array()
            .cloned()
            .unwrap();
        self.request("signrawtransaction", params)
    }

    pub fn sendrawtransaction(&self, txhex: &str) -> Result<String> {
        self.request(
            "sendrawtransaction",
            vec![serde_json::to_value(txhex).unwrap()],
        )
    }

    pub fn decoderawtransaction(&self, txhex: &str) -> Result<RawTransactionInfo> {
        self.request(
            "decoderawtransaction",
            vec![serde_json::to_value(txhex).unwrap()],
        )
    }

    pub fn addwitnessaddress(&self, addr: &str) -> Result<String> {
        self.request(
            "addwitnessaddress",
            vec![serde_json::to_value(addr).unwrap()],
        )
    }

    pub fn listtransactions(
        &self,
        count: u32,
        from: u32,
        include_watch_only: bool,
    ) -> Result<Vec<TransactionInfo>> {
        let params = json!(["*", count, from, include_watch_only])
            .as_array()
            .cloned()
            .unwrap();
        self.request("listtransactions", params)
    }

    pub fn listunspent<V: AsRef<str> + Serialize>(
        &self,
        min_confirmations: u32,
        max_confirmations: u32,
        addresses: &[V],
    ) -> Result<Vec<UnspentTransactionInfo>> {
        let params = json!([min_confirmations, max_confirmations, addresses])
            .as_array()
            .cloned()
            .unwrap();
        self.request("listunspent", params)
    }

    pub fn importaddress(&self, addr: &str, label: &str, rescan: bool, p2sh: bool) -> Result<()> {
        let params = json!([addr, label, rescan, p2sh])
            .as_array()
            .cloned()
            .unwrap();
        // special case for decode {"result":null}
        let r: Result<Option<bool>> = self.request("importaddress", params);
        match r {
            Ok(_) |
            Err(Error::Rpc(RpcError::NoErrorOrResult)) => Ok(()),
            Err(e) => Err(e),
        }
    }

    pub fn generate(&self, nblocks: u64, maxtries: u64) -> Result<Vec<String>> {
        let params = json!([nblocks, maxtries]).as_array().cloned().unwrap();
        self.request("generate", params)
    }

    pub fn generatetoaddress(
        &self,
        nblocks: u64,
        addr: &str,
        maxtries: u64,
    ) -> Result<Vec<String>> {
        let params = json!([nblocks, addr, maxtries])
            .as_array()
            .cloned()
            .unwrap();
        self.request("generatetoaddress", params)
    }

    pub fn stop(&self) -> Result<String> {
        self.request("stop", vec![])
    }

    pub fn getreceivedbyaddress(&self, addr: &str, minconf: u64) -> Result<f64> {
        let params = json!([addr, minconf]).as_array().cloned().unwrap();
        self.request("getreceivedbyaddress", params)
    }

    pub fn getblockcount(&self) -> Result<u64> {
        self.request("getblockcount", vec![])
    }

    pub fn getbestblockhash(&self) -> Result<String> {
        self.request("getbestblockhash", vec![])
    }

    pub fn getblockhash(&self, height: u64) -> Result<String> {
        let params = json!([height])
            .as_array()
            .cloned()
            .unwrap();
        self.request("getblockhash", params)
    }

    pub fn getblock<S: AsRef<str> + Serialize>(&self, hash: S) -> Result<String> {
        let params = json!([hash.as_ref(), 0])
            .as_array()
            .cloned()
            .unwrap();
        self.request("getblock", params)
    }
}
