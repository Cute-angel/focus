use futures::future::{join_all, BoxFuture};
use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;
use crate::api::extension::Results;
use crate::api::types::PluginResult;

#[derive(Debug, Clone)]
pub struct ShortcutContext {
    pub raw_input: String,
    pub trimmed_input: String,
    pub rest_after_prefix: String,
}



impl ShortcutContext {
    pub fn new(input: &str) -> Self {
        let trimmed_input = input.trim().to_string();
        let rest_after_prefix = trimmed_input.chars().skip(1).collect::<String>();

        Self {
            raw_input: input.to_string(),
            trimmed_input,
            rest_after_prefix,
        }
    }

    pub fn prefix(&self) -> Option<char> {
        self.trimmed_input.chars().next()
    }
}

#[derive(Debug, Clone)]
pub struct ScoredItem<T> {
    pub score: u64,
    pub value: T,
}

impl<T> ScoredItem<T>{
    pub fn new(score: u64, value: T) -> Self {
        ScoredItem { score, value }
    }
}

impl<T:Default> Default for ScoredItem<T> {
    fn default() -> Self {
        ScoredItem {
            score:0,
            value: T::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum DispatchStage {
    Exact,
    Any,
    Fixed,
    AnyWithFixed,
}

#[derive(Debug, Clone)]
pub struct ShortcutResult<T> {
    pub stage: DispatchStage,
    pub items: Vec<ScoredItem<T>>,
}

impl<T> From<ShortcutResult<T>> for Vec<T>{
    fn from(shortcut: ShortcutResult<T>) -> Vec<T> {
        shortcut.items.into_iter().map(|it| it.value).collect()
    }
}

impl From<ShortcutResult<PluginResult>> for PluginResult {
    fn from(shortcut: ShortcutResult<PluginResult>) -> PluginResult {
        let mut total = 0;
        let mut items = Vec::new();
        for i in shortcut.items {
            if i.score > 0u64 {
                match i.value {
                    PluginResult::ExtensionResult(i) => {
                        items.push(i);
                        total += 1;
                    }
                    PluginResult::Results(mut i) => {
                        items.append(&mut i.items);
                        total+= i.total_count;
                    }
                    PluginResult::PluginError(_) => {}
                    PluginResult::Null => {}
                };
            }
        }
        Results {
            total_count: total,
            items,
        }.into()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ShortcutError {
    #[error("duplicated exact key: {0}")]
    DuplicateExactKey(char),
}

type ExactHandler<C, T> =
    Arc<dyn Fn(ShortcutContext, C) -> BoxFuture<'static, Vec<T>> + Send + Sync>;
type AnyHandler<C, T> =
    Arc<dyn Fn(ShortcutContext, C) -> BoxFuture<'static, Vec<ScoredItem<T>>> + Send + Sync>;
type FixedHandler<C, T> =
    Arc<dyn Fn(ShortcutContext, C) -> BoxFuture<'static, Vec<T>> + Send + Sync>;

pub enum ShortcutRegistration<C, T>
where
    C: Clone + Send + Sync + 'static,
    T: Send + 'static,
{
    Exact {
        key: char,
        handler: ExactHandler<C, T>,
    },
    Any {
        plugin_priority: i32,
        handler: AnyHandler<C, T>,
    },
    Fixed {
        score: u64,
        plugin_priority: i32,
        handler: FixedHandler<C, T>,
    },
}

impl<C, T> ShortcutRegistration<C, T>
where
    C: Clone + Send + Sync + 'static,
    T: Send + 'static,
{
    pub fn exact<F, Fut>(key: impl Into<char>, handler: F) -> Self
    where
        F: Fn(ShortcutContext, C) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Vec<T>> + Send + 'static,
    {
        let wrapped: ExactHandler<C, T> =
            Arc::new(move |ctx, runtime| Box::pin(handler(ctx, runtime)));
        Self::Exact {
            key: key.into(),
            handler: wrapped,
        }
    }

    pub fn any<F, Fut>(plugin_priority: i32, handler: F) -> Self
    where
        F: Fn(ShortcutContext, C) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Vec<ScoredItem<T>>> + Send + 'static,
    {
        let wrapped: AnyHandler<C, T> =
            Arc::new(move |ctx, runtime| Box::pin(handler(ctx, runtime)));
        Self::Any {
            plugin_priority,
            handler: wrapped,
        }
    }

    pub fn fixed<F, Fut>(score: u64, plugin_priority: i32, handler: F) -> Self
    where
        F: Fn(ShortcutContext, C) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Vec<T>> + Send + 'static,
    {
        let wrapped: FixedHandler<C, T> =
            Arc::new(move |ctx, runtime| Box::pin(handler(ctx, runtime)));
        Self::Fixed {
            score,
            plugin_priority,
            handler: wrapped,
        }
    }
}

struct ExactEntry<C, T> {
    handler: ExactHandler<C, T>,
}

struct AnyEntry<C, T> {
    handler: AnyHandler<C, T>,
    plugin_priority: i32,
    register_order: usize,
}

struct FixedEntry<C, T> {
    handler: FixedHandler<C, T>,
    score: u64,
    plugin_priority: i32,
    register_order: usize,
}

struct RankedItem<T> {
    score: u64,
    plugin_priority: i32,
    register_order: usize,
    item_order: usize,
    value: T,
}

pub struct ShortcutsDispatcher<C, T>
where
    C: Clone + Send + Sync + 'static,
    T: Send + 'static,
{
    exact_handlers: HashMap<char, ExactEntry<C, T>>,
    any_handlers: Vec<AnyEntry<C, T>>,
    fixed_handlers: Vec<FixedEntry<C, T>>,
    next_register_order: usize,
}

impl<C, T> Default for ShortcutsDispatcher<C, T>
where
    C: Clone + Send + Sync + 'static,
    T: Send + 'static,
{
    fn default() -> Self {
        Self {
            exact_handlers: HashMap::new(),
            any_handlers: Vec::new(),
            fixed_handlers: Vec::new(),
            next_register_order: 0,
        }
    }
}

impl<C, T> ShortcutsDispatcher<C, T>
where
    C: Clone + Send + Sync + 'static,
    T: Send + 'static,
{
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a [`ShortcutRegistration`] into dispatcher.
    ///
    /// Returns [`ShortcutError::DuplicateExactKey`] when registering
    /// an `Exact` shortcut with an existing key.
    ///
    /// # Examples
    /// ```ignore
    /// use crate::core::shortcut::{ShortcutRegistration, ShortcutsDispatcher};
    ///
    /// let mut dispatcher = ShortcutsDispatcher::<(), String>::new();
    /// dispatcher
    ///     .register(ShortcutRegistration::exact("app", |_ctx, _| async move {
    ///         vec!["ok".to_string()]
    ///     }))
    ///     .unwrap();
    /// ```
    pub fn register(
        &mut self,
        registration: ShortcutRegistration<C, T>,
    ) -> Result<(), ShortcutError> {
        match registration {
            ShortcutRegistration::Exact { key, handler } => {
                if self.exact_handlers.contains_key(&key) {
                    return Err(ShortcutError::DuplicateExactKey(key));
                }
                self.exact_handlers.insert(key, ExactEntry { handler });
                Ok(())
            }
            ShortcutRegistration::Any {
                plugin_priority,
                handler,
            } => {
                let register_order = self.take_register_order();
                self.any_handlers.push(AnyEntry {
                    handler,
                    plugin_priority,
                    register_order,
                });
                Ok(())
            }
            ShortcutRegistration::Fixed {
                score,
                plugin_priority,
                handler,
            } => {
                let register_order = self.take_register_order();
                self.fixed_handlers.push(FixedEntry {
                    handler,
                    score,
                    plugin_priority,
                    register_order,
                });
                Ok(())
            }
        }
    }
    /// Register an exact-prefix handler.
    ///
    /// This is a shorthand for:
    /// `self.register(ShortcutRegistration::exact(key, handler))`.
    ///
    /// # Examples
    /// ```ignore
    /// use crate::core::shortcut::ShortcutsDispatcher;
    ///
    /// let mut dispatcher = ShortcutsDispatcher::<(), String>::new();
    /// dispatcher
    ///     .register_exact("=", |ctx, _| async move {
    ///         vec![format!("exact: {}", ctx.rest_after_prefix)]
    ///     })
    ///     .unwrap();
    /// ```
    pub fn register_exact<F, Fut>(
        &mut self,
        key: impl Into<char>,
        handler: F,
    ) -> Result<(), ShortcutError>
    where
        F: Fn(ShortcutContext, C) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Vec<T>> + Send + 'static,
    {
        self.register(ShortcutRegistration::exact(key, handler))
    }

    /// Register a dynamic-score `Any` handler.
    ///
    /// This is a shorthand for:
    /// `self.register(ShortcutRegistration::any(plugin_priority, handler))`.
    ///
    /// # Examples
    /// ```ignore
    /// use crate::core::shortcut::{ScoredItem, ShortcutsDispatcher};
    ///
    /// let mut dispatcher = ShortcutsDispatcher::<(), String>::new();
    /// dispatcher.register_any(10, |ctx, _| async move {
    ///     vec![ScoredItem {
    ///         score: 90,
    ///         value: format!("any: {}", ctx.trimmed_input),
    ///     }]
    /// });
    /// ```
    pub fn register_any<F, Fut>(&mut self, plugin_priority: i32, handler: F)
    where
        F: Fn(ShortcutContext, C) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Vec<ScoredItem<T>>> + Send + 'static,
    {
        self.register(ShortcutRegistration::any(plugin_priority, handler))
            .expect("register_any should not fail");
    }

    /// Register a fixed-score fallback handler.
    ///
    /// This is a shorthand for:
    /// `self.register(ShortcutRegistration::fixed(score, plugin_priority, handler))`.
    ///
    /// # Examples
    /// ```ignore
    /// use crate::core::shortcut::ShortcutsDispatcher;
    ///
    /// let mut dispatcher = ShortcutsDispatcher::<(), String>::new();
    /// dispatcher.register_fixed(20, 0, |ctx, _| async move {
    ///     if ctx.trimmed_input.is_empty() {
    ///         vec!["fallback".to_string()]
    ///     } else {
    ///         Vec::new()
    ///     }
    /// });
    /// ```
    pub fn register_fixed<F, Fut>(&mut self, score: u64, plugin_priority: i32, handler: F)
    where
        F: Fn(ShortcutContext, C) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Vec<T>> + Send + 'static,
    {
        self.register(ShortcutRegistration::fixed(score, plugin_priority, handler))
            .expect("register_fixed should not fail");
    }

    /// Run order: Exact -> Any(all handlers) -> Fixed(fallback/fill).
    ///
    /// - Exact: first token hit, short-circuit and return.
    /// - Any: execute all handlers, then sort by score desc.
    /// - Fixed: run only when Any has no result, or Any count is lower than `top_k`.
    pub async fn run(&self, input: &str, runtime: C, top_k: Option<usize>) -> ShortcutResult<T> {
        let ctx = ShortcutContext::new(input);

        if let Some(prefix) = ctx.prefix() {
            if let Some(entry) = self.exact_handlers.get(&prefix) {
                let exact_values = (entry.handler)(ctx.clone(), runtime).await;
                let mut items = exact_values
                    .into_iter()
                    .enumerate()
                    .map(|(idx, value)| ScoredItem {
                        score: u64::MAX - (idx as u64),
                        value,
                    })
                    .collect::<Vec<_>>();
                Self::truncate_if_needed(&mut items, top_k);
                return ShortcutResult {
                    stage: DispatchStage::Exact,
                    items,
                };
            }
        }

        let mut any_ranked = self.collect_any_ranked(ctx.clone(), runtime.clone()).await;
        any_ranked.sort_by(Self::ranked_cmp);
        let mut any_items = any_ranked
            .into_iter()
            .map(|item| ScoredItem {
                score: item.score,
                value: item.value,
            })
            .collect::<Vec<_>>();

        let mut need_fixed = any_items.is_empty();
        if let Some(limit) = top_k {
            if any_items.len() < limit {
                need_fixed = true;
            }
        }
        if self.fixed_handlers.is_empty() {
            need_fixed = false;
        }

        if !need_fixed {
            Self::truncate_if_needed(&mut any_items, top_k);
            return ShortcutResult {
                stage: DispatchStage::Any,
                items: any_items,
            };
        }

        let fixed_limit = top_k.map(|limit| limit.saturating_sub(any_items.len()));
        let mut fixed_ranked = self.collect_fixed_ranked(ctx, runtime).await;
        fixed_ranked.sort_by(Self::ranked_cmp);
        let mut fixed_items = fixed_ranked
            .into_iter()
            .map(|item| ScoredItem {
                score: item.score,
                value: item.value,
            })
            .collect::<Vec<_>>();
        Self::truncate_if_needed(&mut fixed_items, fixed_limit);

        let appended_fixed = !fixed_items.is_empty();
        let stage = if any_items.is_empty() {
            DispatchStage::Fixed
        } else if appended_fixed {
            DispatchStage::AnyWithFixed
        } else {
            DispatchStage::Any
        };
        any_items.extend(fixed_items);
        Self::truncate_if_needed(&mut any_items, top_k);

        ShortcutResult {
            stage,
            items: any_items,
        }
    }

    fn take_register_order(&mut self) -> usize {
        let order = self.next_register_order;
        self.next_register_order += 1;
        order
    }

    fn truncate_if_needed<U>(items: &mut Vec<U>, limit: Option<usize>) {
        if let Some(limit) = limit {
            if items.len() > limit {
                items.truncate(limit);
            }
        }
    }

    fn ranked_cmp(left: &RankedItem<T>, right: &RankedItem<T>) -> std::cmp::Ordering {
        right
            .score
            .cmp(&left.score)
            .then_with(|| right.plugin_priority.cmp(&left.plugin_priority))
            .then_with(|| left.register_order.cmp(&right.register_order))
            .then_with(|| left.item_order.cmp(&right.item_order))
    }

    async fn collect_any_ranked(&self, ctx: ShortcutContext, runtime: C) -> Vec<RankedItem<T>> {
        let futs = self.any_handlers.iter().map(|entry| {
            let ctx = ctx.clone();
            let runtime = runtime.clone();
            async move {
                let values = (entry.handler)(ctx, runtime).await;
                values
                    .into_iter()
                    .enumerate()
                    .map(|(item_order, item)| RankedItem {
                        score: item.score,
                        plugin_priority: entry.plugin_priority,
                        register_order: entry.register_order,
                        item_order,
                        value: item.value,
                    })
                    .collect::<Vec<_>>()
            }
        });

        join_all(futs)
            .await
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
    }

    async fn collect_fixed_ranked(&self, ctx: ShortcutContext, runtime: C) -> Vec<RankedItem<T>> {
        let futs = self.fixed_handlers.iter().map(|entry| {
            let ctx = ctx.clone();
            let runtime = runtime.clone();
            async move {
                let values = (entry.handler)(ctx, runtime).await;
                values
                    .into_iter()
                    .enumerate()
                    .map(|(item_order, value)| RankedItem {
                        score: entry.score,
                        plugin_priority: entry.plugin_priority,
                        register_order: entry.register_order,
                        item_order,
                        value,
                    })
                    .collect::<Vec<_>>()
            }
        });

        join_all(futs)
            .await
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::executor::block_on;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn exact_short_circuit_any_and_fixed() {
        let any_called = Arc::new(AtomicUsize::new(0));
        let fixed_called = Arc::new(AtomicUsize::new(0));

        let mut dispatcher = ShortcutsDispatcher::<(), String>::new();
        dispatcher
            .register_exact('>', |_ctx, _| async move { vec!["exact".to_string()] })
            .unwrap();

        let any_called_clone = any_called.clone();
        dispatcher.register_any(0, move |_ctx, _| {
            let any_called_clone = any_called_clone.clone();
            async move {
                any_called_clone.fetch_add(1, Ordering::SeqCst);
                vec![ScoredItem {
                    score: 10,
                    value: "any".to_string(),
                }]
            }
        });

        let fixed_called_clone = fixed_called.clone();
        dispatcher.register_fixed(1, 0, move |_ctx, _| {
            let fixed_called_clone = fixed_called_clone.clone();
            async move {
                fixed_called_clone.fetch_add(1, Ordering::SeqCst);
                vec!["fixed".to_string()]
            }
        });

        let result = block_on(dispatcher.run(">query", (), Some(10)));
        assert_eq!(result.stage, DispatchStage::Exact);
        assert_eq!(result.items.len(), 1);
        assert_eq!(result.items[0].value, "exact");
        assert_eq!(any_called.load(Ordering::SeqCst), 0);
        assert_eq!(fixed_called.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn any_runs_all_handlers_and_sorts() {
        let called = Arc::new(AtomicUsize::new(0));
        let mut dispatcher = ShortcutsDispatcher::<(), String>::new();

        let called_1 = called.clone();
        dispatcher.register_any(10, move |_ctx, _| {
            let called_1 = called_1.clone();
            async move {
                called_1.fetch_add(1, Ordering::SeqCst);
                vec![ScoredItem {
                    score: 20,
                    value: "mid".to_string(),
                }]
            }
        });

        let called_2 = called.clone();
        dispatcher.register_any(10, move |_ctx, _| {
            let called_2 = called_2.clone();
            async move {
                called_2.fetch_add(1, Ordering::SeqCst);
                vec![ScoredItem {
                    score: 100,
                    value: "top".to_string(),
                }]
            }
        });

        let result = block_on(dispatcher.run("not_exact", (), Some(10)));
        assert_eq!(result.stage, DispatchStage::Any);
        assert_eq!(result.items.len(), 2);
        assert_eq!(result.items[0].value, "top");
        assert_eq!(result.items[1].value, "mid");
        assert_eq!(called.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn fixed_fallback_when_any_is_empty() {
        let mut dispatcher = ShortcutsDispatcher::<(), String>::new();
        dispatcher.register_any(0, |_ctx, _| async move { Vec::<ScoredItem<String>>::new() });
        dispatcher.register_fixed(1, 0, |_ctx, _| async move { vec!["fixed".to_string()] });

        let result = block_on(dispatcher.run("unknown", (), Some(10)));
        assert_eq!(result.stage, DispatchStage::Fixed);
        assert_eq!(result.items.len(), 1);
        assert_eq!(result.items[0].value, "fixed");
    }

    #[test]
    fn fixed_fill_when_any_not_enough_for_limit() {
        let mut dispatcher = ShortcutsDispatcher::<(), String>::new();
        dispatcher.register_any(0, |_ctx, _| async move {
            vec![ScoredItem {
                score: 100,
                value: "any".to_string(),
            }]
        });
        dispatcher.register_fixed(1, 0, |_ctx, _| async move {
            vec!["fixed1".to_string(), "fixed2".to_string()]
        });

        let result = block_on(dispatcher.run("unknown", (), Some(2)));
        assert_eq!(result.stage, DispatchStage::AnyWithFixed);
        assert_eq!(result.items.len(), 2);
        assert_eq!(result.items[0].value, "any");
        assert_eq!(result.items[1].value, "fixed1");
    }
}
