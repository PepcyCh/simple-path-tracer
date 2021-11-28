use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    convert::{TryFrom, TryInto},
    path::PathBuf,
};

use anyhow::Context;

use crate::core::color::Color;

pub struct InputParams {
    params: HashMap<String, InputParamsValue>,
    name: Cow<'static, str>,
    visited_names: HashSet<String>,
    base_path: PathBuf,
}

pub enum InputParamsValue {
    Int(i32),
    Float(f32),
    Bool(bool),
    String(String),
    Array(Vec<InputParamsValue>),
}

macro_rules! params_get {
    ( $( ( $name:ident, $type:ty, $variant:ident, $hint:expr ) ),+ $(,)? ) => {
        $(
            paste::paste! {
                #[allow(dead_code)]
                pub fn [<get_ $name>](&mut self, key: &str) -> anyhow::Result<$type> {
                    if let Some(value) = self.params.get(key) {
                        if let InputParamsValue::$variant(value) = value {
                            self.visited_names.insert(key.to_owned());
                            return Ok(*value);
                        }
                        anyhow::bail!(format!("{} - '{}' should be {}", self.name, key, $hint));
                    }
                    anyhow::bail!(format!("{} - there is no '{}' field", self.name, key));
                }

                #[allow(dead_code)]
                pub fn [<get_ $name _or>](&mut self, key: &str, fallback: $type) -> $type {
                    if let Ok(value) = self.[<get_ $name>](key) {
                        value
                    } else {
                        fallback
                    }
                }
            }
        )+
    };
}

macro_rules! params_get_vec {
    ( $( ( $name:ident, $type:ty, $len:expr, $variant:ident, $hint:expr ) ),+ $(,)? ) => {
        $(
            paste::paste! {
                #[allow(dead_code)]
                pub fn [<get_ $name>](&mut self, key: &str) -> anyhow::Result<[$type; $len]> {
                    if let Some(value) = self.params.get(key) {
                        let error_info = format!(
                            "{} - '{}' shuold be array with {} {}s",
                            self.name,
                            key,
                            $len,
                            $hint,
                        );
                        if let InputParamsValue::Array(arr) = value {
                            if arr.len() == $len {
                                let mut result = [$type::default(); $len];
                                for i in 0..$len {
                                    if let InputParamsValue::$variant(ele) = arr[i] {
                                        result[i] = ele;
                                    } else {
                                        anyhow::bail!(error_info.clone());
                                    }
                                }
                                self.visited_names.insert(key.to_owned());
                                return Ok(result);
                            }
                        }
                        anyhow::bail!(error_info);
                    }
                    anyhow::bail!(format!("{} - there is no '{}' field", self.name, key));
                }

                #[allow(dead_code)]
                pub fn [<get_ $name _or>](
                    &mut self,
                    key: &str,
                    fallback: [$type; $len],
                ) -> [$type; $len] {
                    if let Ok(value) = self.[<get_ $name>](key) {
                        value
                    } else {
                        fallback
                    }
                }
            }
        )+
    };
}

