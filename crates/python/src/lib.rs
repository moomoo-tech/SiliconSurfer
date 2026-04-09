use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::sync::{Arc, OnceLock};

use agent_browser_core::FetchMode;
use agent_browser_core::cdp::BrowserSession;
use agent_browser_core::distiller_fast::DistillMode;
use agent_browser_core::probe::{Probe, ProbeCheck, ProbeRequest};
use agent_browser_core::router::Engine;
use tokio::sync::Mutex as TokioMutex;

/// Dedicated Tokio runtime on its own thread — never conflicts with Python asyncio.
/// All Rust async operations run here, even if Python is inside an event loop.
static RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

fn get_runtime() -> &'static tokio::runtime::Runtime {
    RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .expect("failed to create tokio runtime")
    })
}

/// Run an async future on our dedicated runtime, safely from any Python context.
/// Works whether Python is in asyncio or not — uses spawn_blocking to avoid nesting.
fn run_async<F, T>(fut: F) -> T
where
    F: std::future::Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    let rt = get_runtime();
    // Check if we're already inside a tokio runtime (e.g. called from async Python)
    if tokio::runtime::Handle::try_current().is_ok() {
        // We're inside a runtime — can't block_on. Use a dedicated thread.
        std::thread::scope(|s| {
            s.spawn(|| rt.block_on(fut))
                .join()
                .expect("async task panicked")
        })
    } else {
        // Not inside a runtime — safe to block_on directly.
        rt.block_on(fut)
    }
}

static ENGINE: OnceLock<Engine> = OnceLock::new();
static PROBE: OnceLock<Probe> = OnceLock::new();

fn get_engine() -> &'static Engine {
    ENGINE.get_or_init(Engine::new)
}

fn ensure_browser_started() {
    static STARTED: OnceLock<bool> = OnceLock::new();
    STARTED.get_or_init(|| {
        let _ = run_async(get_engine().start_browser());
        true
    });
}

fn get_probe() -> &'static Probe {
    PROBE.get_or_init(|| {
        let engine = get_engine();
        Probe::new().with_browser(engine.browser_pool())
    })
}

// ---- Fetch API ----

#[pyfunction]
#[pyo3(signature = (url, output="markdown", mode="t0", fast=false))]
fn fetch(py: Python<'_>, url: &str, output: &str, mode: &str, fast: bool) -> PyResult<Py<PyDict>> {
    let fetch_mode = match mode {
        "t1" => FetchMode::T1,
        "auto" => FetchMode::Auto,
        _ => FetchMode::T0,
    };

    let url = url.to_string();
    let output = output.to_string();

    let result = run_async(async move {
        let engine = get_engine();
        if fast {
            engine.fetch_fast(&url, &output, fetch_mode).await
        } else {
            engine.fetch(&url, &output, fetch_mode).await
        }
    })
    .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

    let dict = PyDict::new(py);
    dict.set_item("url", &result.url)?;
    dict.set_item("title", &result.title)?;
    dict.set_item("content", &result.content)?;
    dict.set_item("content_length", result.content_length)?;
    dict.set_item("mode_used", &result.mode_used)?;
    Ok(dict.unbind())
}

#[pyfunction]
#[pyo3(signature = (urls, output="markdown", mode="t0"))]
fn fetch_many(
    py: Python<'_>,
    urls: Vec<String>,
    output: &str,
    mode: &str,
) -> PyResult<Vec<Py<PyDict>>> {
    let fetch_mode = match mode {
        "t1" => FetchMode::T1,
        "auto" => FetchMode::Auto,
        _ => FetchMode::T0,
    };
    let output = output.to_string();

    let results = run_async(async move {
        let engine = get_engine();
        let futs: Vec<_> = urls
            .iter()
            .map(|url| engine.fetch(url, &output, fetch_mode))
            .collect();
        futures::future::join_all(futs).await
    });

    let mut py_results = Vec::with_capacity(results.len());
    for (i, result) in results.into_iter().enumerate() {
        let dict = PyDict::new(py);
        match result {
            Ok(r) => {
                dict.set_item("url", &r.url)?;
                dict.set_item("title", &r.title)?;
                dict.set_item("content", &r.content)?;
                dict.set_item("content_length", r.content_length)?;
                dict.set_item("mode_used", &r.mode_used)?;
            }
            Err(e) => {
                dict.set_item("url", format!("url_{}", i))?;
                dict.set_item("error", e.to_string())?;
            }
        }
        py_results.push(dict.unbind());
    }
    Ok(py_results)
}

// ---- Probe API ----

