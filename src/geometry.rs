#![allow(dead_code)]

use pyo3::exceptions::PyIndexError;
use pyo3::exceptions::PyTypeError;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::pyclass;
use pyo3::types::PyIterator;

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use pyo3::types::PyAny;
use pyo3::types::PyRange;
use pyo3::types::PyType;
use pyo3::PyResult;
use std::cmp::Ord;

pub fn extract_integer_pair(pair: &Bound<PyAny>) -> PyResult<(i32, i32)> {
    if let Ok((x, y)) = pair.extract::<(i32, i32)>() {
        return Ok((x, y));
    }

    let iter = PyIterator::from_object(pair)?;
    let mut values = Vec::new();

    for item in iter {
        let item = item?;
        let value: i32 = item.extract()?;
        values.push(value);

        if values.len() > 2 {
            return Err(PyValueError::new_err(
                "Too many values to unpack (expected 2)",
            ));
        }
    }

    if values.len() == 2 {
        Ok((values[0], values[1]))
    } else {
        Err(PyValueError::new_err(format!(
            "Not enough values to unpack (expected 2, got {})",
            values.len()
        )))
    }
}

pub fn extract_integer_quad(pair: &Bound<PyAny>) -> PyResult<(i32, i32, i32, i32)> {
    if let Ok(quad) = pair.extract::<(i32, i32, i32, i32)>() {
        return Ok(quad);
    }

    let iter = PyIterator::from_object(pair)?;
    let mut values = Vec::new();

    for item in iter {
        let item = item?;
        let value: i32 = item.extract()?;
        values.push(value);

        if values.len() > 4 {
            return Err(PyValueError::new_err(
                "Too many values to unpack (expected 2)",
            ));
        }
    }

    if values.len() == 4 {
        Ok((values[0], values[1], values[2], values[3]))
    } else {
        Err(PyValueError::new_err(format!(
            "Not enough values to unpack (expected 4, got {})",
            values.len()
        )))
    }
}

pub fn clamp<T: Ord + Copy>(value: T, minimum: T, maximum: T) -> T {
    if minimum > maximum {
        if value < maximum {
            return maximum;
        }
        if value > minimum {
            return minimum;
        }
        value
    } else {
        if value < minimum {
            return minimum;
        }
        if value > maximum {
            return maximum;
        }
        value
    }
}

#[pyclass(name = "Offset")]
#[derive(Debug, Clone)]
pub struct GeometryOffset {
    #[pyo3(get)]
    pub x: i32,
    #[pyo3(get)]
    pub y: i32,
}

#[derive(FromPyObject)]
enum OffsetPair {
    Tuple((i32, i32)),
    Offset(GeometryOffset),
}

impl OffsetPair {
    fn to_tuple(&self) -> (i32, i32) {
        match self {
            OffsetPair::Tuple(tuple) => *tuple,
            OffsetPair::Offset(offset) => (offset.x, offset.y),
        }
    }
}

#[pymethods]
impl GeometryOffset {
    #[new]
    #[pyo3(signature=(x=0, y=0))]
    fn new(x: i32, y: i32) -> Self {
        GeometryOffset { x, y }
    }

    fn __repr__(&self) -> String {
        format!("Offset(x={}, y={})", self.x, self.y)
    }

    #[getter]
    pub fn is_origin(&self) -> bool {
        self.x == 0 && self.y == 0
    }

    #[getter]
    pub fn clamped(&self) -> Self {
        GeometryOffset {
            x: if self.x < 0 { 0 } else { self.x },
            y: if self.y < 0 { 0 } else { self.y },
        }
    }

    #[getter]
    pub fn transpose(&self) -> (i32, i32) {
        (self.y, self.x)
    }

    fn __bool__(&self) -> bool {
        self.x != 0 || self.y != 0
    }

    fn __getitem__(&self, index: isize) -> PyResult<i32> {
        let offset = if index < 0 { 2 + index } else { index };
        match offset {
            0 => Ok(self.x),
            1 => Ok(self.y),
            _ => Err(PyErr::new::<PyIndexError, _>(
                "Offset index is out of range",
            )),
        }
    }

    fn __eq__(&self, rhs: &GeometryOffset) -> bool {
        self.x == rhs.x && self.y == rhs.y
    }

    fn __hash__(&self) -> isize {
        let mut hasher = DefaultHasher::new();
        self.x.hash(&mut hasher);
        self.y.hash(&mut hasher);
        hasher.finish() as isize
    }

    fn __len__(&self) -> usize {
        2
    }