macro_rules! params_get_array {
    ( $( ( $name:ident, $type:ty, $variant:ident, $hint:expr ) ),+ $(,)? ) => {
        $(
            paste::paste! {
                #[allow(dead_code)]
                pub fn [<get_ $name _array>](
                    &mut self,
                    key: &str,
                    len: Option<usize>,
                ) -> anyhow::Result<Vec<$type>> {
                    if let Some(value) = self.params.get(key) {
                        let error_info = if let Some(len) = len {
                            format!(
                                "{} - '{}' shuold be array with {} {}s",
                                self.name,
                                key,
                                len,
                                $hint,
                            )
                        } else {
                            format!("{} - '{}' shuold be array of {}", self.name, key, $hint)
                        };
                        if let InputParamsValue::Array(arr) = value {
                            if let Some(len) = len {
                                if arr.len() != len {
                                    anyhow::bail!(error_info.clone());
                                }
                            }
                            let mut result = Vec::with_capacity(arr.len());
                            for i in 0..arr.len() {
                                if let InputParamsValue::$variant(ele) = arr[i] {
                                    result.push(ele);
                                } else {
                                    anyhow::bail!(error_info.clone());
                                }
                            }
                            self.visited_names.insert(key.to_owned());
                            return Ok(result);
                        }
                        anyhow::bail!(error_info);
                    }
                    anyhow::bail!(format!("{} - there is no '{}' field", self.name, key));
                }

                #[allow(dead_code)]
                pub fn [<get_ $name _2darray>](
                    &mut self,
                    key: &str,
                    len1: Option<usize>,
                    len2: Option<usize>,
                ) -> anyhow::Result<Vec<Vec<$type>>> {
                    if let Some(value) = self.params.get(key) {
                        let error_info = format!("{} - '{}' shuold be 2D array with {}x{} {}s",
                            self.name,
                            key,
                            len1.map_or("?".to_owned(), |len| len.to_string()),
                            len2.map_or("?".to_owned(), |len| len.to_string()),
                            $hint,
                        );
                        if let InputParamsValue::Array(arr) = value {
                            if let Some(len1) = len1 {
                                if arr.len() != len1 {
                                    anyhow::bail!(error_info.clone());
                                }
                            }
                            let mut result = Vec::with_capacity(arr.len());
                            for i in 0..arr.len() {
                                if let InputParamsValue::Array(arr2) = &arr[i] {
                                    if let Some(len2) = len2 {
                                        if arr2.len() != len2 {
                                            anyhow::bail!(error_info.clone());
                                        }
                                    }
                                    let mut result2 = Vec::with_capacity(arr2.len());
                                    for j in 0..arr2.len() {
                                        if let InputParamsValue::$variant(ele) = &arr2[j] {
                                            result2.push(*ele);
                                        } else {
                                            anyhow::bail!(error_info.clone());
                                        }
                                    }
                                    result.push(result2);
                                } else {
                                    anyhow::bail!(error_info.clone());
                                }
                            }
                            self.visited_names.insert(key.to_owned());
                            return Ok(result);
                        }
                        anyhow::bail!(error_info);
                    }
                    anyhow::bail!(format!("{} - there is no '{}' field", self.name, key));
                }

                #[allow(dead_code)]
                pub fn [<get_ $name _3darray>](
                    &mut self,
                    key: &str,
                    len1: Option<usize>,
                    len2: Option<usize>,
                    len3: Option<usize>,
                ) -> anyhow::Result<Vec<Vec<Vec<$type>>>> {
                    if let Some(value) = self.params.get(key) {
                        let error_info = format!("{} - '{}' shuold be 3D array with {}x{}x{} {}s",
                            self.name,
                            key,
                            len1.map_or("?".to_owned(), |len| len.to_string()),
                            len2.map_or("?".to_owned(), |len| len.to_string()),
                            len3.map_or("?".to_owned(), |len| len.to_string()),
                            $hint,
                        );
                        if let InputParamsValue::Array(arr) = value {
                            if let Some(len1) = len1 {
                                if arr.len() != len1 {
                                    anyhow::bail!(error_info.clone());
                                }
                            }
                            let mut result = Vec::with_capacity(arr.len());
                            for i in 0..arr.len() {
                                if let InputParamsValue::Array(arr2) = &arr[i] {
                                    if let Some(len2) = len2 {
                                        if arr2.len() != len2 {
                                            anyhow::bail!(error_info.clone());
                                        }
                                    }
                                    let mut result2 = Vec::with_capacity(arr2.len());
                                    for j in 0..arr2.len() {
                                        if let InputParamsValue::Array(arr3) = &arr2[j] {
                                            if let Some(len3) = len3 {
                                                if arr3.len() != len3 {
                                                    anyhow::bail!(error_info.clone());
                                                }
                                            }
                                            let mut result3 = Vec::with_capacity(arr3.len());
                                            for k in 0..arr3.len() {
                                                if let InputParamsValue::$variant(ele) = &arr3[k] {
                                                    result3.push(*ele);
                                                } else {
                                                    anyhow::bail!(error_info.clone());
                                                }
                                            }
                                            result2.push(result3);
                                        } else {
                                            anyhow::bail!(error_info.clone());
                                        }
                                    }
                                    result.push(result2);
                                } else {
                                    anyhow::bail!(error_info.clone());
                                }
                            }
                            self.visited_names.insert(key.to_owned());
                            return Ok(result);
                        }
                        anyhow::bail!(error_info);
                    }
                    anyhow::bail!(format!("{} - there is no '{}' field", self.name, key));
                }
            }
        )+
    };
}

