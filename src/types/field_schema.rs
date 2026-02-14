use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct FieldSchema {
    pub label: String,
    pub key: String,
    #[serde(rename = "type")]
    pub field_type: String,
    pub required: Option<bool>,
    pub options: Option<Vec<String>>,
    pub storage: Option<String>,
    pub display: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
#[serde(rename_all = "snake_case")]
pub struct DocTypeSchema {
    pub label: String,
    pub has_markdown: bool,
    pub fields: Vec<FieldSchema>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FrontendSchema {
    pub types: HashMap<DocType, DocTypeSchema>,
}

/*
impl Deref for FrontendSchema {
    type Target = HashMap<DocType, DocTypeSchema>;

    fn deref(&self) -> &Self::Target {
        &self.types
    }
}
*/
use std::fs;

use crate::errors::app_error::AppError;
use crate::types::DocType;

pub fn load_frontend_schema(path: &str) -> Result<FrontendSchema, AppError> {
    match fs::read_to_string(path) {
        Ok(file_content) => match serde_json::from_str(&file_content) {
            Ok(schema) => Ok(schema),
            Err(e) => {
                log::error!("Could not parse schema {}: {}", path, e);
                Err(AppError::Internal(e.to_string()))
            }
        },
        Err(e) => Err(AppError::Internal(e.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_load_frontend_schema() {
        // Create a temp file path
        let mut path = std::env::temp_dir();
        path.push("test_frontend_schema.json");

        // Minimal valid schema JSON
        let json = r#"{
  "types": {
    "recipe": {
      "label": "Recipe",
      "has_markdown": true,
      "fields": [
        {
          "label": "Title",
          "key": "title",
          "type": "text",
          "required": true
        },
        {
          "label": "Description",
          "key": "description",
          "type": "textarea",
          "required": true,
          "storage": "column"
        },
        {
          "label": "Author",
          "key": "author",
          "type": "text",
          "required": true
        },
        {
          "label": "Tags",
          "key": "tags",
          "type": "text",
          "required": true,
          "storage": "meta"
        },
        {
          "label": "Prep Time",
          "key": "prep_time",
          "type": "number",
          "storage": "meta"
        },
        {
          "label": "Cook Time",
          "key": "cook_time",
          "type": "number",
          "storage": "meta"
        },
        {
          "label": "Servings",
          "key": "servings",
          "type": "number",
          "storage": "meta"
        },
        {
          "label": "Difficulty",
          "key": "difficulty",
          "type": "select",
          "options": [
            "easy",
            "medium",
            "hard"
          ],
          "storage": "meta"
        },
        {
          "label": "Dietary",
          "key": "dietary",
          "type": "checkbox",
          "options": [
            "gluten-free",
            "non-dairy",
            "vegetarian",
            "vegan"
          ],
          "storage": "meta"
        },
        {
          "label": "Source",
          "key": "source",
          "type": "text",
          "storage": "meta"
        },
        {
          "label": "Body Markdown",
          "key": "body_md",
          "type": "markdown",
          "required": true
        }
      ]
    },
    "organization": {
      "label": "Organization",
      "has_markdown": true,
      "fields": [
        {
          "label": "Title",
          "key": "title",
          "type": "text",
          "required": true
        },
        {
          "label": "Description",
          "key": "description",
          "type": "textarea"
        },
        {
          "label": "Author",
          "key": "author",
          "type": "text"
        },
        {
          "label": "Tags",
          "key": "tags",
          "type": "text",
          "storage": "meta"
        },
        {
          "label": "Location",
          "key": "location",
          "type": "text",
          "storage": "meta"
        },
        {
          "label": "Website",
          "key": "website",
          "type": "text",
          "storage": "meta"
        },
        {
          "label": "Featured Image",
          "key": "featured_image",
          "type": "text",
          "storage": "meta"
        },
        {
          "label": "body_md",
          "key": "body_md",
          "type": "hidden",
          "display": false
        }
      ]
    },
    "event": {
      "label": "Event",
      "has_markdown": true,
      "fields": [
        {
          "label": "Title",
          "key": "title",
          "type": "text",
          "required": true
        },
        {
          "label": "Description",
          "key": "description",
          "type": "textarea"
        },
        {
          "label": "Author",
          "key": "author",
          "type": "text"
        },
        {
          "label": "Tags",
          "key": "tags",
          "type": "text",
          "storage": "meta"
        },
        {
          "label": "Start Date",
          "key": "event_date",
          "type": "date",
          "storage": "meta"
        },
        {
          "label": "Location",
          "key": "location",
          "type": "text",
          "storage": "meta"
        },
        {
          "label": "Capacity",
          "key": "capacity",
          "type": "number",
          "storage": "meta"
        },
        {
          "label": "Registration Link",
          "key": "registration_link",
          "type": "text",
          "storage": "meta"
        },
        {
          "label": "Body Markdown",
          "key": "body_md",
          "type": "markdown"
        }
      ]
    },
    "post": {
      "label": "Post",
      "has_markdown": true,
      "fields": [
        {
          "label": "Title",
          "key": "title",
          "type": "text",
          "required": true
        },
        {
          "label": "Description",
          "key": "description",
          "type": "textarea"
        },
        {
          "label": "Author",
          "key": "author",
          "type": "text"
        },
        {
          "label": "Tags",
          "key": "tags",
          "type": "text",
          "storage": "meta"
        },
        {
          "label": "Date",
          "key": "date",
          "type": "date",
          "storage": "meta"
        },
        {
          "label": "Body Markdown",
          "key": "body_md",
          "type": "markdown",
          "required": true
        }
      ]
    }
  }
}
        "#;

        // Write to temp file
        fs::write(&path, json).expect("failed to write test schema");

        // Load schema
        let schema = load_frontend_schema(path.to_str().unwrap()).expect("failed to load schema");

        // Assertions
        assert!(schema.types.contains_key(&DocType::Recipe));
        let recipe = schema.types.get(&DocType::Recipe).unwrap();
        assert_eq!(recipe.label, DocType::Recipe.label());
        assert!(recipe.has_markdown);
        //assert_eq!(recipe.fields.len(), 2);
        assert_eq!(recipe.fields[0].key, "title");
        assert_eq!(recipe.fields[0].field_type, "text");
        assert_eq!(recipe.fields[1].key, "description");

        // Cleanup
        fs::remove_file(path).ok();
    }
}