    fn __add__(&self, rhs: &Bound<PyAny>) -> PyResult<GeometryOffset> {
        if let Ok(offset) = rhs.extract::<GeometryOffset>() {
            Ok(GeometryOffset {
                x: self.x + offset.x,
                y: self.y + offset.y,
            })
        } else if let Ok((x, y)) = rhs.extract::<(i32, i32)>() {
            Ok(GeometryOffset {
                x: self.x + x,
                y: self.y + y,
            })
        } else {
            Err(PyTypeError::new_err(
                "Expected Offset or tuple of (int, int)",
            ))
        }
    }

    fn __sub__(&self, rhs: &Bound<PyAny>) -> PyResult<GeometryOffset> {
        if let Ok(offset) = rhs.extract::<GeometryOffset>() {
            Ok(GeometryOffset {
                x: self.x - offset.x,
                y: self.y - offset.y,
            })
        } else if let Ok((x, y)) = rhs.extract::<(i32, i32)>() {
            Ok(GeometryOffset {
                x: self.x - x,
                y: self.y - y,
            })
        } else {
            Err(PyTypeError::new_err(
                "Expected Offset or tuple of (int, int)",
            ))
        }
    }

    fn __mul__(&self, rhs: &Bound<PyAny>) -> PyResult<GeometryOffset> {
        if let Ok(factor) = rhs.extract::<i32>() {
            Ok(GeometryOffset {
                x: self.x * factor,
                y: self.y * factor,
            })
        } else if let Ok(factor) = rhs.extract::<f64>() {
            Ok(GeometryOffset {
                x: (self.x as f64 * factor).floor() as i32,
                y: (self.y as f64 * factor).floor() as i32,
            })
        } else if let Ok(factor) = rhs.extract::<(i32, i32)>() {
            Ok(GeometryOffset {
                x: self.x * factor.0,
                y: self.y * factor.1,
            })
        } else if let Ok(factor) = rhs.extract::<(f64, f64)>() {
            Ok(GeometryOffset {
                x: (self.x as f64 * factor.0).floor() as i32,
                y: (self.y as f64 * factor.1).floor() as i32,
            })
        } else {
            Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                "Can't multiply by this type",
            ))
        }
    }

    fn __neg__(&self) -> Self {
        GeometryOffset {
            x: -self.x,
            y: -self.y,
        }
    }

    pub fn blend(&self, destination: GeometryOffset, factor: f64) -> GeometryOffset {
        let x = self.x as f64 + (destination.x as f64 - self.x as f64) * factor;
        let y = self.y as f64 + (destination.y as f64 - self.y as f64) * factor;
        GeometryOffset {
            x: x.floor() as i32,
            y: y.floor() as i32,
        }
    }

    pub fn get_distance_to(&self, other: GeometryOffset) -> f64 {
        let dx = (other.x - self.x) as f64;
        let dy = (other.y - self.y) as f64;
        (dx * dx + dy * dy).sqrt()
    }

    pub fn clamp(&self, width: i32, height: i32) -> Self {
        GeometryOffset {
            x: clamp(self.x, 0, width - 1),
            y: clamp(self.y, 0, height - 1),
        }
    }
}

#[pyclass(frozen)]
#[derive(Debug, Clone)]
pub struct Size {
    #[pyo3(get)]
    pub width: i32,
    #[pyo3(get)]
    pub height: i32,
}

#[pymethods]
impl Size {
    #[new]
    #[pyo3(signature=(width=0, height=0))]
    fn new(width: i32, height: i32) -> Self {
        Size { width, height }
    }

    fn __repr__(&self) -> String {
        format!("Size(width={}, height={})", self.width, self.height)
    }

    fn __getitem__(&self, index: isize) -> PyResult<i32> {
        let offset = if index < 0 { 2 + index } else { index };
        match offset {
            0 => Ok(self.width),
            1 => Ok(self.height),
            _ => Err(PyIndexError::new_err("index out of range")),
        }
    }

    fn __eq__(&self, rhs: &Size) -> bool {
        self.width == rhs.width && self.height == rhs.height
    }

    fn __hash__(&self) -> isize {
        let mut hasher = DefaultHasher::new();
        self.width.hash(&mut hasher);
        self.height.hash(&mut hasher);
        hasher.finish() as isize
    }

    fn __len__(&self) -> usize {
        2
    }

    fn __bool__(&self) -> bool {
        return self.width * self.height != 0;
    }

    fn _as_tuple(&self) -> (i32, i32) {
        (self.width, self.height)
    }

    fn __add__(&self, size: &Bound<PyAny>) -> PyResult<Self> {
        if let Ok(size) = size.extract::<(i32, i32)>() {
            Ok(Size {
                width: self.width + size.0,
                height: self.height + size.1,
            })
        } else if let Ok(size) = size.extract::<Size>() {
            Ok(Size {
                width: self.width + size.width,
                height: self.height + size.height,
            })
        } else {
            Err(PyTypeError::new_err("Expected tuple of (int, int) or Size"))
        }
    }

