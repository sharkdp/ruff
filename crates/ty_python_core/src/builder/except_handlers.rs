use crate::use_def::FlowSnapshot;

use super::SemanticIndexBuilder;

/// The nested `try` contexts for each active scope.
#[derive(Debug, Default)]
pub(super) struct TryNodeContextStackManager(Vec<Vec<TryNodeContext>>);

impl TryNodeContextStackManager {
    /// Start tracking `try` contexts for a nested scope.
    pub(super) fn enter_nested_scope(&mut self) {
        self.0.push(Vec::new());
    }

    /// Stop tracking `try` contexts for the current scope.
    pub(super) fn exit_scope(&mut self) {
        let popped_context = self.0.pop();
        debug_assert!(
            popped_context.is_some(),
            "exit_scope() should never be called on an empty stack \
(this indicates an unbalanced `enter_nested_scope()`/`exit_scope()` pair of calls)"
        );
    }

    /// Track a nested `try` block in the current scope.
    pub(super) fn push_context(&mut self) {
        self.current_try_context_stack()
            .push(TryNodeContext::default());
    }

    /// Stop tracking the innermost `try` block in the current scope.
    pub(super) fn pop_context(&mut self) -> TryNodeContext {
        self.current_try_context_stack()
            .pop()
            .expect("Cannot pop a `try` block off an empty `TryBlockContexts` stack")
    }

    /// Retrieve the [`TryNodeContext`] that is currently at the top of the stack, and take all
    /// snapshots recorded while visiting the `try` suite.
    pub(super) fn take_try_suite_snapshots(&mut self) -> Vec<FlowSnapshot> {
        std::mem::take(
            &mut self
                .current_try_context_stack()
                .last_mut()
                .expect("Cannot take snapshots from an empty `TryBlockContexts` stack")
                .try_suite_snapshots,
        )
    }

    /// Record the definition for every active `try` block in the current scope.
    pub(super) fn record_definition(&mut self, builder: &SemanticIndexBuilder) {
        for context in self.current_try_context_stack() {
            context.try_suite_snapshots.push(builder.flow_snapshot());
        }
    }

    /// Record terminal control flow entering the innermost `finally` suite.
    pub(super) fn record_terminal_finally_entry(&mut self, builder: &SemanticIndexBuilder) {
        if let Some(context) = self.current_try_context_stack().last_mut() {
            context
                .terminal_finally_entry_snapshots
                .push(builder.flow_snapshot());
        }
    }

    /// Retrieve the `try` contexts for the current scope.
    fn current_try_context_stack(&mut self) -> &mut Vec<TryNodeContext> {
        self.0
            .last_mut()
            .expect("There should always be at least one `TryBlockContexts` on the stack")
    }
}

/// Context for tracking definitions over the course of a single
/// [`ruff_python_ast::StmtTry`] node
///
/// It will likely be necessary to add more fields to this struct in the future
/// when we add more advanced handling of `finally` branches.
#[derive(Debug, Default)]
pub(super) struct TryNodeContext {
    try_suite_snapshots: Vec<FlowSnapshot>,
    terminal_finally_entry_snapshots: Vec<FlowSnapshot>,
}

impl TryNodeContext {
    pub(super) fn into_terminal_finally_entry_snapshots(self) -> Vec<FlowSnapshot> {
        self.terminal_finally_entry_snapshots
    }
}
