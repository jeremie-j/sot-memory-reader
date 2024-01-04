use std::collections::HashMap;
use std::fs;

use serde::de::{self, Deserializer};
use serde::Deserialize;

fn deserialize_hex<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if s.starts_with('-') {
        return Ok(0);
    }
    u32::from_str_radix(&s[2..], 16).map_err(de::Error::custom)
}

#[derive(Deserialize)]
pub struct SdkAttribute {
    pub Name: String,
    pub Type: String,
    #[serde(deserialize_with = "deserialize_hex")]
    pub Size: u32,
    #[serde(deserialize_with = "deserialize_hex")]
    pub Offset: u32,
}

#[derive(Deserialize)]
pub struct SdkClass {
    pub Super: String,
    #[serde(deserialize_with = "deserialize_hex")]
    pub FullSize: u32,
    #[serde(deserialize_with = "deserialize_hex")]
    pub InheritedSize: u32,
    #[serde(deserialize_with = "deserialize_hex")]
    pub ClassSize: u32,
    pub Attributes: Vec<SdkAttribute>,
}

#[derive(Deserialize)]
pub struct SdkStruct {
    #[serde(deserialize_with = "deserialize_hex")]
    pub ClassSize: u32,
    pub Attributes: Vec<SdkAttribute>,
}

pub struct SdkService {
    pub classes: HashMap<String, SdkClass>,
    pub structs: HashMap<String, SdkStruct>,
}
impl SdkService {
    pub fn new() -> Self {
        Self {
            classes: HashMap::new(),
            structs: HashMap::new(),
        }
    }

    pub fn scan_sdk(&mut self) {
        let sdk_path = option_env!("SDK_PATH").unwrap_or("./JSON-SDK");
        let paths = fs::read_dir(sdk_path).unwrap();
        for path in paths {
            if let Ok(path) = path {
                let path_buff = path.path();
                let string_path = path_buff.to_str().unwrap();

                let file = fs::File::open(path.path()).unwrap();
                if string_path.ends_with("Classes.json") {
                    match serde_json::from_reader::<_, HashMap<String, SdkClass>>(file) {
                        Ok(v) => self.classes.extend(v),
                        Err(v) => panic!("File: {} \n{}", string_path, v),
                    }
                } else if string_path.ends_with("Structs.json") {
                    match serde_json::from_reader::<_, HashMap<String, SdkStruct>>(file) {
                        Ok(v) => self.structs.extend(v),
                        Err(v) => panic!("File: {} \n{}", string_path, v),
                    }
                } else {
                    println!("Could not scan {}", string_path);
                    continue;
                }
            }
        }
    }

    /// ## Example:
    /// ```
    /// sdk_service.get_offset("Actor.bHidden") // 124
    /// ```
    pub fn get_offset(&self, attribute_path: &str) -> u32 {
        let split = attribute_path.split('.').collect::<Vec<&str>>();
        let (struct_or_class_name, attribute_name) = match split.as_slice() {
            [first, second, ..] => (first, second),
            _ => panic!("Expected at least one '.' in attribute_path"),
        };
        let attributes = match self.get_attributes(struct_or_class_name) {
            Some(v) => v,
            None => panic!(
                "{}",
                format!(
                    "Class or Struct \"{}\" does not exist",
                    struct_or_class_name
                )
            ),
        };

        match attributes.iter().find(|v| &v.Name == attribute_name) {
            Some(v) => v.Offset,
            None => panic!(
                "{}",
                format!(
                    "Class or Struct attribute \"{}\" does not exist",
                    attribute_name
                )
            ),
        }
    }

    pub fn get_size(&self, attribute_path: &'static str) -> u32 {
        let split = attribute_path.split('.').collect::<Vec<&str>>();
        let (struct_or_class_name, attribute_name) = match split.as_slice() {
            [first, second, ..] => (first, second),
            _ => panic!("Expected at least one '.' in attribute_path"),
        };
        let attributes = match self.get_attributes(struct_or_class_name) {
            Some(v) => v,
            None => panic!(
                "{}",
                format!("Class or Struct \"{}\" does not exist", attribute_name)
            ),
        };

        match attributes.iter().find(|v| &v.Name == attribute_name) {
            Some(v) => v.Size,
            None => panic!(
                "{}",
                format!(
                    "Class or Struct attribute \"{}\" does not exist",
                    attribute_name
                )
            ),
        }
    }

    fn get_attributes(&self, struct_or_class_name: &str) -> Option<&Vec<SdkAttribute>> {
        if let Some(class) = self.classes.get(struct_or_class_name) {
            Some(&class.Attributes)
        } else if let Some(struct_) = self.structs.get(struct_or_class_name) {
            Some(&struct_.Attributes)
        } else {
            None
        }
    }
}