    fn __sub__(&self, size: &Bound<PyAny>) -> PyResult<Self> {
        if let Ok(size) = size.extract::<(i32, i32)>() {
            Ok(Size {
                width: self.width - size.0,
                height: self.height - size.1,
            })
        } else if let Ok(size) = size.extract::<Size>() {
            Ok(Size {
                width: self.width - size.width,
                height: self.height - size.height,
            })
        } else {
            Err(PyTypeError::new_err("Expected tuple of (int, int) or Size"))
        }
    }

    #[getter]
    fn region(&self) -> Region {
        Region {
            x: 0,
            y: 0,
            width: self.width,
            height: self.height,
        }
    }

    #[getter]
    fn area(&self) -> i32 {
        self.width * self.height
    }

    #[getter]
    fn line_range(&self, py: Python) -> PyResult<PyObject> {
        let range = PyRange::new(py, 0, self.height as isize)?;
        Ok(range.into())
    }

    fn with_width(&self, width: i32) -> Size {
        Size {
            width,
            height: self.height,
        }
    }

    fn with_height(&self, height: i32) -> Size {
        Size {
            width: self.width,
            height,
        }
    }

    fn contains(&self, x: i32, y: i32) -> bool {
        x >= 0 && x < self.width && y >= 0 && y < self.height
    }

    fn contains_point(&self, point: &Bound<PyAny>) -> bool {
        if let Ok(point) = point.extract::<(i32, i32)>() {
            self.contains(point.0, point.1)
        } else if let Ok(point) = point.extract::<GeometryOffset>() {
            self.contains(point.x, point.y)
        } else {
            false
        }
    }

    fn __contains__(&self, rhs: &Bound<PyAny>) -> PyResult<bool> {
        if let Ok(point) = rhs.extract::<(i32, i32)>() {
            Ok(self.contains(point.0, point.1))
        } else {
            Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                "Dimensions.__contains__ requires an iterable of two integers",
            ))
        }
    }

    fn clamp_offset(&self, offset: &GeometryOffset) -> GeometryOffset {
        offset.clamp(self.width, self.height)
    }
}

#[pyclass(frozen)]
#[derive(Debug, Clone, Copy)]
pub struct Region {
    #[pyo3(get)]
    pub x: i32,
    #[pyo3(get)]
    pub y: i32,
    #[pyo3(get)]
    pub width: i32,
    #[pyo3(get)]
    pub height: i32,
}

