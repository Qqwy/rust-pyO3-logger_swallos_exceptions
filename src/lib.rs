use pyo3::prelude::*;
use std::{future::IntoFuture, sync::Arc, time::Duration};

/// In the production code, this is a complicated type that manages an open connection to a remote service.
#[pyclass]
#[derive(Debug)]
pub struct IncrementerClient {
    runtime: Arc<tokio::runtime::Runtime>,
}

#[pymethods]
impl IncrementerClient {
    #[new]
    pub fn new() -> PyResult<Self> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Failed to create Tokio runtime in opsqueue client");
    Ok(Self{runtime: Arc::new(runtime)})
    }

    pub fn sum(&self, py: Python<'_>, a: usize, mut b: usize, sleepy: bool) -> PyResult<usize> {
        py.allow_threads(|| {
            self.block_unless_interrupted(async {
                let mut result = a;
                while b != 0 {
                    // Simulate a difficult task that requires async:
                    result += 1;
                    b -= 1;
                    tokio::task::yield_now().await;

                    // Verbose logging after every step:
                    log::debug!("Incrementing {result} by 1...");

                    if result % 500 == 0 {
                        log::info!("Currently at result {result}...");
                        if sleepy {
                            tokio::time::sleep(Duration::from_millis(1000)).await;
                        }
                    }
                };
                log::info!("Final result: {result}");
                Ok(result)
            })
        })
    }
}

// What follows are internal helper functions
// that are not available directly from Python
impl IncrementerClient {
    pub fn block_unless_interrupted<T, E>(
        &self,
        future: impl IntoFuture<Output = Result<T, E>>,
    ) -> Result<T, E>
    where
        E: From<FatalPythonException>,
    {
        self.runtime.block_on(run_unless_interrupted(future))
    }

    pub fn sleep_unless_interrupted<E>(&self, duration: Duration) -> Result<(), E>
    where
        E: From<FatalPythonException>,
    {
        self.block_unless_interrupted(async {
            tokio::time::sleep(duration).await;
            Ok(())
        })
    }
}

pub async fn run_unless_interrupted<T, E>(
    future: impl IntoFuture<Output = Result<T, E>>,
) -> Result<T, E>
where
    E: From<FatalPythonException>,
{
    tokio::select! {
        res = future => res,
        py_err = check_signals_in_background() => Err(py_err)?,
    }
}

pub const SIGNAL_CHECK_INTERVAL: Duration = Duration::from_millis(100);

pub async fn check_signals_in_background() -> FatalPythonException {
    loop {
        tokio::time::sleep(SIGNAL_CHECK_INTERVAL).await;
        if let Err(err) = Python::with_gil(|py| py.check_signals()) {
            return err.into();
        }
    }
}

/// Indicates a 'fatal' PyErr: Any Python exception which is _not_ a subclass of `PyException`.
///
/// These are known as 'fatal' exceptions in Python.
/// c.f. https://docs.python.org/3/tutorial/errors.html#tut-userexceptions
///
/// We don't consume/wrap these errors but propagate them,
/// allowing things like KeyboardInterrupt, SystemExit or MemoryError,
/// to trigger cleanup-and-exit.
#[derive(thiserror::Error, Debug)]
#[error("Fatal Python exception: {0}")]
pub struct FatalPythonException(#[from] pub PyErr);

impl From<FatalPythonException> for PyErr {
    fn from(value: FatalPythonException) -> Self {
        value.0
    }
}

/// A Python module implemented in Rust.
#[pymodule]
fn incrementer(m: &Bound<'_, PyModule>) -> PyResult<()> {
    pyo3_log::init();

    m.add_class::<IncrementerClient>()?;
    Ok(())
}
