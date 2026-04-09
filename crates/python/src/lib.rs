use pyo3::prelude::*;
use pyo3::exceptions::PyRuntimeError;
use pyo3::types::PyDict;
use std::sync::{Arc, OnceLock};

use agent_browser_core::probe::{Probe, ProbeCheck, ProbeRequest};
use agent_browser_core::router::Engine;
use agent_browser_core::{FetchMode, FetchOptions, Fetcher};

static RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
static ENGINE: OnceLock<Engine> = OnceLock::new();
static PROBE: OnceLock<Probe> = OnceLock::new();

fn get_runtime() -> &'static tokio::runtime::Runtime {
    RUNTIME.get_or_init(|| {
        tokio::runtime::Runtime::new().expect("failed to create tokio runtime")
    })
}

fn get_engine() -> &'static Engine {
    ENGINE.get_or_init(|| {
        let engine = Engine::new();
        // Start T1 browser daemon
        let _ = get_runtime().block_on(engine.start_browser());
        engine
    })
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
fn fetch(
    py: Python<'_>,
    url: &str,
    output: &str,
    mode: &str,
    fast: bool,
) -> PyResult<Py<PyDict>> {
    let fetch_mode = match mode {
        "t1" => FetchMode::T1,
        "auto" => FetchMode::Auto,
        _ => FetchMode::T0,
    };

    let result = get_runtime()
        .block_on(async {
            let engine = get_engine();
            if fast {
                engine.fetch_fast(url, output, fetch_mode).await
            } else {
                engine.fetch(url, output, fetch_mode).await
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

    let results = get_runtime().block_on(async {
        let engine = get_engine();
        let futs: Vec<_> = urls
            .iter()
            .map(|url| engine.fetch(url, &output, fetch_mode))
            .collect();
        futures::future::join_all(futs).await
    });

    let mut py_results = Vec::with_capacity(results.len());
    for (url, result) in urls.iter().zip(results) {
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
                dict.set_item("url", url)?;
                dict.set_item("error", e.to_string())?;
            }
        }
        py_results.push(dict.unbind());
    }
    Ok(py_results)
}

// ---- Probe API (direct CDP, no HTTP) ----

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
    // Convert Python check dicts to Rust ProbeCheck
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

    let result = get_runtime()
        .block_on(get_probe().check(req))
        .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

    // Convert to Python dict
    let json_str =
        serde_json::to_string(&result).map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

    // Parse JSON string to Python dict
    let json_mod = py.import("json")?;
    let py_dict: Py<PyDict> = json_mod
        .call_method1("loads", (&json_str,))?
        .downcast_into::<PyDict>()
        .map_err(|e| PyRuntimeError::new_err(e.to_string()))?
        .unbind();
    Ok(py_dict)
}

#[pymodule]
fn agent_browser(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(fetch, m)?)?;
    m.add_function(wrap_pyfunction!(fetch_many, m)?)?;
    m.add_function(wrap_pyfunction!(probe, m)?)?;
    Ok(())
}