#[pymethods]
impl Region {
    #[new]
    #[pyo3(signature=(x=0, y=0, width=0, height=0))]
    fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Region {
            x,
            y,
            width,
            height,
        }
    }

    fn __eq__(&self, rhs: &Region) -> bool {
        self.x == rhs.x && self.y == rhs.y && self.width == rhs.width && self.height == rhs.height
    }

    fn __hash__(&self) -> isize {
        let mut hasher = DefaultHasher::new();
        self.x.hash(&mut hasher);
        self.y.hash(&mut hasher);
        self.width.hash(&mut hasher);
        self.height.hash(&mut hasher);
        hasher.finish() as isize
    }

    #[classmethod]
    fn from_union(_cls: &Bound<'_, PyType>, regions: &Bound<PyAny>) -> PyResult<Region> {
        let mut region_vec = Vec::new();

        // Try to iterate over the input
        let iter = PyIterator::from_object(regions)?;

        for item_result in iter {
            let item = item_result?;

            // Extract each item as a Region
            let region = item.extract::<PyRef<Region>>()?;
            region_vec.push(region);
        }
        if region_vec.is_empty() {
            return Err(PyValueError::new_err("At least one region expected"));
        }

        let min_x = region_vec.iter().map(|r| r.x).min().unwrap();
        let max_x = region_vec.iter().map(|r| r.right()).max().unwrap();
        let min_y = region_vec.iter().map(|r| r.y).min().unwrap();
        let max_y = region_vec.iter().map(|r| r.bottom()).max().unwrap();

        Ok(Region {
            x: min_x,
            y: min_y,
            width: max_x - min_x,
            height: max_y - min_y,
        })
    }

    #[classmethod]
    fn from_corners(_cls: &Bound<'_, PyType>, x1: i32, y1: i32, x2: i32, y2: i32) -> Region {
        Region {
            x: x1,
            y: y1,
            width: x2 - x1,
            height: y2 - y1,
        }
    }

    #[classmethod]
    fn from_offset(
        _cls: &Bound<'_, PyType>,
        offset: &Bound<PyAny>,
        size: &Bound<PyAny>,
    ) -> PyResult<Region> {
        let (x, y) = extract_integer_pair(offset)?;
        let (width, height) = extract_integer_pair(size)?;
        Ok(Region {
            x,
            y,
            width,
            height,
        })
    }

    #[classmethod]
    #[pyo3(signature=(window_region, region, *, top = false))]
    pub fn get_scroll_to_visible(
        _cls: &Bound<'_, PyType>,
        window_region: &Region,
        region: &Region,
        top: bool,
    ) -> GeometryOffset {
        if !top && window_region.contains_region(region) {
            // Region is already inside the window, so no need to move it.
            return GeometryOffset { x: 0, y: 0 };
        }

        let (window_left, window_top, window_right, window_bottom) = window_region.corners();
        let region = region._crop_size(window_region.size()._as_tuple());
        let (left, top_, right, bottom) = region.corners();
        let mut delta_x = 0;
        let mut delta_y = 0;

        if !((window_right > left && left >= window_left)
            && (window_right > right && right >= window_left))
        {
            // The region does not fit
            // The window needs to scroll on the X axis to bring region into view
            let option1 = left - window_left;
            let option2 = left - (window_right - region.width);
            delta_x = if option1.abs() < option2.abs() {
                option1
            } else {
                option2
            };
        }

        if top {
            delta_y = top_ - window_top;
        } else if !((window_bottom > top_ && top_ >= window_top)
            && (window_bottom > bottom && bottom >= window_top))
        {
            // The window needs to scroll on the Y axis to bring region into view
            let option1 = top_ - window_top;
            let option2 = top_ - (window_bottom - region.height);
            delta_y = if option1.abs() < option2.abs() {
                option1
            } else {
                option2
            };
        }

        GeometryOffset {
            x: delta_x,
            y: delta_y,
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "Region(x={}, y={}, width={}, height={})",
            self.x, self.y, self.width, self.height
        )
    }

    fn __getitem__(&self, index: isize) -> PyResult<i32> {
        let offset = if index < 0 { 4 + index } else { index };
        match offset {
            0 => Ok(self.x),
            1 => Ok(self.y),
            2 => Ok(self.width),
            3 => Ok(self.height),
            _ => Err(PyIndexError::new_err("index out of range")),
        }
    }

    fn __len__(&self) -> usize {
        4
    }

    fn __bool__(&self) -> bool {
        return self.width * self.height > 0;
    }

    fn __add__(&self, rhs: &Bound<PyAny>) -> PyResult<Region> {
        let (x, y) = extract_integer_pair(rhs)?;
        Ok(Region {
            x: self.x + x,
            y: self.y + y,
            width: self.width,
            height: self.height,
        })
    }

    fn __sub__(&self, rhs: &Bound<PyAny>) -> PyResult<Region> {
        let (x, y) = extract_integer_pair(rhs)?;
        Ok(Region {
            x: self.x - x,
            y: self.y - y,
            width: self.width,
            height: self.height,
        })
    }

    fn get_spacing_between(&self, region: &Region) -> Spacing {
        Spacing {
            top: region.y - self.y,
            right: self.right() - region.right(),
            bottom: self.bottom() - region.bottom(),
            left: region.x - self.x,
        }
    }

    #[getter]
    fn column_span(&self) -> (i32, i32) {
        (self.x, self.x + self.width)
    }

    #[getter]
    fn line_span(&self) -> (i32, i32) {
        (self.y, self.y + self.height)
    }

    #[getter]
    fn right(&self) -> i32 {
        self.x + self.width
    }

    #[getter]
    fn bottom(&self) -> i32 {
        self.y + self.height
    }

    #[getter]
    fn area(&self) -> i32 {
        self.width * self.height
    }

    #[getter]
    fn offset(&self) -> GeometryOffset {
        GeometryOffset {
            x: self.x,
            y: self.y,
        }
    }

    #[getter]
    fn center(&self) -> (f64, f64) {
        let Region {
            x,
            y,
            width,
            height,
        } = *self;
        (
            x as f64 + (width as f64) / 2.0,
            y as f64 + (height as f64) / 2.0,
        )
    }

    #[getter]
    fn bottom_left(&self) -> GeometryOffset {
        GeometryOffset {
            x: self.x,
            y: self.y + self.height,
        }
    }

    #[getter]
    fn top_right(&self) -> GeometryOffset {
        GeometryOffset {
            x: self.x + self.width,
            y: self.y,
        }
    }

    #[getter]
    fn bottom_right(&self) -> GeometryOffset {
        GeometryOffset {
            x: self.x + self.width,
            y: self.y + self.height,
        }
    }

    #[getter]
    fn bottom_right_inclusive(&self) -> GeometryOffset {
        GeometryOffset {
            x: self.x + self.width - 1,
            y: self.y + self.height - 1,
        }
    }

    #[getter]
    fn size(&self) -> Size {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    #[getter]
    fn corners(&self) -> (i32, i32, i32, i32) {
        let Region {
            x,
            y,
            width,
            height,
        } = *self;
        (x, y, x + width, y + height)
    }

    #[getter]
    fn column_range(&self, py: Python) -> PyResult<PyObject> {
        let range = PyRange::new(py, self.x as isize, (self.x + self.width) as isize)?;
        Ok(range.into())
    }

    #[getter]
    fn line_range(&self, py: Python) -> PyResult<PyObject> {
        let range = PyRange::new(py, self.y as isize, (self.y + self.height) as isize)?;
        Ok(range.into())
    }

    #[getter]
    fn reset_offset(&self) -> Self {
        Region {
            x: 0,
            y: 0,
            width: self.width,
            height: self.height,
        }
    }

    fn at_offset(&self, offset: (i32, i32)) -> Region {
        Region {
            x: offset.0,
            y: offset.1,
            width: self.width,
            height: self.height,
        }
    }

    fn crop_size(&self, size: &Bound<PyAny>) -> PyResult<Region> {
        let size = extract_integer_pair(size)?;
        Ok(self._crop_size(size))
    }

    fn _crop_size(&self, size: (i32, i32)) -> Region {
        Region {
            x: self.x,
            y: self.y,
            width: self.width.min(size.0),
            height: self.height.min(size.1),
        }
    }

    fn expand(&self, size: (i32, i32)) -> Region {
        let (expand_width, expand_height) = size;
        Region {
            x: self.x - expand_width,
            y: self.y - expand_height,
            width: self.width + expand_width * 2,
            height: self.height + expand_height * 2,
        }
    }

    fn overlaps(&self, other: &Region) -> bool {
        let (x, y, x2, y2) = self.corners();
        let (ox, oy, ox2, oy2) = other.corners();
        return ((x2 > ox && ox >= x) || (x2 > ox2 && ox2 > x) || (ox < x && ox2 >= x2))
            && ((y2 > oy && oy >= y) || (y2 > oy2 && oy2 > y) || (oy < y && oy2 >= y2));
    }

    fn contains(&self, x: i32, y: i32) -> bool {
        self.x + self.width > x && x >= self.x && self.y + self.height > y && y >= self.y
    }

    fn contains_point(&self, point: &Bound<PyAny>) -> PyResult<bool> {
        if let Ok((x, y)) = point.extract::<(i32, i32)>() {
            Ok(self.contains(x, y))
        } else if let Ok(GeometryOffset { x, y }) = point.extract::<GeometryOffset>() {
            Ok(self.contains(x, y))
        } else {
            Err(PyTypeError::new_err("Expected tuple of (int, int)"))
        }
    }

    fn contains_region(&self, other: &Region) -> bool {
        let (x1, y1, x2, y2) = self.corners();
        let (ox, oy, ox2, oy2) = other.corners();
        return (x2 >= ox && ox >= x1)
            && (y2 >= oy && oy >= y1)
            && (x2 >= ox2 && ox2 >= x1)
            && (y2 >= oy2 && oy2 >= y1);
    }

    fn translate(&self, offset: OffsetPair) -> Region {
        let (offset_x, offset_y) = offset.to_tuple();
        Region {
            x: self.x + offset_x,
            y: self.y + offset_y,
            width: self.width,
            height: self.height,
        }
    }

    fn __contains__(&self, rhs: &Bound<PyAny>) -> bool {
        if let Ok(region) = rhs.extract::<Region>() {
            self.contains_region(&region)
        } else if let Ok((x, y)) = rhs.extract::<(i32, i32)>() {
            self.contains(x, y)
        } else if let Ok(GeometryOffset { x, y }) = rhs.extract::<GeometryOffset>() {
            self.contains(x, y)
        } else {
            return false;
        }
    }

    fn clip(&self, width: i32, height: i32) -> Region {
        let (x1, y1, x2, y2) = self.corners();
        let x = clamp(x1, 0, width);
        let y = clamp(y1, 0, height);
        Region {
            x,
            y,
            width: clamp(x2, 0, width) - x,
            height: clamp(y2, 0, height) - y,
        }
    }

    fn grow(&self, margin: &Bound<PyAny>) -> PyResult<Region> {
        let grow_margin = extract_integer_quad(margin)?;
        if grow_margin == (0, 0, 0, 0) {
            return Ok(self.clone());
        }
        Ok(self._grow(grow_margin))
    }

    fn _grow(&self, margin: (i32, i32, i32, i32)) -> Region {
        if margin == (0, 0, 0, 0) {
            return self.clone();
        }
        let (top, right, bottom, left) = margin;
        let Region {
            x,
            y,
            width,
            height,
        } = self;
        Region {
            x: x - left,
            y: y - top,
            width: 0.max(width + (left + right)),
            height: 0.max(height + (top + bottom)),
        }
    }

    fn shrink(&self, margin: &Bound<PyAny>) -> PyResult<Region> {
        let shrink_margin = extract_integer_quad(margin)?;
        if shrink_margin == (0, 0, 0, 0) {
            return Ok(self.clone());
        }
        Ok(self._shrink(shrink_margin))
    }

    fn _shrink(&self, margin: (i32, i32, i32, i32)) -> Region {
        if margin == (0, 0, 0, 0) {
            return self.clone();
        }
        let (top, right, bottom, left) = margin;
        let Region {
            x,
            y,
            width,
            height,
        } = self;
        Region {
            x: x + left,
            y: y + top,
            width: 0.max(width - (left + right)),
            height: 0.max(height - (top + bottom)),
        }
    }

    fn intersection(&self, region: &Region) -> Region {
        let (x1, y1, w1, h1) = (self.x, self.y, self.width, self.height);
        let (cx1, cy1, w2, h2) = (region.x, region.y, region.width, region.height);
        let x2 = x1 + w1;
        let y2 = y1 + h1;
        let cx2 = cx1 + w2;
        let cy2 = cy1 + h2;

        let rx1 = if x1 > cx2 {
            cx2
        } else if x1 < cx1 {
            cx1
        } else {
            x1
        };
        let ry1 = if y1 > cy2 {
            cy2
        } else if y1 < cy1 {
            cy1
        } else {
            y1
        };
        let rx2 = if x2 > cx2 {
            cx2
        } else if x2 < cx1 {
            cx1
        } else {
            x2
        };
        let ry2 = if y2 > cy2 {
            cy2
        } else if y2 < cy1 {
            cy1
        } else {
            y2
        };

        Region {
            x: rx1,
            y: ry1,
            width: rx2 - rx1,
            height: ry2 - ry1,
        }
    }

    fn union(&self, region: &Region) -> Region {
        let (x1, y1, x2, y2) = self.corners();
        let (ox1, oy1, ox2, oy2) = region.corners();
        let x = x1.min(ox1);
        let y = y1.min(oy1);
        Region {
            x,
            y,
            width: x2.max(ox2) - x,
            height: y2.max(oy2) - y,
        }
    }

    fn split(&self, mut cut_x: i32, mut cut_y: i32) -> (Region, Region, Region, Region) {
        let Region {
            x,
            y,
            width,
            height,
        } = *self;

        if cut_x < 0 {
            cut_x = width + cut_x;
        }
        if cut_y < 0 {
            cut_y = height + cut_y;
        }
        (
            Region {
                x: x,
                y: y,
                width: cut_x,
                height: cut_y,
            },
            Region {
                x: x + cut_x,
                y: y,
                width: width - cut_x,
                height: cut_y,
            },
            Region {
                x: x,
                y: y + cut_y,
                width: cut_x,
                height: height - cut_y,
            },
            Region {
                x: x + cut_x,
                y: y + cut_y,
                width: width - cut_x,
                height: height - cut_y,
            },
        )
    }

    fn split_horizontal(&self, mut cut: i32) -> (Region, Region) {
        let Region {
            x,
            y,
            width,
            height,
        } = *self;
        if cut < 0 {
            cut = height + cut;
        }
        (
            Region {
                x: x,
                y: y,
                width: width,
                height: cut,
            },
            Region {
                x: x,
                y: y + cut,
                width: width,
                height: height - cut,
            },
        )
    }

    fn split_vertical(&self, mut cut: i32) -> (Region, Region) {
        let Region {
            x,
            y,
            width,
            height,
        } = *self;
        if cut < 0 {
            cut = width + cut;
        }
        (
            Region {
                x,
                y,
                width: cut,
                height,
            },
            Region {
                x: x + cut,
                y,
                width: width - cut,
                height,
            },
        )
    }

    #[pyo3(signature=(container, x_axis=true, y_axis=true))]
    fn translate_inside(&self, container: &Region, x_axis: bool, y_axis: bool) -> Region {
        let Region {
            x: x1,
            y: y1,
            width: width1,
            height: height1,
        } = *container;
        let Region {
            x: x2,
            y: y2,
            width: width2,
            height: height2,
        } = *self;
        Region {
            x: if x_axis {
                x2.min(x1 + width1 - width2).max(x1)
            } else {
                x2
            },
            y: if y_axis {
                y2.min(y1 + height1 - height2).max(y1)
            } else {
                y2
            },
            width: width2,
            height: height2,
        }
    }

    #[pyo3(signature = (x_axis=1, y_axis=1, margin=None))]
    fn inflect(&self, x_axis: i32, y_axis: i32, margin: Option<Spacing>) -> Region {
        let inflect_margin = margin.unwrap_or(Spacing {
            top: 0,
            right: 0,
            bottom: 0,
            left: 0,
        });
        let Region {
            mut x,
            mut y,
            width,
            height,
        } = *self;
        if x_axis != 0 {
            x = x + (width + inflect_margin.max_width()) * x_axis;
        }
        if y_axis != 0 {
            y = y + (height + inflect_margin.max_height()) * y_axis;
        }
        Region {
            x,
            y,
            width,
            height,
        }
    }

    fn constrain(
        &self,
        constrain_x: &str,
        constrain_y: &str,
        margin: &Spacing,
        container: &Region,
    ) -> Region {
        let margin_region = self._grow(margin._as_tuple());
        let mut region = *self;

        fn compare_span(
            span_start: i32,
            span_end: i32,
            container_start: i32,
            container_end: i32,
        ) -> i32 {
            if span_start > container_start && span_end <= container_end {
                0
            } else if span_start < container_start {
                -1
            } else {
                1
            }
        }

        if constrain_x == "inflect" || constrain_y == "inflect" {
            let x_axis = if constrain_x == "inflect" {
                -compare_span(
                    margin_region.x,
                    margin_region.right(),
                    container.x,
                    container.right(),
                )
            } else {
                0
            };
            let y_axis = if constrain_y == "inflect" {
                -compare_span(
                    margin_region.y,
                    margin_region.bottom(),
                    container.y,
                    container.bottom(),
                )
            } else {
                0
            };
            region = region.inflect(x_axis, y_axis, Some(*margin))
        }

        region.translate_inside(
            &container._shrink(margin._as_tuple()),
            constrain_x != "none",
            constrain_y != "none",
        )
    }
}

