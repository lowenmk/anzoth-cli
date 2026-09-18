use codex_core::CodexThread;
use codex_core::NewThread;
use codex_core::StartThreadOptions;
use codex_core::ThreadManager;
use codex_core::config::Config;
use codex_protocol::ThreadId;
use codex_protocol::error::CodexErr;
use codex_protocol::error::Result as CodexResult;
use codex_protocol::protocol::InitialHistory;
use codex_protocol::protocol::Op;
use codex_protocol::protocol::W3cTraceContext;
use codex_protocol::user_input::UserInput;
use serde_json::Value;
use std::sync::Arc;
use std::sync::Weak;

/// A fully resolved agent invocation.
///
/// Agent discovery owns rendering `prompt`, including any selected skill
/// references. The runtime only starts that prompt in isolated forked context.
pub struct AgentInvocation {
    pub config: Config,
    pub prompt: String,
    pub parent_trace: Option<W3cTraceContext>,
    pub output_schema: Option<Value>,
}

/// A spawned agent whose initial turn has been submitted.
pub struct AgentRun {
    pub thread_id: ThreadId,
    pub turn_id: String,
    pub thread: Arc<CodexThread>,
}

/// Runs resolved agents in threads forked by the owning [`ThreadManager`].
#[derive(Clone)]
pub struct AgentRunner {
    thread_manager: Weak<ThreadManager>,
}

impl AgentRunner {
    pub fn new(thread_manager: Weak<ThreadManager>) -> Self {
        Self { thread_manager }
    }

    /// Starts a resolved agent in a fork of `parent_thread_id`.
    pub async fn start(
        &self,
        parent_thread_id: ThreadId,
        invocation: AgentInvocation,
    ) -> CodexResult<AgentRun> {
        self.start_inner(Some(parent_thread_id), invocation).await
    }

    /// Starts an auxiliary agent with no parent history or fork relationship.
    pub async fn start_isolated(&self, invocation: AgentInvocation) -> CodexResult<AgentRun> {
        self.start_inner(None, invocation).await
    }

    async fn start_inner(
        &self,
        parent_thread_id: Option<ThreadId>,
        invocation: AgentInvocation,
    ) -> CodexResult<AgentRun> {
        let AgentInvocation {
            config,
            prompt,
            parent_trace,
            output_schema,
        } = invocation;
        if prompt.trim().is_empty() {
            return Err(CodexErr::InvalidRequest(
                "agent prompt must not be empty".to_string(),
            ));
        }

        let thread_manager = self
            .thread_manager
            .upgrade()
            .ok_or_else(|| CodexErr::UnsupportedOperation("thread manager dropped".to_string()))?;
        let environments =
            thread_manager.default_environment_selections(&config.cwd, &config.workspace_roots);
        let new_thread = if let Some(parent_thread_id) = parent_thread_id {
            thread_manager
                .spawn_subagent(
                    parent_thread_id,
                    StartThreadOptions {
                        config,
                        allow_provider_model_fallback: false,
                        initial_history: InitialHistory::New,
                        history_mode: None,
                        session_source: None,
                        thread_source: None,
                        dynamic_tools: Vec::new(),
                        metrics_service_name: None,
                        parent_trace: parent_trace.clone(),
                        environments,
                        thread_extension_init: Default::default(),
                        supports_openai_form_elicitation: false,
                    },
                )
                .await?
        } else {
            thread_manager
                .start_thread_with_options(StartThreadOptions {
                    config,
                    allow_provider_model_fallback: false,
                    initial_history: InitialHistory::New,
                    history_mode: None,
                    session_source: None,
                    thread_source: None,
                    dynamic_tools: Vec::new(),
                    metrics_service_name: None,
                    parent_trace: parent_trace.clone(),
                    environments,
                    thread_extension_init: Default::default(),
                    supports_openai_form_elicitation: false,
                })
                .await?
        };
        let NewThread {
            thread_id, thread, ..
        } = new_thread;
        let turn_id = match thread
            .submit_with_trace(
                Op::UserInput {
                    items: vec![UserInput::Text {
                        text: prompt,
                        text_elements: Vec::new(),
                    }],
                    final_output_json_schema: output_schema,
                    responsesapi_client_metadata: None,
                    additional_context: Default::default(),
                    thread_settings: Default::default(),
                },
                parent_trace,
            )
            .await
        {
            Ok(turn_id) => turn_id,
            Err(error) => {
                let _ = thread.shutdown_and_wait().await;
                let _ = thread_manager.remove_thread(&thread_id).await;
                return Err(error);
            }
        };

        Ok(AgentRun {
            thread_id,
            turn_id,
            thread,
        })
    }

    pub async fn cleanup(&self, run: AgentRun) {
        let _ = run.thread.shutdown_and_wait().await;
        if let Some(thread_manager) = self.thread_manager.upgrade() {
            let _ = thread_manager.remove_thread(&run.thread_id).await;
        }
    }
}
