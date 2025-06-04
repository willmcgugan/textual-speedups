mod geometry;
use pyo3::prelude::*;

// /// Formats the sum of two numbers as string.
// #[pyfunction]
// fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
//     Ok((a + b).to_string())
// }

/// A Python module implemented in Rust.
#[pymodule]
fn textual_speedups(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // m.add_function(wrap_pyfunction!(sum_as_string, m)?)?;
    m.add_class::<geometry::GeometryOffset>()?;
    m.add_class::<geometry::Size>()?;
    m.add_class::<geometry::Region>()?;
    m.add_class::<geometry::Spacing>()?;
    Ok(())
}
