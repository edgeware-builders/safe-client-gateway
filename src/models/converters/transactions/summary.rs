extern crate chrono;

use crate::models::backend::transactions::{CreationTransaction, Transaction};
use crate::models::backend::transactions::{
    EthereumTransaction, ModuleTransaction, MultisigTransaction,
};
use crate::models::converters::transactions::safe_app_info::safe_app_info_from;
use crate::models::service::transactions::summary::{ExecutionInfo, TransactionSummary};
use crate::models::service::transactions::{
    Creation, TransactionInfo, TransactionStatus, ID_PREFIX_CREATION_TX, ID_PREFIX_ETHEREUM_TX,
    ID_PREFIX_MODULE_TX, ID_PREFIX_MULTISIG_TX,
};
use crate::providers::info::InfoProvider;
use crate::utils::errors::ApiResult;
use crate::utils::hex_hash;

impl Transaction {
    pub fn to_transaction_summary(
        &self,
        info_provider: &mut dyn InfoProvider,
        safe: &str,
    ) -> ApiResult<Vec<TransactionSummary>> {
        match self {
            Transaction::Multisig(transaction) => {
                Ok(transaction.to_transaction_summary(info_provider)?)
            }
            Transaction::Ethereum(transaction) => {
                Ok(transaction.to_transaction_summary(info_provider, safe))
            }
            Transaction::Module(transaction) => {
                Ok(transaction.to_transaction_summary(info_provider))
            }
            Transaction::Unknown => bail!("Unknown transaction type from backend"),
        }
    }
}

impl MultisigTransaction {
    pub fn to_transaction_summary(
        &self,
        info_provider: &mut dyn InfoProvider,
    ) -> ApiResult<Vec<TransactionSummary>> {
        let safe_info = info_provider.safe_info(&self.safe.to_string())?;
        let tx_status = self.map_status(&safe_info);
        let missing_signers = if tx_status == TransactionStatus::AwaitingConfirmations {
            Some(self.missing_signers(&safe_info.owners))
        } else {
            None
        };
        Ok(vec![TransactionSummary {
            id: create_id!(ID_PREFIX_MULTISIG_TX, &self.safe, self.safe_tx_hash),
            timestamp: self
                .execution_date
                .unwrap_or(self.submission_date)
                .timestamp_millis(),
            tx_status,
            execution_info: Some(ExecutionInfo {
                nonce: self.nonce,
                confirmations_submitted: self.confirmation_count(),
                confirmations_required: self.confirmation_required(safe_info.threshold),
                missing_signers,
            }),
            tx_info: self.transaction_info(info_provider),
            safe_app_info: self
                .origin
                .as_ref()
                .and_then(|origin| safe_app_info_from(origin, info_provider)),
        }])
    }
}

impl EthereumTransaction {
    pub(super) fn to_transaction_summary(
        &self,
        info_provider: &mut dyn InfoProvider,
        safe: &str,
    ) -> Vec<TransactionSummary> {
        match &self.transfers {
            Some(transfers) => transfers
                .into_iter()
                .map(|transfer| TransactionSummary {
                    id: create_id!(
                        ID_PREFIX_ETHEREUM_TX,
                        safe,
                        self.tx_hash,
                        hex_hash(transfer)
                    ),
                    timestamp: self.execution_date.timestamp_millis(),
                    tx_status: TransactionStatus::Success,
                    execution_info: None,
                    safe_app_info: None,
                    tx_info: transfer.to_transfer(info_provider, safe),
                })
                .collect(),
            _ => vec![],
        }
    }
}

impl ModuleTransaction {
    pub(super) fn to_transaction_summary(
        &self,
        info_provider: &mut dyn InfoProvider,
    ) -> Vec<TransactionSummary> {
        vec![TransactionSummary {
            id: create_id!(
                ID_PREFIX_MODULE_TX,
                self.safe,
                self.transaction_hash,
                hex_hash(self)
            ),
            timestamp: self.execution_date.timestamp_millis(),
            tx_status: self.map_status(),
            execution_info: None,
            safe_app_info: None,
            tx_info: self.to_transaction_info(info_provider),
        }]
    }
}

impl CreationTransaction {
    pub fn to_transaction_summary(
        &self,
        safe_address: &String,
        info_provider: &mut dyn InfoProvider,
    ) -> TransactionSummary {
        TransactionSummary {
            id: create_id!(ID_PREFIX_CREATION_TX, safe_address),
            timestamp: self.created.timestamp_millis(),
            tx_status: TransactionStatus::Success,
            tx_info: TransactionInfo::Creation(Creation {
                creator: self.creator.clone(),
                creator_info: info_provider.contract_info(&self.creator).ok(),
                transaction_hash: self.transaction_hash.clone(),
                implementation: self.master_copy.clone(),
                implementation_info: self
                    .master_copy
                    .as_ref()
                    .map(|address| info_provider.contract_info(address).ok())
                    .flatten(),
                factory: self.factory_address.clone(),
                factory_info: self
                    .factory_address
                    .as_ref()
                    .map(|address| info_provider.contract_info(address).ok())
                    .flatten(),
            }),
            execution_info: None,
            safe_app_info: None,
        }
    }
}
