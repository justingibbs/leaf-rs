//! Card execution database operations (stubs for Phase A)

use leaf_core::{CardExecution, Result};
use uuid::Uuid;

use crate::Database;

/// Extension trait for card execution operations
pub trait CardExecutionQueries {
    fn create_card_execution(&self, execution: &CardExecution) -> Result<()>;
    fn get_card_execution(&self, id: Uuid) -> Result<Option<CardExecution>>;
    fn list_card_executions_for_stack_execution(&self, stack_execution_id: Uuid) -> Result<Vec<CardExecution>>;
    fn update_card_execution(&self, execution: &CardExecution) -> Result<()>;
}

impl CardExecutionQueries for Database {
    fn create_card_execution(&self, _execution: &CardExecution) -> Result<()> {
        // TODO: Implement in Phase C
        Ok(())
    }

    fn get_card_execution(&self, _id: Uuid) -> Result<Option<CardExecution>> {
        // TODO: Implement in Phase C
        Ok(None)
    }

    fn list_card_executions_for_stack_execution(&self, _stack_execution_id: Uuid) -> Result<Vec<CardExecution>> {
        // TODO: Implement in Phase C
        Ok(Vec::new())
    }

    fn update_card_execution(&self, _execution: &CardExecution) -> Result<()> {
        // TODO: Implement in Phase C
        Ok(())
    }
}
