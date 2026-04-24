/// Parallax Schema Explorer
/// Crawls local project directories to infer API surfaces for any framework.
/// Supports: Frappe/ERPNext, Django, Laravel, Rails, OpenAPI/Swagger, FastAPI
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaField {
    pub name: String,
    pub field_type: String,
    pub required: bool,
    pub options: Option<Vec<String>>,
    pub description: Option<String>,
    pub default: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaEntity {
    /// DocType name, Model name, Resource name, etc.
    pub name: String,
    pub kind: EntityKind,
    pub fields: Vec<SchemaField>,
    pub endpoints: Vec<InferredEndpoint>,
    pub description: Option<String>,
    pub source_file: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityKind {
    FrappeDoctype,
    DjangoModel,
    LaravelModel,
    RailsModel,
    OpenApiSchema,
    FastApiModel,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferredEndpoint {
    pub method: String,
    pub path: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExplorerResult {
    pub framework: String,
    pub root_path: String,
    pub entities: Vec<SchemaEntity>,
    pub endpoints: Vec<InferredEndpoint>,
    pub raw_spec: Option<serde_json::Value>,
}

pub struct SchemaExplorer;

impl SchemaExplorer {
    /// Explore a directory and auto-detect the framework + schema
    pub fn explore(root: &str) -> Result<ExplorerResult> {
        let path = PathBuf::from(root);

        // OpenAPI / Swagger first (framework-agnostic)
        if let Some(spec) = find_openapi_spec(&path) {
            return parse_openapi(root, &spec);
        }

        // Frappe: look for doctypes folder
        if has_frappe_structure(&path) {
            return explore_frappe(root);
        }

        // Django: look for models.py files
        if has_django_structure(&path) {
            return explore_django(root);
        }

        // Laravel: look for app/Models directory
        if has_laravel_structure(&path) {
            return explore_laravel(root);
        }

        // Rails: look for db/schema.rb
        if has_rails_structure(&path) {
            return explore_rails(root);
        }

        // FastAPI: look for main.py with FastAPI() + Pydantic models
        if has_fastapi_structure(&path) {
            return explore_fastapi(root);
        }

        // Express: look for package.json with express + route files
        if has_express_structure(&path) {
            return explore_express(root);
        }

        // Fallback: generic
        Ok(ExplorerResult {
            framework: "unknown".to_string(),
            root_path: root.to_string(),
            entities: vec![],
            endpoints: vec![],
            raw_spec: None,
        })
    }
}

// =============================================================================
// FRAPPE / ERPNext Explorer
// Crawls: apps/*/doctype/**/*.json for DocType definitions
// =============================================================================
fn has_frappe_structure(path: &Path) -> bool {
    path.join("apps").exists()
        || path.join("sites").exists()
        || path.join("doctype").exists()
        || path.join("hooks.py").exists()
}

fn explore_frappe(root: &str) -> Result<ExplorerResult> {
    let path = PathBuf::from(root);
    let mut entities = Vec::new();

    // Walk the directory looking for DocType JSON files
    for entry in walkdir::WalkDir::new(&path)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let file_path = entry.path();
        if file_path.extension().and_then(|e| e.to_str()) == Some("json") {
            if let Some(parent) = file_path.parent() {
                if parent.file_name().and_then(|n| n.to_str()) == file_path.file_stem().and_then(|n| n.to_str()) {
                    // This is a DocType file (folder name matches file name)
                    if let Ok(entity) = parse_frappe_doctype(file_path) {
                        entities.push(entity);
                    }
                }
            }
        }
    }

    // Infer REST endpoints for each DocType
    let mut all_endpoints = Vec::new();
    for entity in &mut entities {
        let doctype = &entity.name;
        entity.endpoints = vec![
            InferredEndpoint { method: "GET".to_string(), path: format!("/api/resource/{}", doctype), description: Some("List all records".to_string()) },
            InferredEndpoint { method: "GET".to_string(), path: format!("/api/resource/{}/{{name}}", doctype), description: Some("Get single record".to_string()) },
            InferredEndpoint { method: "POST".to_string(), path: format!("/api/resource/{}", doctype), description: Some("Create record".to_string()) },
            InferredEndpoint { method: "PUT".to_string(), path: format!("/api/resource/{}/{{name}}", doctype), description: Some("Update record".to_string()) },
            InferredEndpoint { method: "DELETE".to_string(), path: format!("/api/resource/{}/{{name}}", doctype), description: Some("Delete record".to_string()) },
        ];
        all_endpoints.extend(entity.endpoints.clone());
    }

    Ok(ExplorerResult {
        framework: "frappe".to_string(),
        root_path: root.to_string(),
        entities,
        endpoints: all_endpoints,
        raw_spec: None,
    })
}

fn parse_frappe_doctype(path: &Path) -> Result<SchemaEntity> {
    let content = std::fs::read_to_string(path)?;
    let json: serde_json::Value = serde_json::from_str(&content)?;

    let name = json["name"].as_str().unwrap_or("Unknown").to_string();
    let description = json["description"].as_str().map(String::from);

    let fields = json["fields"].as_array()
        .map(|fields| fields.iter().map(|f| SchemaField {
            name: f["fieldname"].as_str().unwrap_or("").to_string(),
            field_type: f["fieldtype"].as_str().unwrap_or("Data").to_string(),
            required: f["reqd"].as_i64().unwrap_or(0) == 1,
            options: f["options"].as_str().map(|o| o.lines().map(String::from).collect()),
            description: f["description"].as_str().map(String::from),
            default: f.get("default").cloned(),
        }).collect())
        .unwrap_or_default();

    Ok(SchemaEntity {
        name,
        kind: EntityKind::FrappeDoctype,
        fields,
        endpoints: vec![],
        description,
        source_file: path.to_string_lossy().to_string(),
    })
}

// =============================================================================
// DJANGO Explorer
// Crawls: **/models.py → parses class Model(models.Model)
// =============================================================================
fn has_django_structure(path: &Path) -> bool {
    path.join("manage.py").exists()
        || walkdir::WalkDir::new(path).into_iter().any(|e| {
            e.ok().map(|e| e.file_name() == "models.py").unwrap_or(false)
        })
}

fn explore_django(root: &str) -> Result<ExplorerResult> {
    let path = PathBuf::from(root);
    let mut entities = Vec::new();

    for entry in walkdir::WalkDir::new(&path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name() == "models.py")
    {
        let content = std::fs::read_to_string(entry.path()).unwrap_or_default();
        entities.extend(parse_django_models(&content, &entry.path().to_string_lossy()));
    }

    Ok(ExplorerResult {
        framework: "django".to_string(),
        root_path: root.to_string(),
        entities,
        endpoints: vec![],
        raw_spec: None,
    })
}

fn parse_django_models(content: &str, source_file: &str) -> Vec<SchemaEntity> {
    let mut entities = Vec::new();
    let class_re = regex::Regex::new(r"class\s+(\w+)\s*\([^)]*models\.Model[^)]*\)").unwrap();
    let field_re = regex::Regex::new(r"\s+(\w+)\s*=\s*models\.(\w+)\(").unwrap();

    for class_match in class_re.find_iter(content) {
        let class_name = class_re.captures(class_match.as_str())
            .and_then(|c| c.get(1))
            .map(|m| m.as_str().to_string())
            .unwrap_or_default();

        // Get the class body (heuristic: lines after the class until next class or EOF)
        let start = class_match.end();
        let class_body = &content[start..];
        let next_class = class_re.find(class_body).map(|m| m.start()).unwrap_or(class_body.len());
        let body = &class_body[..next_class];

        let fields: Vec<SchemaField> = field_re.captures_iter(body)
            .map(|c| SchemaField {
                name: c.get(1).map(|m| m.as_str().to_string()).unwrap_or_default(),
                field_type: c.get(2).map(|m| m.as_str().to_string()).unwrap_or_default(),
                required: true,
                options: None,
                description: None,
                default: None,
            })
            .collect();

        // Infer DRF-style endpoints
        let endpoints = vec![
            InferredEndpoint { method: "GET".to_string(), path: format!("/api/{}/", class_name.to_lowercase()), description: Some("List".to_string()) },
            InferredEndpoint { method: "POST".to_string(), path: format!("/api/{}/", class_name.to_lowercase()), description: Some("Create".to_string()) },
            InferredEndpoint { method: "GET".to_string(), path: format!("/api/{}/:id/", class_name.to_lowercase()), description: Some("Retrieve".to_string()) },
            InferredEndpoint { method: "PUT".to_string(), path: format!("/api/{}/:id/", class_name.to_lowercase()), description: Some("Update".to_string()) },
            InferredEndpoint { method: "DELETE".to_string(), path: format!("/api/{}/:id/", class_name.to_lowercase()), description: Some("Destroy".to_string()) },
        ];

        entities.push(SchemaEntity {
            name: class_name,
            kind: EntityKind::DjangoModel,
            fields,
            endpoints,
            description: None,
            source_file: source_file.to_string(),
        });
    }

    entities
}

// =============================================================================
// LARAVEL Explorer
// Crawls: app/Models/*.php → parses $fillable, $casts, relationships
// =============================================================================
fn has_laravel_structure(path: &Path) -> bool {
    path.join("artisan").exists() || path.join("app/Models").exists()
}

fn explore_laravel(root: &str) -> Result<ExplorerResult> {
    let models_path = PathBuf::from(root).join("app/Models");
    let mut entities = Vec::new();

    if models_path.exists() {
        for entry in std::fs::read_dir(&models_path).unwrap().flatten() {
            if entry.path().extension().and_then(|e| e.to_str()) == Some("php") {
                let content = std::fs::read_to_string(entry.path()).unwrap_or_default();
                let name = entry.path().file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Unknown")
                    .to_string();

                let fillable_re = regex::Regex::new(r"\\\$fillable\s*=\s*\[([^\]]+)\]").unwrap();
                let fields: Vec<SchemaField> = fillable_re.captures(&content)
                    .and_then(|c| c.get(1))
                    .map(|m| {
                        m.as_str()
                            .split(',')
                            .map(|s| s.trim().trim_matches(|c| c == '\'' || c == '"').to_string())
                            .filter(|s| !s.is_empty())
                            .map(|field_name| SchemaField {
                                name: field_name,
                                field_type: "string".to_string(),
                                required: true,
                                options: None,
                                description: None,
                                default: None,
                            })
                            .collect()
                    })
                    .unwrap_or_default();

                let model_name_lower = name.to_lowercase();
                let endpoints = vec![
                    InferredEndpoint { method: "GET".to_string(), path: format!("/api/{}s", model_name_lower), description: None },
                    InferredEndpoint { method: "POST".to_string(), path: format!("/api/{}s", model_name_lower), description: None },
                    InferredEndpoint { method: "GET".to_string(), path: format!("/api/{}s/{{id}}", model_name_lower), description: None },
                    InferredEndpoint { method: "PUT".to_string(), path: format!("/api/{}s/{{id}}", model_name_lower), description: None },
                    InferredEndpoint { method: "DELETE".to_string(), path: format!("/api/{}s/{{id}}", model_name_lower), description: None },
                ];

                entities.push(SchemaEntity {
                    name,
                    kind: EntityKind::LaravelModel,
                    fields,
                    endpoints,
                    description: None,
                    source_file: entry.path().to_string_lossy().to_string(),
                });
            }
        }
    }

    Ok(ExplorerResult {
        framework: "laravel".to_string(),
        root_path: root.to_string(),
        entities,
        endpoints: vec![],
        raw_spec: None,
    })
}

// =============================================================================
// RAILS Explorer
// Crawls: db/schema.rb for table definitions
// =============================================================================
fn has_rails_structure(path: &Path) -> bool {
    path.join("Gemfile").exists() || path.join("db/schema.rb").exists()
}

fn explore_rails(root: &str) -> Result<ExplorerResult> {
    let schema_path = PathBuf::from(root).join("db/schema.rb");
    let mut entities = Vec::new();

    if schema_path.exists() {
        let content = std::fs::read_to_string(&schema_path).unwrap_or_default();
        let table_re = regex::Regex::new(r#"create_table\s+"(\w+)""#).unwrap();
        let col_re = regex::Regex::new(r#"t\.(\w+)\s+"(\w+)""#).unwrap();

        for table_match in table_re.captures_iter(&content) {
            let table_name = table_match.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
            let model_name = to_pascal_case(&table_name);

            let start = table_match.get(0).unwrap().end();
            let rest = &content[start..];
            let end = rest.find("end").unwrap_or(rest.len());
            let body = &rest[..end];

            let fields: Vec<SchemaField> = col_re.captures_iter(body)
                .map(|c| SchemaField {
                    name: c.get(2).map(|m| m.as_str().to_string()).unwrap_or_default(),
                    field_type: c.get(1).map(|m| m.as_str().to_string()).unwrap_or_default(),
                    required: false,
                    options: None,
                    description: None,
                    default: None,
                })
                .collect();

            entities.push(SchemaEntity {
                name: model_name.clone(),
                kind: EntityKind::RailsModel,
                fields,
                endpoints: vec![
                    InferredEndpoint { method: "GET".to_string(), path: format!("/api/v1/{}", table_name), description: None },
                    InferredEndpoint { method: "POST".to_string(), path: format!("/api/v1/{}", table_name), description: None },
                    InferredEndpoint { method: "GET".to_string(), path: format!("/api/v1/{}/:id", table_name), description: None },
                    InferredEndpoint { method: "PATCH".to_string(), path: format!("/api/v1/{}/:id", table_name), description: None },
                    InferredEndpoint { method: "DELETE".to_string(), path: format!("/api/v1/{}/:id", table_name), description: None },
                ],
                description: None,
                source_file: schema_path.to_string_lossy().to_string(),
            });
        }
    }

    Ok(ExplorerResult {
        framework: "rails".to_string(),
        root_path: root.to_string(),
        entities,
        endpoints: vec![],
        raw_spec: None,
    })
}

// =============================================================================
// OPENAPI / SWAGGER (Framework-agnostic — highest priority)
// Finds: openapi.yaml, swagger.yaml, openapi.json, swagger.json
// =============================================================================
fn find_openapi_spec(root: &Path) -> Option<PathBuf> {
    let candidates = [
        "openapi.yaml", "openapi.yml", "openapi.json",
        "swagger.yaml", "swagger.yml", "swagger.json",
        "api.yaml", "api.json",
        "docs/openapi.yaml", "docs/swagger.yaml",
        "api-docs/openapi.yaml",
    ];

    for candidate in &candidates {
        let p = root.join(candidate);
        if p.exists() {
            return Some(p);
        }
    }
    None
}

fn parse_openapi(root: &str, spec_path: &Path) -> Result<ExplorerResult> {
    let content = std::fs::read_to_string(spec_path)?;
    let spec: serde_json::Value = if spec_path.extension().and_then(|e| e.to_str()) == Some("json") {
        serde_json::from_str(&content)?
    } else {
        serde_yaml::from_str(&content)?
    };

    let mut entities = Vec::new();
    let mut endpoints = Vec::new();

    // Parse schemas
    if let Some(schemas) = spec.pointer("/components/schemas").or_else(|| spec.pointer("/definitions")) {
        if let Some(schemas) = schemas.as_object() {
            for (name, schema) in schemas {
                let fields = schema.get("properties")
                    .and_then(|p| p.as_object())
                    .map(|props| {
                        let required_fields: Vec<String> = schema.get("required")
                            .and_then(|r| r.as_array())
                            .map(|r| r.iter().filter_map(|v| v.as_str()).map(String::from).collect())
                            .unwrap_or_default();
                        props.iter().map(|(field_name, field_def)| SchemaField {
                            name: field_name.clone(),
                            field_type: field_def.get("type").and_then(|t| t.as_str()).unwrap_or("string").to_string(),
                            required: required_fields.contains(field_name),
                            options: field_def.get("enum").and_then(|e| e.as_array()).map(|e| e.iter().filter_map(|v| v.as_str()).map(String::from).collect()),
                            description: field_def.get("description").and_then(|d| d.as_str()).map(String::from),
                            default: field_def.get("default").cloned(),
                        }).collect()
                    })
                    .unwrap_or_default();

                entities.push(SchemaEntity {
                    name: name.clone(),
                    kind: EntityKind::OpenApiSchema,
                    fields,
                    endpoints: vec![],
                    description: schema.get("description").and_then(|d| d.as_str()).map(String::from),
                    source_file: spec_path.to_string_lossy().to_string(),
                });
            }
        }
    }

    // Parse paths
    if let Some(paths) = spec.get("paths").and_then(|p| p.as_object()) {
        for (path, methods) in paths {
            if let Some(methods) = methods.as_object() {
                for (method, op) in methods {
                    let description = op.get("summary")
                        .or_else(|| op.get("description"))
                        .and_then(|d| d.as_str())
                        .map(String::from);
                    endpoints.push(InferredEndpoint {
                        method: method.to_uppercase(),
                        path: path.clone(),
                        description,
                    });
                }
            }
        }
    }

    Ok(ExplorerResult {
        framework: "openapi".to_string(),
        root_path: root.to_string(),
        entities,
        endpoints,
        raw_spec: Some(spec),
    })
}

// =============================================================================
// FASTAPI Explorer
// Crawls: **/*.py for class Model(BaseModel) Pydantic definitions
// and @app.get/post/put/delete/patch decorators for routes
// =============================================================================
fn has_fastapi_structure(path: &Path) -> bool {
    walkdir::WalkDir::new(path).into_iter().any(|e| {
        e.ok().map(|e| {
            if e.path().extension().and_then(|x| x.to_str()) != Some("py") { return false; }
            let content = std::fs::read_to_string(e.path()).unwrap_or_default();
            content.contains("FastAPI()") || content.contains("from fastapi")
        }).unwrap_or(false)
    })
}

fn explore_fastapi(root: &str) -> Result<ExplorerResult> {
    let path = PathBuf::from(root);
    let mut entities = Vec::new();
    let mut endpoints = Vec::new();

    let class_re = regex::Regex::new(r"class\s+(\w+)\s*\([^)]*BaseModel[^)]*\)").unwrap();
    let field_re = regex::Regex::new(r"\s{4}(\w+)\s*:\s*([\w\[\], |]+)").unwrap();
    let route_re = regex::Regex::new(r#"@(?:app|router)\.(get|post|put|patch|delete)\s*\(\s*["']([^"']+)["']"#).unwrap();

    for entry in walkdir::WalkDir::new(&path).into_iter().filter_map(|e| e.ok()) {
        let p = entry.path();
        if p.extension().and_then(|x| x.to_str()) != Some("py") { continue; }
        let content = std::fs::read_to_string(p).unwrap_or_default();

        for class_match in class_re.captures_iter(&content) {
            let class_name = class_match.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
            let start = class_match.get(0).unwrap().end();
            let rest = &content[start..];
            let end = class_re.find(rest).map(|m| m.start()).unwrap_or(rest.len().min(800));
            let body = &rest[..end];

            let fields: Vec<SchemaField> = field_re.captures_iter(body)
                .map(|c| SchemaField {
                    name: c.get(1).map(|m| m.as_str().to_string()).unwrap_or_default(),
                    field_type: c.get(2).map(|m| m.as_str().trim().to_string()).unwrap_or_default(),
                    required: true,
                    options: None, description: None, default: None,
                })
                .collect();

            entities.push(SchemaEntity {
                name: class_name,
                kind: EntityKind::FastApiModel,
                fields,
                endpoints: vec![],
                description: None,
                source_file: p.to_string_lossy().to_string(),
            });
        }

        for cap in route_re.captures_iter(&content) {
            endpoints.push(InferredEndpoint {
                method: cap.get(1).map(|m| m.as_str().to_uppercase()).unwrap_or_default(),
                path: cap.get(2).map(|m| m.as_str().to_string()).unwrap_or_default(),
                description: None,
            });
        }
    }

    Ok(ExplorerResult {
        framework: "fastapi".to_string(),
        root_path: root.to_string(),
        entities,
        endpoints,
        raw_spec: None,
    })
}

// =============================================================================
// EXPRESS Explorer
// Crawls: **/*.js|*.ts for router.get/post/put/delete/patch route registrations
// =============================================================================
fn has_express_structure(path: &Path) -> bool {
    let pkg = path.join("package.json");
    if pkg.exists() {
        let content = std::fs::read_to_string(pkg).unwrap_or_default();
        if content.contains("\"express\"") { return true; }
    }
    false
}

fn explore_express(root: &str) -> Result<ExplorerResult> {
    let path = PathBuf::from(root);
    let mut endpoints = Vec::new();

    let route_re = regex::Regex::new(
        r#"(?:router|app)\.(get|post|put|patch|delete)\s*\(\s*["'`]([^"'`]+)["'`]"#
    ).unwrap();

    for entry in walkdir::WalkDir::new(&path).into_iter().filter_map(|e| e.ok()) {
        let p = entry.path();
        let ext = p.extension().and_then(|x| x.to_str()).unwrap_or("");
        if ext != "js" && ext != "ts" && ext != "mjs" { continue; }
        // Skip node_modules
        if p.to_string_lossy().contains("node_modules") { continue; }
        let content = std::fs::read_to_string(p).unwrap_or_default();

        for cap in route_re.captures_iter(&content) {
            endpoints.push(InferredEndpoint {
                method: cap.get(1).map(|m| m.as_str().to_uppercase()).unwrap_or_default(),
                path: cap.get(2).map(|m| m.as_str().to_string()).unwrap_or_default(),
                description: None,
            });
        }
    }

    Ok(ExplorerResult {
        framework: "express".to_string(),
        root_path: root.to_string(),
        entities: vec![],
        endpoints,
        raw_spec: None,
    })
}

// ---------------------------------------------------------------------------
// Utilities
// ---------------------------------------------------------------------------
fn to_pascal_case(s: &str) -> String {
    s.split('_')
        .map(|word| {
            let mut c = word.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect()
}
