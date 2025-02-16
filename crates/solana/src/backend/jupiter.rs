use crate::{
    constants::{DEFAULT_SLIPPAGE_BPS, JUP_BASE_PATH},
    error::Error,
};
use jupiter_swap_api_client::{
    quote::{QuoteRequest, QuoteResponse},
    swap::{SwapInstructionsResponse, SwapRequest},
    transaction_config::TransactionConfig,
    JupiterSwapApiClient,
};
use solana_sdk::{hash::Hash, pubkey::Pubkey, signer::Signer, transaction::Transaction};
use x_link_types::account::Account;

pub struct Backend {
    client: JupiterSwapApiClient,
}

impl Default for Backend {
    fn default() -> Self {
        Self::new()
    }
}

// NOTE: So many fucking options in QuoteRequest and TransactionConfig
// Gotta explore those
impl Backend {
    pub fn new() -> Self {
        let client = JupiterSwapApiClient::new(JUP_BASE_PATH.to_string());
        Self { client }
    }

    fn transaction_config(&self) -> TransactionConfig {
        TransactionConfig {
            ..Default::default()
        }
    }

    pub(crate) async fn quote(
        &self,
        input_mint: Pubkey,
        output_mint: Pubkey,
        amount: u64,
    ) -> Result<QuoteResponse, Error> {
        let request = QuoteRequest {
            input_mint,
            output_mint,
            amount,
            slippage_bps: DEFAULT_SLIPPAGE_BPS,
            ..Default::default()
        };
        self.client.quote(&request).await.map_err(Error::from)
    }

    pub(crate) async fn instructions(
        &self,
        account: &Account,
        input_mint: &Pubkey,
        output_mint: &Pubkey,
        amount: u64,
    ) -> Result<SwapInstructionsResponse, Error> {
        let quote = self.quote(*input_mint, *output_mint, amount).await?;
        let request = SwapRequest {
            user_public_key: account.pubkey(),
            quote_response: quote,
            config: TransactionConfig::default(),
        };
        self.client
            .swap_instructions(&request)
            .await
            .map_err(Error::from)
    }

    pub(crate) async fn swap_transaction(
        &self,
        account: &Account,
        input_mint: &Pubkey,
        output_mint: &Pubkey,
        amount: u64,
        recent_blockhash: Hash,
    ) -> Result<Transaction, Error> {
        let instructions = self
            .instructions(account, input_mint, output_mint, amount)
            .await?;
        let mut ixs = vec![];
        ixs.extend(instructions.compute_budget_instructions);
        ixs.extend(instructions.setup_instructions);
        ixs.push(instructions.swap_instruction);
        if let Some(ix) = instructions.cleanup_instruction {
            ixs.push(ix)
        }
        let tx = Transaction::new_signed_with_payer(
            &ixs,
            Some(&account.pubkey()),
            &[account],
            recent_blockhash,
        );
        Ok(tx)
    }
}