enum SpacingDimensions {
    Single(i32),
    Tuple1(i32),
    Tuple2(i32, i32),
    Tuple4(i32, i32, i32, i32),
}

#[pyclass(frozen)]
#[derive(Debug, Clone, Copy)]
pub struct Spacing {
    #[pyo3(get)]
    pub top: i32,
    #[pyo3(get)]
    pub right: i32,
    #[pyo3(get)]
    pub bottom: i32,
    #[pyo3(get)]
    pub left: i32,
}

#[pymethods]
impl Spacing {
    #[new]
    #[pyo3(signature=(top=0, right=0, bottom=0, left=0))]
    fn new(top: i32, right: i32, bottom: i32, left: i32) -> Spacing {
        Spacing {
            top,
            right,
            bottom,
            left,
        }
    }
    fn __repr__(&self) -> String {
        format!(
            "Spacing(top={}, right={}, bottom={}, left={})",
            self.top, self.right, self.bottom, self.left
        )
    }

    fn __getitem__(&self, index: isize) -> PyResult<i32> {
        let offset = if index < 0 { 4 + index } else { index };
        match offset {
            0 => Ok(self.top),
            1 => Ok(self.right),
            2 => Ok(self.bottom),
            3 => Ok(self.left),
            _ => Err(PyIndexError::new_err("index out of range")),
        }
    }