impl InputParams {
    pub fn set_name(&mut self, name: Cow<'static, str>) {
        self.name = name;
    }

    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    pub fn set_base_path(&mut self, path: PathBuf) {
        self.base_path = path;
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.params.contains_key(key)
    }

    params_get! {
        (int, i32, Int, "integer"),
        (float, f32, Float, "float"),
        (bool, bool, Bool, "boolean"),
    }

    params_get_vec! {
        (int2, i32, 2, Int, "integer"),
        (int3, i32, 3, Int, "integer"),
        (int4, i32, 4, Int, "integer"),
        (float2, f32, 2, Float, "float"),
        (float3, f32, 3, Float, "float"),
        (float4, f32, 4, Float, "float"),
    }

    params_get_array! {
        (int, i32, Int, "integer"),
        (float, f32, Float, "float"),
        (bool, bool, Bool, "boolean"),
    }

    #[allow(dead_code)]
    pub fn get_matrix(&mut self, key: &str) -> anyhow::Result<glam::Mat4> {
        if let Some(value) = self.params.get(key) {
            if let InputParamsValue::Array(arr) = value {
                let error_info =
                    format!("{} - '{}' should be an array of 16 floats", self.name, key);
                if arr.len() == 16 {
                    let mut matrix = glam::Mat4::IDENTITY;
                    for i in 0..16 {
                        if let InputParamsValue::Float(ele) = arr[i] {
                            let col_num = i / 4;
                            match i % 4 {
                                0 => matrix.col_mut(col_num).x = ele,
                                1 => matrix.col_mut(col_num).y = ele,
                                2 => matrix.col_mut(col_num).z = ele,
                                3 => matrix.col_mut(col_num).w = ele,
                                _ => panic!("unreachable match arm"),
                            }
                        }
                    }
                    self.visited_names.insert(key.to_owned());
                    return Ok(matrix);
                }
                anyhow::bail!(error_info);
            }
            anyhow::bail!(format!("{} - '{}' should be an array", self.name, key));
        }
        anyhow::bail!(format!("{} - there is no '{}' field", self.name, key));
    }

    #[allow(dead_code)]
    pub fn get_str(&mut self, key: &str) -> anyhow::Result<String> {
        if let Some(value) = self.params.get(key) {
            if let InputParamsValue::String(value) = value {
                self.visited_names.insert(key.to_owned());
                return Ok(value.clone());
            }
            anyhow::bail!(format!("{} - '{}' should be string", self.name, key));
        }
        anyhow::bail!(format!("{} - there is no '{}' field", self.name, key));
    }

    #[allow(dead_code)]
    pub fn get_str_or(&mut self, key: &str, fallback: &str) -> String {
        if let Ok(value) = self.get_str(key) {
            value
        } else {
            fallback.to_owned()
        }
    }

    #[allow(dead_code)]
    pub fn get_file_path(&mut self, key: &str) -> anyhow::Result<PathBuf> {
        let filename = self.get_str(key)?;
        let path = self.base_path.with_file_name(filename);
        Ok(path)
    }

    #[allow(dead_code)]
    pub fn get_image(&mut self, key: &str) -> anyhow::Result<image::DynamicImage> {
        let filename = self.get_str(key)?;
        let path = self.base_path.with_file_name(filename);
        let path_str = path.to_str().unwrap();
        image::open(path_str).context(format!("{} - can't read image '{}'", self.name, path_str))
    }

    #[allow(dead_code)]
    pub fn get_exr_image(&mut self, key: &str) -> anyhow::Result<Vec<Vec<Color>>> {
        let filename = self.get_str(key)?;
        let path = self.base_path.with_file_name(filename);
        let path_str = path.to_str().unwrap();
        Ok(exr::image::read::read_first_rgba_layer_from_file(
            path_str,
            |resolution: exr::math::Vec2<usize>, _| {
                vec![vec![Color::BLACK; resolution.width()]; resolution.height()]
            },
            |image, pos, (r, g, b, _): (f32, f32, f32, f32)| {
                image[pos.height()][pos.width()] = Color::new(r, g, b)
            },
        )?
        .layer_data
        .channel_data
        .pixels)
    }

