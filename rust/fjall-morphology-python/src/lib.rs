use fjall_morphology_core::FjallMorphology;
use pyo3::{exceptions::PyRuntimeError, prelude::*};

#[pyclass]
struct PyFjallMorphology {
    inner: FjallMorphology,
}

#[pymethods]
impl PyFjallMorphology {
    #[new]
    fn new(folder: &str) -> PyResult<Self> {
        let inner =
            FjallMorphology::new(folder).map_err(|err| PyRuntimeError::new_err(err.to_string()))?;
        Ok(Self { inner })
    }

    fn build_from_path(&mut self, path: &str) -> PyResult<()> {
        self.inner
            .build_from_path(path)
            .map_err(|err| PyRuntimeError::new_err(err.to_string()))
    }

    fn lookup(&self, fragment: &str) -> PyResult<Option<Vec<u8>>> {
        self.inner
            .lookup(fragment)
            .map_err(|err| PyRuntimeError::new_err(err.to_string()))
    }

    fn lookup_with_cont(&self, fragment: &str) -> PyResult<Vec<u8>> {
        self.inner
            .lookup_with_cont(fragment)
            .map_err(|err| PyRuntimeError::new_err(err.to_string()))
    }
}

#[pymodule]
mod _core {
    #[pymodule_export]
    use super::PyFjallMorphology;
}
// fn _core(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
//     m.add_class::<PyFjallMorphology>()?;
//     Ok(())
// }
