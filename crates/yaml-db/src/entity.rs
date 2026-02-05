use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::fs;
use std::marker::PhantomData;
use std::path::PathBuf;

/// Trait for types that can be stored in YAML collections
pub trait YamlEntity: Serialize + DeserializeOwned + Clone {
    /// The collection name in YAML (e.g., "tasks", "features", "disciplines")
    const COLLECTION_NAME: &'static str;
}

/// Generic YAML file manager for entity collections
/// Provides CRUD operations with atomic writes for any type implementing YamlEntity
#[derive(Debug, Clone)]
pub struct EntityFile<T: YamlEntity> {
    path: PathBuf,
    items: Vec<T>,
    _phantom: PhantomData<T>,
}

impl<T: YamlEntity> EntityFile<T> {
    /// Create a new entity file manager
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            items: Vec::new(),
            _phantom: PhantomData,
        }
    }

    /// Load entities from YAML file
    /// Returns empty collection if file doesn't exist
    pub fn load(&mut self) -> Result<(), String> {
        if !self.path.exists() {
            self.items = Vec::new();
            return Ok(());
        }

        let content = fs::read_to_string(&self.path)
            .map_err(|e| format!("Failed to read {} file: {}", T::COLLECTION_NAME, e))?;

        let data: YamlWrapper<T> = serde_yaml::from_str(&content)
            .map_err(|e| format!("Failed to parse {} YAML: {}", T::COLLECTION_NAME, e))?;

        self.items = data.items;
        Ok(())
    }

    /// Save entities to YAML file
    pub fn save(&self) -> Result<(), String> {
        let data = YamlWrapper {
            items: self.items.clone(),
            _phantom: PhantomData,
        };

        let yaml = serde_yaml::to_string(&data)
            .map_err(|e| format!("Failed to serialize {}: {}", T::COLLECTION_NAME, e))?;

        fs::write(&self.path, yaml)
            .map_err(|e| format!("Failed to write {} file: {}", T::COLLECTION_NAME, e))?;

        Ok(())
    }

    /// Save to temporary file (atomic write pattern - step 1)
    pub fn save_to_temp(&self) -> Result<(), String> {
        let temp_path = self.path.with_extension("yaml.tmp");

        let data = YamlWrapper {
            items: self.items.clone(),
            _phantom: PhantomData,
        };

        let yaml = serde_yaml::to_string(&data)
            .map_err(|e| format!("Failed to serialize {}: {}", T::COLLECTION_NAME, e))?;

        fs::write(&temp_path, yaml)
            .map_err(|e| format!("Failed to write temp {} file: {}", T::COLLECTION_NAME, e))?;

        Ok(())
    }

    /// Commit temporary file (atomic write pattern - step 2)
    pub fn commit_temp(&self) -> Result<(), String> {
        let temp_path = self.path.with_extension("yaml.tmp");
        fs::rename(&temp_path, &self.path)
            .map_err(|e| format!("Failed to rename temp {} file: {}", T::COLLECTION_NAME, e))?;
        Ok(())
    }

    /// Rollback temporary file (cleanup on error)
    pub fn rollback_temp(&self) {
        let temp_path = self.path.with_extension("yaml.tmp");
        let _ = fs::remove_file(&temp_path); // Ignore errors
    }

    /// Get all entities as a slice
    pub fn get_all(&self) -> &[T] {
        &self.items
    }

    /// Get mutable reference to items (for entity-specific operations)
    pub fn items_mut(&mut self) -> &mut Vec<T> {
        &mut self.items
    }

    /// Add an item to the collection
    pub fn add(&mut self, item: T) {
        self.items.push(item);
    }

    /// Get the file path
    pub fn path(&self) -> &PathBuf {
        &self.path
    }
}

/// Wrapper struct for YAML serialization
/// Serializes as: { collection_name: [...items...] }
#[derive(Debug, Clone)]
struct YamlWrapper<T: YamlEntity> {
    items: Vec<T>,
    _phantom: PhantomData<T>,
}

// Custom serialization to handle dynamic collection names
impl<T: YamlEntity> Serialize for YamlWrapper<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(Some(1))?;
        map.serialize_entry(T::COLLECTION_NAME, &self.items)?;
        map.end()
    }
}

// Custom deserialization to handle dynamic collection names
impl<'de, T: YamlEntity> Deserialize<'de> for YamlWrapper<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{Error, MapAccess, Visitor};
        use std::fmt;

        struct WrapperVisitor<T: YamlEntity> {
            _phantom: PhantomData<T>,
        }

        impl<'de, T: YamlEntity> Visitor<'de> for WrapperVisitor<T> {
            type Value = YamlWrapper<T>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a map with collection name as key")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut items = None;

                while let Some(key) = map.next_key::<String>()? {
                    if key == T::COLLECTION_NAME {
                        if items.is_some() {
                            return Err(Error::duplicate_field(T::COLLECTION_NAME));
                        }
                        items = Some(map.next_value()?);
                    } else {
                        // Skip unknown fields
                        map.next_value::<serde::de::IgnoredAny>()?;
                    }
                }

                let items = items.ok_or_else(|| Error::missing_field(T::COLLECTION_NAME))?;

                Ok(YamlWrapper {
                    items,
                    _phantom: PhantomData,
                })
            }
        }

        deserializer.deserialize_map(WrapperVisitor {
            _phantom: PhantomData,
        })
    }
}
