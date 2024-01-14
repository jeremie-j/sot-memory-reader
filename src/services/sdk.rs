use std::collections::HashMap;
use std::fs;
use std::sync::OnceLock;

use serde::de::{self, Deserializer};
use serde::Deserialize;

const FILE_WHITELIST: [&str; 10] = [
    "BP_EmissaryTable_01_Classes.json",
    "BP_EmissaryTable_GoldHoarder_01_Classes.json",
    "BP_EmissaryTable_MerchantAlliance_01_Classes.json",
    "BP_EmissaryTable_OrderOfSouls_01_Classes.json",
    "BP_EmissaryTable_Sov_01_a_Classes.json",
    "BP_FactionEmissaryTable_Athena_Classes.json",
    "BP_FactionEmissaryTable_Reapers_Classes.json",
    "EmissaryLevel_Classes.json",
    "Athena_Structs.json",
    "Athena_Classes.json",
];

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
#[allow(non_snake_case)]
pub struct SdkAttribute {
    pub Name: String,
    pub Type: String,
    #[serde(deserialize_with = "deserialize_hex")]
    pub Size: u32,
    #[serde(deserialize_with = "deserialize_hex")]
    pub Offset: u32,
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
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
#[allow(non_snake_case)]
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
        let paths = fs::read_dir(sdk_path)
            .unwrap()
            .filter_map(|path| match path {
                Ok(p) => Some(p),
                Err(_) => None,
            });

        for path in paths {
            let file_name = path.file_name().into_string().unwrap();
            if !FILE_WHITELIST.contains(&file_name.as_str()) {
                continue;
            }

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

    pub fn get_attribute_size(&self, attribute_path: &'static str) -> u32 {
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

    pub fn get_class_or_struct_size(&self, struct_or_class_name: &str) -> u32 {
        if let Some(class) = self.classes.get(struct_or_class_name) {
            return class.ClassSize;
        } else if let Some(struct_) = self.structs.get(struct_or_class_name) {
            return struct_.ClassSize;
        } else {
            panic!(
                "{}",
                format!(
                    "Class or Struct \"{}\" does not exist",
                    struct_or_class_name
                )
            );
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

pub fn sdk_service() -> &'static SdkService {
    static SDK_SERVICE: OnceLock<SdkService> = OnceLock::new();
    SDK_SERVICE.get_or_init(|| {
        let mut sdk_service = SdkService::new();
        sdk_service.scan_sdk();
        sdk_service
    })
}
