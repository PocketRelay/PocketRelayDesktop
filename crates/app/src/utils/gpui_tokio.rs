// Sourced from https://github.com/zed-industries/zed/blob/main/crates/gpui_tokio/src/gpui_tokio.rs
// slightly modified for this simplified use case

use std::future::Future;

use gpui_kit::{App, AppContext, AsyncApp, Global, ReadGlobal, Task};

pub use tokio::task::JoinError;

use crate::utils::defer::defer;

struct GlobalTokio {
    runtime: tokio::runtime::Runtime,
}

impl Global for GlobalTokio {}

impl GlobalTokio {
    fn new(runtime: tokio::runtime::Runtime) -> Self {
        Self { runtime }
    }
}

/// Initializes the Tokio wrapper using a new Tokio runtime with 2 worker threads.
pub fn init_tokio(cx: &mut App) {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        // Since we now have two executors, let's try to keep our footprint small
        .worker_threads(2)
        .enable_all()
        .build()
        .expect("Failed to initialize Tokio");

    cx.set_global(GlobalTokio::new(runtime));
}

pub struct Tokio;

impl Tokio {
    /// Spawns the given future on Tokio's thread pool, and returns it via a GPUI task
    /// Note that the Tokio task will be cancelled if the GPUI task is dropped
    #[allow(unused)]
    pub fn spawn<C, Fut, R>(cx: &C, f: Fut) -> Task<Result<R, JoinError>>
    where
        C: AppContext,
        Fut: Future<Output = R> + Send + 'static,
        R: Send + 'static,
    {
        cx.read_global(|tokio: &GlobalTokio, cx| {
            let join_handle = tokio.runtime.handle().spawn(f);
            let abort_handle = join_handle.abort_handle();
            let cancel = defer(move || {
                abort_handle.abort();
            });
            cx.background_spawn(async move {
                let result = join_handle.await;
                cancel.abort();
                result
            })
        })
    }

    /// Spawns the given future on Tokio's thread pool, and returns it via a GPUI task
    /// Note that the Tokio task will be cancelled if the GPUI task is dropped
    #[allow(unused)]
    pub fn spawn_with_callback<C, Fut, R, Callback>(cx: &C, f: Fut, callback: Callback)
    where
        C: AppContext,
        Fut: Future<Output = R> + Send + 'static,
        R: Send + 'static,
        Callback: FnOnce(&mut App, Result<R, JoinError>) + 'static,
    {
        cx.read_global(|tokio: &GlobalTokio, cx| {
            let join_handle = tokio.runtime.handle().spawn(f);
            let abort_handle = join_handle.abort_handle();
            let cancel = defer(move || {
                abort_handle.abort();
            });
            cx.spawn(move |app: &mut AsyncApp| {
                let app = app.clone();
                async move {
                    let result = join_handle.await;
                    cancel.abort();
                    app.update(move |cx| {
                        callback(cx, result);
                    });
                }
            })
            .detach();
        })
    }

    /// Blocks on the given future on Tokio's thread pool, and returns it via a GPUI task
    /// Note that the Tokio task will be cancelled if the GPUI task is dropped
    #[allow(unused)]
    pub fn block_on<C, Fut, R>(cx: &C, f: Fut) -> R
    where
        C: AppContext,
        Fut: Future<Output = R> + Send + 'static,
        R: Send + 'static,
    {
        cx.read_global(|tokio: &GlobalTokio, cx| tokio.runtime.handle().block_on(f))
    }

    /// Spawns the given future on Tokio's thread pool, and returns it via a GPUI task
    /// Note that the Tokio task will be cancelled if the GPUI task is dropped
    #[allow(unused)]
    pub fn spawn_result<C, Fut, R>(cx: &C, f: Fut) -> Task<anyhow::Result<R>>
    where
        C: AppContext,
        Fut: Future<Output = anyhow::Result<R>> + Send + 'static,
        R: Send + 'static,
    {
        cx.read_global(|tokio: &GlobalTokio, cx| {
            let join_handle = tokio.runtime.handle().spawn(f);
            let abort_handle = join_handle.abort_handle();
            let cancel = defer(move || {
                abort_handle.abort();
            });
            cx.background_spawn(async move {
                let result = join_handle.await?;
                drop(cancel);
                result
            })
        })
    }

    #[allow(unused)]
    pub fn handle(cx: &App) -> tokio::runtime::Handle {
        GlobalTokio::global(cx).runtime.handle().clone()
    }
}