    fn __len__(&self) -> usize {
        4
    }

    fn __eq__(&self, rhs: &Spacing) -> bool {
        self.top == rhs.top
            && self.right == rhs.right
            && self.bottom == rhs.bottom
            && self.left == rhs.left
    }

    fn __hash__(&self) -> isize {
        let mut hasher = DefaultHasher::new();
        self.top.hash(&mut hasher);
        self.right.hash(&mut hasher);
        self.bottom.hash(&mut hasher);
        self.left.hash(&mut hasher);
        hasher.finish() as isize
    }

    fn _as_tuple(&self) -> (i32, i32, i32, i32) {
        (self.top, self.right, self.bottom, self.left)
    }

    fn __add__(&self, rhs: &Bound<PyAny>) -> PyResult<Spacing> {
        if let Ok((top, right, bottom, left)) = rhs.extract::<(i32, i32, i32, i32)>() {
            Ok(Spacing {
                top: self.top + top,
                right: self.right + right,
                bottom: self.bottom + bottom,
                left: self.left + left,
            })
        } else if let Ok(Spacing {
            top,
            right,
            bottom,
            left,
        }) = rhs.extract::<Spacing>()
        {
            Ok(Spacing {
                top: self.top + top,
                right: self.right + right,
                bottom: self.bottom + bottom,
                left: self.left + left,
            })
        } else {
            Err(PyTypeError::new_err(
                "Expected tuple of (int, int, int, int)",
            ))
        }
    }

