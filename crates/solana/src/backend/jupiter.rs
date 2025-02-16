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
            config: self.transaction_config(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::{NATIVE_MINT, USDC_MINT};
    use solana_sdk::signature::Keypair;
    use x_link_types::account::Account;

    // Helper function to create test account
    fn create_test_account() -> Account {
        let keypair = Keypair::new();
        Account::new(123456, keypair)
    }

    #[tokio::test]
    async fn test_quote() {
        let backend = Backend::new();
        // Test quoting 1 SOL to USDC
        let amount = 1_000_000_000; // 1 SOL in lamports
        let result = backend.quote(NATIVE_MINT, USDC_MINT, amount).await;

        assert!(result.is_ok(), "Quote should succeed");
        let quote = result.unwrap();

        // Basic validation of quote response
        assert_eq!(quote.input_mint, NATIVE_MINT);
        assert_eq!(quote.output_mint, USDC_MINT);
        assert_eq!(quote.in_amount, amount);
        assert!(quote.out_amount > 0);
    }

    #[tokio::test]
    async fn test_instructions() {
        let backend = Backend::new();
        let account = create_test_account();

        // Test getting swap instructions for 1 SOL to USDC
        let amount = 1_000_000_000; // 1 SOL in lamports
        let result = backend
            .instructions(&account, &NATIVE_MINT, &USDC_MINT, amount)
            .await;

        assert!(result.is_ok(), "Getting instructions should succeed");
        let instructions = result.unwrap();

        // Validate instruction response
        assert!(!instructions.compute_budget_instructions.is_empty());
        assert!(instructions.swap_instruction.program_id != Pubkey::default());
    }

    #[tokio::test]
    async fn test_swap_transaction() {
        let backend = Backend::new();
        let account = create_test_account();

        // Test creating swap transaction for 1 SOL to USDC
        let amount = 1_000_000_000; // 1 SOL in lamports
        let recent_blockhash = Hash::default(); // In real usage, this would come from the cluster

        let result = backend
            .swap_transaction(&account, &NATIVE_MINT, &USDC_MINT, amount, recent_blockhash)
            .await;

        assert!(result.is_ok(), "Creating swap transaction should succeed");
        let transaction = result.unwrap();

        // Validate transaction
        assert!(!transaction.message.instructions.is_empty());
        assert_eq!(transaction.message.header.num_required_signatures, 1);
    }

    #[test]
    fn test_transaction_config() {
        let backend = Backend::new();
        let config = backend.transaction_config();

        // Test that transaction config has default values
        assert_eq!(config, TransactionConfig::default());
    }
}