    #[allow(dead_code)]
    pub fn check_unused_keys(&self) {
        for k in self.params.keys() {
            if !k.starts_with("#") && !self.visited_names.contains(k) {
                log::warn!("{} - unused key '{}'", self.name, k);
            }
        }
    }
}

impl TryFrom<&serde_json::Value> for InputParamsValue {
    type Error = anyhow::Error;

    fn try_from(value: &serde_json::Value) -> Result<Self, Self::Error> {
        match value {
            serde_json::Value::Null => {
                anyhow::bail!("can't convert to InputParamsValue from null json")
            }
            serde_json::Value::Bool(v) => Ok(Self::Bool(v.clone())),
            serde_json::Value::Number(v) => {
                if let Some(v) = v.as_i64() {
                    Ok(Self::Int(v as i32))
                } else {
                    Ok(Self::Float(v.as_f64().unwrap() as f32))
                }
            }
            serde_json::Value::String(v) => Ok(Self::String(v.clone())),
            serde_json::Value::Array(arr) => {
                let mut values = Vec::<InputParamsValue>::with_capacity(arr.len());
                for v in arr {
                    match v.try_into() {
                        Ok(v) => values.push(v),
                        Err(e) => {
                            anyhow::bail!(format!("can't convert array element: {}", e.to_string()))
                        }
                    }
                }
                Ok(Self::Array(values))
            }
            serde_json::Value::Object(_) => {
                anyhow::bail!("can't convert to InputParamsValue from object json")
            }
        }
    }
}

impl TryFrom<serde_json::Value> for InputParamsValue {
    type Error = anyhow::Error;

    fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
        match value {
            serde_json::Value::Null => {
                anyhow::bail!("can't convert to InputParamsValue from null json")
            }
            serde_json::Value::Bool(v) => Ok(Self::Bool(v)),
            serde_json::Value::Number(v) => {
                if let Some(v) = v.as_i64() {
                    Ok(Self::Int(v as i32))
                } else {
                    Ok(Self::Float(v.as_f64().unwrap() as f32))
                }
            }
            serde_json::Value::String(v) => Ok(Self::String(v)),
            serde_json::Value::Array(arr) => {
                let mut values = Vec::<InputParamsValue>::with_capacity(arr.len());
                for v in arr {
                    match v.try_into() {
                        Ok(v) => values.push(v),
                        Err(e) => {
                            anyhow::bail!(format!("can't convert array element: {}", e.to_string()))
                        }
                    }
                }
                Ok(Self::Array(values))
            }
            serde_json::Value::Object(_) => {
                anyhow::bail!("can't convert to InputParamsValue from object json")
            }
        }
    }
}

impl TryFrom<&serde_json::Value> for InputParams {
    type Error = anyhow::Error;

    fn try_from(value: &serde_json::Value) -> Result<Self, Self::Error> {
        if let serde_json::Value::Object(value) = value {
            let mut params = HashMap::<String, InputParamsValue>::with_capacity(value.len());
            for (k, v) in value {
                match v.try_into() {
                    Ok(v) => {
                        params.insert(k.clone(), v);
                    }
                    Err(e) => {
                        anyhow::bail!(format!("can't convert member '{}': {}", k, e.to_string()))
                    }
                }
            }
            Ok(Self {
                params,
                name: Cow::Owned("".to_owned()),
                visited_names: HashSet::new(),
                base_path: PathBuf::default(),
            })
        } else {
            anyhow::bail!("can't convert to InputParams from non-object json value");
        }
    }
}

impl TryFrom<serde_json::Value> for InputParams {
    type Error = anyhow::Error;

    fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
        if let serde_json::Value::Object(value) = value {
            let mut params = HashMap::<String, InputParamsValue>::with_capacity(value.len());
            for (k, v) in value {
                match v.try_into() {
                    Ok(v) => {
                        params.insert(k, v);
                    }
                    Err(e) => {
                        anyhow::bail!(format!("can't convert member '{}': {}", k, e.to_string()))
                    }
                }
            }
            Ok(Self {
                params,
                name: Cow::Owned("".to_owned()),
                visited_names: HashSet::new(),
                base_path: PathBuf::default(),
            })
        } else {
            anyhow::bail!("can't convert to InputParams from non-object json value");
        }
    }
}