    fn __sub__(&self, rhs: &Bound<PyAny>) -> PyResult<Spacing> {
        if let Ok((top, right, bottom, left)) = rhs.extract::<(i32, i32, i32, i32)>() {
            Ok(Spacing {
                top: self.top - top,
                right: self.right - right,
                bottom: self.bottom - bottom,
                left: self.left - left,
            })
        } else if let Ok(Spacing {
            top,
            right,
            bottom,
            left,
        }) = rhs.extract::<Spacing>()
        {
            Ok(Spacing {
                top: self.top - top,
                right: self.right - right,
                bottom: self.bottom - bottom,
                left: self.left - left,
            })
        } else {
            Err(PyTypeError::new_err(
                "Expected tuple of (int, int, int, int)",
            ))
        }
    }

    #[getter]
    fn width(&self) -> i32 {
        self.left + self.right
    }

    #[getter]
    fn height(&self) -> i32 {
        self.top + self.bottom
    }

    #[getter]
    fn max_width(&self) -> i32 {
        self.left.max(self.right)
    }

    #[getter]
    fn max_height(&self) -> i32 {
        self.top.max(self.bottom)
    }

    #[getter]
    fn top_left(&self) -> (i32, i32) {
        (self.left, self.top)
    }

    #[getter]
    fn bottom_right(&self) -> (i32, i32) {
        (self.right, self.bottom)
    }

