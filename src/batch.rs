use crate::actions::handle_action;
use crate::logger::ActionLogger;
use crate::models::{Action, StartupEntry};

pub struct BatchProcessor {
    logger: Option<ActionLogger>,
}

impl BatchProcessor {
    pub fn new(logger: Option<ActionLogger>) -> Self {
        Self { logger }
    }

    pub fn process_batch(
        &self,
        entries: &[StartupEntry],
        action: Action,
    ) -> BatchResult {
        let mut success_count = 0;
        let mut failed_count = 0;
        let mut errors = Vec::new();

        for entry in entries {
            match handle_action(entry, action) {
                Ok(_) => {
                    success_count += 1;
                    if let Some(ref logger) = self.logger {
                        let _ = logger.log_action(
                            &action.to_string(),
                            &entry.name,
                            true,
                            None,
                        );
                    }
                }
                Err(e) => {
                    failed_count += 1;
                    let error_msg = format!("{}: {}", entry.name, e);
                    errors.push(error_msg.clone());
                    if let Some(ref logger) = self.logger {
                        let _ = logger.log_action(
                            &action.to_string(),
                            &entry.name,
                            false,
                            Some(&e.to_string()),
                        );
                    }
                }
            }
        }

        if let Some(ref logger) = self.logger {
            let _ = logger.log_batch_action(
                &action.to_string(),
                entries.len(),
                success_count,
            );
        }

        BatchResult {
            total: entries.len(),
            success: success_count,
            failed: failed_count,
            errors,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BatchResult {
    pub total: usize,
    pub success: usize,
    pub failed: usize,
    pub errors: Vec<String>,
}

impl BatchResult {
    pub fn success_rate(&self) -> f64 {
        if self.total == 0 {
            return 0.0;
        }
        (self.success as f64 / self.total as f64) * 100.0
    }

    pub fn summary(&self) -> String {
        format!(
            "Batch operation completed: {} successful, {} failed out of {} total ({:.1}% success rate)",
            self.success,
            self.failed,
            self.total,
            self.success_rate()
        )
    }
}

