use thiserror::Error;

pub mod copilot;
pub mod gateway;
pub mod privacy;
pub mod tools;
pub mod wallet;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum AiError {
    #[error(
        "AI credit balance is exhausted. Please recharge your wallet to continue using Copilot."
    )]
    CreditsExhausted,
    #[error("Cloud AI Gateway is unreachable: {0}")]
    GatewayUnreachable(String),
    #[error("Cloud AI Gateway authentication failed: {0}")]
    GatewayAuthFailed(String),
    #[error("Rate limit exceeded. Please wait a moment and try again.")]
    RateLimited,
    #[error("Conversation not found: {0}")]
    ConversationNotFound(String),
    #[error("Tool execution failed: {0}")]
    ToolExecutionFailed(String),
    #[error("Database error: {0}")]
    Sql(String),
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    #[error("Validation error: {0}")]
    Validation(String),
}

impl AiError {
    pub fn code(&self) -> &'static str {
        match self {
            AiError::CreditsExhausted => "credits_exhausted",
            AiError::GatewayUnreachable(_) => "gateway_unreachable",
            AiError::GatewayAuthFailed(_) => "gateway_auth_failed",
            AiError::RateLimited => "rate_limited",
            AiError::ConversationNotFound(_) => "not_found",
            AiError::ToolExecutionFailed(_) => "tool_execution_failed",
            AiError::Sql(_) => "database_error",
            AiError::Unauthorized(_) => "unauthorized",
            AiError::Validation(_) => "validation_error",
        }
    }
}

impl From<rusqlite::Error> for AiError {
    fn from(err: rusqlite::Error) -> Self {
        AiError::Sql(err.to_string())
    }
}

pub use copilot::{
    ask_copilot, clear_conversation, get_audit_log, get_conversation, AiAuditLogItem,
    CopilotConversationDetails, CopilotMessageItem, CopilotResponse,
};
pub use gateway::{
    clear_mock_gateway, get_gateway_config, set_mock_gateway_response, test_gateway_connection,
    update_gateway_config, AiGatewayConfig, GatewayTestResult,
};
pub use privacy::{
    get_privacy_settings, update_privacy_settings, PrivacyRedactionEngine, PrivacySettings,
    ProtectionMetadata,
};
pub use tools::{
    execute_tool, get_available_tool_schemas, GetAttendanceStatsArgs, GetBatchSummaryArgs,
    GetExamResultsSummaryArgs, GetTodayOperationalSummaryArgs, QueryStudentsWithDuesArgs,
    SearchStudentsArgs, ToolDeclaration,
};
pub use wallet::{
    deduct_credits, get_credit_ledger, get_wallet_balance, record_credit_grant, sync_credits,
    AiWalletBalance, CreditTransactionItem,
};