    #[getter]
    fn totals(&self) -> (i32, i32) {
        (self.left + self.right, self.top + self.bottom)
    }

    fn __bool__(&self) -> bool {
        self.top != 0 || self.right != 0 || self.bottom != 0 || self.left != 0
    }

    #[getter]
    fn css(&self) -> String {
        let Spacing {
            top,
            right,
            bottom,
            left,
        } = *self;
        if top == right && right == bottom && bottom == left && left == top {
            format!("{}", top)
        } else if (top, right) == (bottom, left) {
            format!("{} {}", top, right)
        } else {
            format!("{} {} {} {}", top, right, bottom, left)
        }
    }

    #[classmethod]
    fn unpack(_cls: &Bound<'_, PyType>, pad: &Bound<PyAny>) -> PyResult<Spacing> {
        if let Ok(space) = pad.extract::<i32>() {
            Ok(Spacing {
                top: space,
                right: space,
                bottom: space,
                left: space,
            })
        } else if let Ok((space,)) = pad.extract::<(i32,)>() {
            Ok(Spacing {
                top: space,
                right: space,
                bottom: space,
                left: space,
            })
        } else if let Ok((top, right)) = pad.extract::<(i32, i32)>() {
            Ok(Spacing {
                top: top,
                right: right,
                bottom: top,
                left: right,
            })
        } else if let Ok((top, right, bottom, left)) = pad.extract::<(i32, i32, i32, i32)>() {
            Ok(Spacing {
                top: top,
                right: right,
                bottom: bottom,
                left: left,
            })
        } else {
            let iter = PyIterator::from_object(pad)?;
            let mut values = Vec::new();

            for item in iter {
                let item = item?;
                let value: i32 = item.extract()?;
                values.push(value);

                if values.len() > 4 {
                    return Err(PyValueError::new_err(
                        "Too many values to unpack (expected max 4)",
                    ));
                }
            }

            match values.len() {
                1 => Ok(Spacing {
                    top: values[0],
                    right: values[0],
                    bottom: values[0],
                    left: values[0],
                }),
                2 => Ok(Spacing {
                    top: values[0],
                    right: values[1],
                    bottom: values[0],
                    left: values[1],
                }),
                4 => Ok(Spacing {
                    top: values[0],
                    right: values[1],
                    bottom: values[2],
                    left: values[3],
                }),
                _ => Err(PyValueError::new_err(
                    "Expected integer or tuple of 1, 2, 4 integers",
                )),
            }
        }
    }

    #[classmethod]
    fn vertical(_cls: &Bound<'_, PyType>, amount: i32) -> Spacing {
        Spacing {
            top: amount,
            right: 0,
            bottom: amount,
            left: 0,
        }
    }

    #[classmethod]
    fn horizontal(_cls: &Bound<'_, PyType>, amount: i32) -> Spacing {
        Spacing {
            top: 0,
            right: amount,
            bottom: 0,
            left: amount,
        }
    }

    #[classmethod]
    fn all(_cls: &Bound<'_, PyType>, amount: i32) -> Spacing {
        Spacing {
            top: amount,
            right: amount,
            bottom: amount,
            left: amount,
        }
    }

    fn grow_maximum(&self, other: &Spacing) -> Spacing {
        let Spacing {
            top,
            right,
            bottom,
            left,
        } = *self;
        let Spacing {
            top: other_top,
            right: other_right,
            bottom: other_bottom,
            left: other_left,
        } = *other;
        Spacing {
            top: top.max(other_top),
            right: right.max(other_right),
            bottom: bottom.max(other_bottom),
            left: left.max(other_left),
        }
    }
}