#[pyfunction]
#[pyo3(signature = (url, checks=None, contains=None, render_js=false, snapshot=false))]
fn probe(
    py: Python<'_>,
    url: &str,
    checks: Option<Vec<Py<PyDict>>>,
    contains: Option<Vec<String>>,
    render_js: bool,
    snapshot: bool,
) -> PyResult<Py<PyDict>> {
    let rust_checks: Vec<ProbeCheck> = if let Some(checks) = checks {
        checks
            .iter()
            .map(|d| {
                let d = d.bind(py);
                ProbeCheck {
                    selector: d
                        .get_item("selector")
                        .ok()
                        .flatten()
                        .and_then(|v| v.extract::<String>().ok())
                        .unwrap_or_default(),
                    contains_text: d
                        .get_item("contains_text")
                        .ok()
                        .flatten()
                        .and_then(|v| v.extract::<String>().ok()),
                    attr: d
                        .get_item("attr")
                        .ok()
                        .flatten()
                        .and_then(|v| v.extract::<String>().ok()),
                    attr_value: d
                        .get_item("attr_value")
                        .ok()
                        .flatten()
                        .and_then(|v| v.extract::<String>().ok()),
                }
            })
            .collect()
    } else {
        vec![]
    };

    let req = ProbeRequest {
        url: url.to_string(),
        checks: rust_checks,
        contains: contains.unwrap_or_default(),
        timeout_secs: 10,
        snapshot,
        render_js,
    };

    let result = run_async(async move { get_probe().check(req).await })
        .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

    let json_str =
        serde_json::to_string(&result).map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

    let json_mod = py.import("json")?;
    let py_dict: Py<PyDict> = json_mod
        .call_method1("loads", (&json_str,))?
        .cast_into::<PyDict>()
        .map_err(|e| PyRuntimeError::new_err(e.to_string()))?
        .unbind();
    Ok(py_dict)
}

// ---- Session API (stateful browser interaction) ----

/// Wraps BrowserSession for Python — thread-safe via Arc<TokioMutex>.
#[pyclass]
struct Session {
    inner: Arc<TokioMutex<BrowserSession>>,
}

#[pymethods]
impl Session {
    #[new]
    fn new() -> PyResult<Self> {
        ensure_browser_started();
        let pool = get_engine().browser_pool();
        let session = run_async(async move { BrowserSession::new(pool).await })
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        Ok(Self {
            inner: Arc::new(TokioMutex::new(session)),
        })
    }

    /// Navigate to a URL. Returns {"success": bool, "url": str, "detail": str}.
    fn navigate(&self, py: Python<'_>, url: &str) -> PyResult<Py<PyDict>> {
        let inner = self.inner.clone();
        let url = url.to_string();
        let result = run_async(async move { inner.lock().await.navigate(&url).await })
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        action_result_to_dict(py, &result)
    }

    /// Get distilled view of current page. mode: reader/operator/spider/data/developer.
    fn see(&self, mode: &str) -> PyResult<String> {
        let inner = self.inner.clone();
        let distill_mode = parse_distill_mode(mode);
        run_async(async move { inner.lock().await.see(distill_mode).await })
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }

    /// Click element by CSS selector.
    fn click(&self, py: Python<'_>, selector: &str) -> PyResult<Py<PyDict>> {
        let inner = self.inner.clone();
        let selector = selector.to_string();
        let result = run_async(async move { inner.lock().await.click(&selector).await })
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        action_result_to_dict(py, &result)
    }

    /// Click element by @eN agent reference.
    fn click_agent_ref(&self, py: Python<'_>, ref_id: &str) -> PyResult<Py<PyDict>> {
        let inner = self.inner.clone();
        let ref_id = ref_id.to_string();
        let result = run_async(async move { inner.lock().await.click_agent_ref(&ref_id).await })
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        action_result_to_dict(py, &result)
    }

    /// Fill a form field by CSS selector.
    fn fill(&self, py: Python<'_>, selector: &str, value: &str) -> PyResult<Py<PyDict>> {
        let inner = self.inner.clone();
        let selector = selector.to_string();
        let value = value.to_string();
        let result = run_async(async move { inner.lock().await.fill(&selector, &value).await })
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        action_result_to_dict(py, &result)
    }

    /// Fill a form field by @eN agent reference.
    fn fill_agent_ref(&self, py: Python<'_>, ref_id: &str, value: &str) -> PyResult<Py<PyDict>> {
        let inner = self.inner.clone();
        let ref_id = ref_id.to_string();
        let value = value.to_string();
        let result =
            run_async(async move { inner.lock().await.fill_agent_ref(&ref_id, &value).await })
                .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        action_result_to_dict(py, &result)
    }

    /// Submit a form.
    fn submit(&self, py: Python<'_>, selector: &str) -> PyResult<Py<PyDict>> {
        let inner = self.inner.clone();
        let selector = selector.to_string();
        let result = run_async(async move { inner.lock().await.submit(&selector).await })
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        action_result_to_dict(py, &result)
    }

    /// Get raw HTML of current page.
    fn content(&self) -> PyResult<String> {
        let inner = self.inner.clone();
        run_async(async move { inner.lock().await.content().await })
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }

    /// Get current URL.
    fn url(&self) -> PyResult<String> {
        let inner = self.inner.clone();
        run_async(async move { inner.lock().await.url().await })
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }
}

fn parse_distill_mode(mode: &str) -> DistillMode {
    match mode {
        "operator" => DistillMode::Operator,
        "spider" => DistillMode::Spider,
        "data" => DistillMode::Data,
        "developer" => DistillMode::Developer,
        _ => DistillMode::Reader,
    }
}

fn action_result_to_dict(
    py: Python<'_>,
    result: &agent_browser_core::cdp::ActionResult,
) -> PyResult<Py<PyDict>> {
    let dict = PyDict::new(py);
    dict.set_item("success", result.success)?;
    dict.set_item("url", &result.url)?;
    dict.set_item("detail", &result.detail)?;
    Ok(dict.unbind())
}

#[pymodule]
fn agent_browser(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(fetch, m)?)?;
    m.add_function(wrap_pyfunction!(fetch_many, m)?)?;
    m.add_function(wrap_pyfunction!(probe, m)?)?;
    m.add_class::<Session>()?;
    Ok(())
}
