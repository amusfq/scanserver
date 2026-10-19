use std::{
    env,
    fs::File,
    io::{self, Read, Write},
    path::{Path, PathBuf},
    process::Output,
    sync::atomic::{AtomicU64, Ordering},
    time::UNIX_EPOCH,
};

use axum::{
    Json,
    body::Body,
    extract::{Multipart, Path as AxumPath, RawQuery, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use chrono::{DateTime, Local, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use zip::{CompressionMethod, ZipWriter, write::SimpleFileOptions};

use crate::{AppState, printer, scanner};

const DEFAULT_PREVIEW: &[u8] = include_bytes!("../../app-server/data/preview/default.jpg");
const MAX_PRINT_FILE_SIZE: u64 = 80 * 1024 * 1024;
static PRINT_UPLOAD_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Debug)]
pub struct ApiError {
    status: StatusCode,
    message: String,
}

impl ApiError {
    fn bad_request(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message: message.into(),
        }
    }

    fn not_found(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            message: message.into(),
        }
    }

    fn server(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: message.into(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(json!({"message": self.message, "code": -1})),
        )
            .into_response()
    }
}

impl From<io::Error> for ApiError {
    fn from(error: io::Error) -> Self {
        Self::server(error.to_string())
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FileInfo {
    name: String,
    path: String,
    extension: String,
    last_modified: String,
    size: u64,
    size_string: String,
    is_directory: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    deleted: Option<bool>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ScanRequest {
    #[serde(default)]
    params: ScanParams,
    #[serde(default)]
    filters: Vec<String>,
    pipeline: Option<String>,
    batch: Option<String>,
    index: Option<i32>,
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ScanParams {
    device_id: Option<String>,
    resolution: Option<f64>,
    mode: Option<String>,
    source: Option<String>,
    adf_mode: Option<String>,
    left: Option<f64>,
    top: Option<f64>,
    width: Option<f64>,
    height: Option<f64>,
    page_width: Option<f64>,
    page_height: Option<f64>,
    brightness: Option<f64>,
    contrast: Option<f64>,
    dynamic_lineart: Option<bool>,
    ald: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RenameInput {
    new_name: String,
}

pub async fn read_context(
    State(state): State<std::sync::Arc<AppState>>,
) -> Result<Json<Value>, ApiError> {
    let devices = device_list(&state).await?;
    let diagnostics = [state.scanimage.as_path(), state.convert.as_path()].map(|path| {
        let success = path.is_file();
        json!({
            "success": success,
            "message": if success { format!("Found {}", path.display()) } else { format!("Unable to find file {}", path.display()) }
        })
    });

    Ok(Json(json!({
        "devices": devices,
        "version": env!("CARGO_PKG_VERSION"),
        "diagnostics": diagnostics,
        "paperSizes": paper_sizes(),
        "actions": [],
    })))
}

pub async fn delete_context(State(state): State<std::sync::Arc<AppState>>) -> Json<Value> {
    *state.devices.write().await = None;
    let _ = tokio::fs::remove_file(state.preview_dir().join("preview.tif")).await;
    Json(json!({}))
}

async fn device_list(state: &AppState) -> Result<Vec<scanner::Device>, ApiError> {
    if let Some(devices) = state.devices.read().await.as_ref() {
        return Ok(devices.clone());
    }

    let mut ids = env::var("DEVICES")
        .unwrap_or_default()
        .split(
            env::var("DELIMITER")
                .unwrap_or_else(|_| ";".to_owned())
                .as_str(),
        )
        .filter(|id| !id.is_empty())
        .map(str::to_owned)
        .collect::<Vec<_>>();
    if env::var("SCANIMAGE_LIST_IGNORE")
        .unwrap_or_default()
        .is_empty()
    {
        let output = run(&state.scanimage, &["-L".to_owned()]).await?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        for id in scanner::parse_device_ids(&stdout) {
            if !ids.contains(&id) {
                ids.push(id);
            }
        }
    }

    let pipelines = scanner::pipeline_options();
    let mut devices = Vec::new();
    for id in ids {
        match run(
            &state.scanimage,
            &["-d".to_owned(), id.clone(), "-A".to_owned()],
        )
        .await
        {
            Ok(output) => {
                let text = String::from_utf8_lossy(&output.stdout);
                if let Ok(device) = scanner::parse_device(&id, &text, &pipelines) {
                    devices.push(device);
                }
            }
            Err(error) => eprintln!("ignoring scanner {id}: {}", error.message),
        }
    }

    *state.devices.write().await = Some(devices.clone());
    Ok(devices)
}

fn paper_sizes() -> Vec<Value> {
    [
        ("A3 (@:paper-size.portrait)", 297.0, 420.0),
        ("A4 (@:paper-size.portrait)", 210.0, 297.0),
        ("A5 (@:paper-size.portrait)", 148.0, 210.0),
        ("A5 (@:paper-size.landscape)", 210.0, 148.0),
        ("A6 (@:paper-size.portrait)", 105.0, 148.0),
        ("A6 (@:paper-size.landscape)", 148.0, 105.0),
        ("B5 (@:paper-size.portrait)", 176.0, 250.0),
        ("@:paper-size.letter (@:paper-size.portrait)", 216.0, 279.0),
        ("@:paper-size.legal (@:paper-size.portrait)", 216.0, 356.0),
        ("@:paper-size.tabloid (@:paper-size.portrait)", 279.0, 432.0),
    ]
    .into_iter()
    .map(|(name, x, y)| json!({"name": name, "dimensions": {"x": x, "y": y}}))
    .collect()
}

pub async fn file_list(
    State(state): State<std::sync::Arc<AppState>>,
) -> Result<Json<Vec<FileInfo>>, ApiError> {
    let mut entries = tokio::fs::read_dir(state.output_dir())
        .await
        .map_err(|error| ApiError::server(error.to_string()))?;
    let mut files = Vec::new();
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        let metadata = entry.metadata().await?;
        if metadata.is_file() {
            files.push(file_info(&path, metadata, None));
        }
    }
    files.sort_by(|a, b| b.last_modified.cmp(&a.last_modified));
    Ok(Json(files))
}

pub async fn print_capabilities(
    State(state): State<std::sync::Arc<AppState>>,
) -> Result<Json<Value>, ApiError> {
    let capabilities = state
        .printer
        .capabilities()
        .await
        .map_err(ApiError::server)?;
    serde_json::to_value(capabilities)
        .map(Json)
        .map_err(|error| ApiError::server(error.to_string()))
}

pub async fn print_file(
    State(state): State<std::sync::Arc<AppState>>,
    AxumPath(filename): AxumPath<String>,
    Json(settings): Json<printer::Settings>,
) -> Result<Json<Value>, ApiError> {
    let path = safe_path(&state.output_dir(), &filename)?;
    validate_print_path(&path).await?;
    let job = state
        .printer
        .submit(&path.to_string_lossy(), &settings)
        .await
        .map_err(ApiError::bad_request)?;
    Ok(Json(json!({"message": job})))
}

pub async fn print_upload(
    State(state): State<std::sync::Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<Json<Value>, ApiError> {
    let mut file: Option<(String, Vec<u8>)> = None;
    let mut settings: Option<printer::Settings> = None;
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|error| ApiError::bad_request(error.to_string()))?
    {
        match field.name() {
            Some("file") if file.is_none() => {
                let filename = field
                    .file_name()
                    .map(str::to_owned)
                    .ok_or_else(|| ApiError::bad_request("Missing upload filename"))?;
                let bytes = field
                    .bytes()
                    .await
                    .map_err(|error| ApiError::bad_request(error.to_string()))?;
                if bytes.is_empty() || bytes.len() as u64 > MAX_PRINT_FILE_SIZE {
                    return Err(ApiError::bad_request(
                        "File must be between 1 byte and 80 MB",
                    ));
                }
                file = Some((filename, bytes.to_vec()));
            }
            Some("settings") if settings.is_none() => {
                let text = field
                    .text()
                    .await
                    .map_err(|error| ApiError::bad_request(error.to_string()))?;
                settings = Some(
                    serde_json::from_str(&text)
                        .map_err(|error| ApiError::bad_request(error.to_string()))?,
                );
            }
            _ => {
                return Err(ApiError::bad_request(
                    "Unexpected or duplicate upload field",
                ));
            }
        }
    }

    let (filename, bytes) = file.ok_or_else(|| ApiError::bad_request("Missing file"))?;
    let settings = settings.unwrap_or_default();
    let extension = printable_extension(&filename)?;
    validate_print_content(extension, &bytes)?;
    let path = store_print_upload(&state.temp_dir(), extension, &bytes).await?;
    let result = state
        .printer
        .submit(&path.to_string_lossy(), &settings)
        .await;
    let _ = tokio::fs::remove_file(path).await;
    let job = result.map_err(ApiError::bad_request)?;
    Ok(Json(json!({"message": job})))
}

async fn validate_print_path(path: &Path) -> Result<(), ApiError> {
    let metadata = tokio::fs::metadata(path)
        .await
        .map_err(|_| ApiError::not_found("File not found"))?;
    if !metadata.is_file() {
        return Err(ApiError::bad_request(
            "Only PDF and image files can be printed",
        ));
    }
    if metadata.len() == 0 || metadata.len() > MAX_PRINT_FILE_SIZE {
        return Err(ApiError::bad_request(
            "File must be between 1 byte and 80 MB",
        ));
    }
    let extension = printable_extension(
        path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default(),
    )?;
    let bytes = tokio::fs::read(path).await?;
    validate_print_content(extension, &bytes)
}

fn printable_extension(filename: &str) -> Result<&str, ApiError> {
    match Path::new(filename)
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "pdf" => Ok("pdf"),
        "jpg" | "jpeg" => Ok("jpg"),
        "png" => Ok("png"),
        "tif" | "tiff" => Ok("tif"),
        _ => Err(ApiError::bad_request(
            "Only PDF, JPEG, PNG, and TIFF files can be printed",
        )),
    }
}

fn validate_print_content(extension: &str, bytes: &[u8]) -> Result<(), ApiError> {
    let valid = match extension {
        "pdf" => bytes
            .get(..bytes.len().min(1024))
            .is_some_and(|head| head.windows(5).any(|window| window == b"%PDF-")),
        "jpg" => bytes.starts_with(&[0xff, 0xd8, 0xff]),
        "png" => bytes.starts_with(b"\x89PNG\r\n\x1a\n"),
        "tif" => bytes.starts_with(b"II*\0") || bytes.starts_with(b"MM\0*"),
        _ => false,
    };
    if valid {
        Ok(())
    } else {
        Err(ApiError::bad_request(
            "File contents do not match its file type",
        ))
    }
}

async fn store_print_upload(
    directory: &Path,
    extension: &str,
    bytes: &[u8],
) -> Result<PathBuf, ApiError> {
    loop {
        let id = PRINT_UPLOAD_ID.fetch_add(1, Ordering::Relaxed);
        let path = directory.join(format!(
            "print-{}-{id}.{extension}",
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default()
        ));
        match tokio::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .await
        {
            Ok(mut file) => {
                if let Err(error) = file.write_all(bytes).await {
                    let _ = tokio::fs::remove_file(&path).await;
                    return Err(ApiError::from(error));
                }
                return Ok(path);
            }
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(ApiError::from(error)),
        }
    }
}

pub async fn download(
    State(state): State<std::sync::Arc<AppState>>,
    AxumPath(filename): AxumPath<String>,
) -> Result<Response, ApiError> {
    let path = safe_path(&state.output_dir(), &filename)?;
    let bytes = tokio::fs::read(&path)
        .await
        .map_err(|_| ApiError::not_found("File not found"))?;
    let mime = mime_guess::from_path(&path).first_or_octet_stream();
    let safe_ascii = filename
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || " ._-".contains(c) {
                c
            } else {
                '_'
            }
        })
        .collect::<String>();
    let disposition = format!(
        "attachment; filename=\"{safe_ascii}\"; filename*=UTF-8''{}",
        urlencoding::encode(&filename)
    );
    let mut response = Response::new(Body::from(bytes));
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(mime.as_ref())
            .unwrap_or(HeaderValue::from_static("application/octet-stream")),
    );
    response.headers_mut().insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&disposition).map_err(|error| ApiError::server(error.to_string()))?,
    );
    Ok(response)
}

pub async fn delete_file(
    State(state): State<std::sync::Arc<AppState>>,
    AxumPath(filename): AxumPath<String>,
) -> Result<Json<FileInfo>, ApiError> {
    let path = safe_path(&state.output_dir(), &filename)?;
    let metadata = tokio::fs::metadata(&path)
        .await
        .map_err(|_| ApiError::not_found("File not found"))?;
    let info = file_info(&path, metadata, Some(true));
    tokio::fs::remove_file(&path).await?;
    let _ = tokio::fs::remove_file(safe_path(&state.thumbnail_dir(), &filename)?).await;
    Ok(Json(info))
}

pub async fn rename_file(
    State(state): State<std::sync::Arc<AppState>>,
    AxumPath(filename): AxumPath<String>,
    Json(input): Json<RenameInput>,
) -> Result<Json<Value>, ApiError> {
    validate_filename(&input.new_name)?;
    let source = safe_path(&state.output_dir(), &filename)?;
    let destination = safe_path(&state.output_dir(), &input.new_name)?;
    if tokio::fs::try_exists(&destination).await? {
        return Err(ApiError::bad_request(
            "A file with that name already exists",
        ));
    }
    tokio::fs::rename(&source, &destination)
        .await
        .map_err(|_| ApiError::not_found("File not found"))?;
    let thumbnail = safe_path(&state.thumbnail_dir(), &filename)?;
    if tokio::fs::try_exists(&thumbnail).await? {
        let new_thumbnail = safe_path(&state.thumbnail_dir(), &input.new_name)?;
        tokio::fs::rename(thumbnail, new_thumbnail).await?;
    }
    Ok(Json(json!("200")))
}

pub async fn thumbnail(
    State(state): State<std::sync::Arc<AppState>>,
    AxumPath(filename): AxumPath<String>,
) -> Result<Response, ApiError> {
    let source = safe_path(&state.output_dir(), &filename)?;
    if filename.to_lowercase().ends_with(".zip") {
        return Ok(([(header::CONTENT_TYPE, "image/jpeg")], Body::empty()).into_response());
    }
    let cached = safe_path(&state.thumbnail_dir(), &filename)?;
    let bytes = if tokio::fs::try_exists(&cached).await? {
        tokio::fs::read(cached).await?
    } else {
        if !tokio::fs::try_exists(&source).await? {
            return Err(ApiError::not_found("File not found"));
        }
        let input = format!("{}[0]", source.display());
        let output = run(
            &state.convert,
            &[
                input,
                "-resize".to_owned(),
                "256".to_owned(),
                "-quality".to_owned(),
                "75".to_owned(),
                "jpg:-".to_owned(),
            ],
        )
        .await?;
        tokio::fs::write(&cached, &output.stdout).await?;
        output.stdout
    };
    Ok(([(header::CONTENT_TYPE, "image/jpeg")], Body::from(bytes)).into_response())
}

pub async fn file_action(
    AxumPath((_filename, action)): AxumPath<(String, String)>,
    _headers: HeaderMap,
) -> Result<Json<Value>, ApiError> {
    Err(ApiError::not_found(format!(
        "No matching file action: {action}"
    )))
}

pub async fn read_preview(
    State(state): State<std::sync::Arc<AppState>>,
    RawQuery(query): RawQuery,
) -> Result<Json<Value>, ApiError> {
    let filters = form_urlencoded::parse(query.as_deref().unwrap_or_default().as_bytes())
        .filter(|(key, _)| key == "filter")
        .map(|(_, value)| value.into_owned())
        .collect::<Vec<_>>();
    let path = state.preview_dir().join("preview.tif");
    if !tokio::fs::try_exists(&path).await? {
        return Ok(Json(json!({"content": BASE64.encode(DEFAULT_PREVIEW)})));
    }
    let mut args = vec![path.display().to_string()];
    args.extend(filter_args(&filters)?);
    args.extend(["-quality".to_owned(), "75".to_owned(), "jpg:-".to_owned()]);
    let image = run(&state.convert, &args).await?;
    Ok(Json(json!({"content": BASE64.encode(image.stdout)})))
}

pub async fn create_preview(
    State(state): State<std::sync::Arc<AppState>>,
    Json(request): Json<ScanRequest>,
) -> Result<Json<Value>, ApiError> {
    let _guard = state.scan_lock.lock().await;
    let devices = device_list(&state).await?;
    let device = selected_device(&devices, request.params.device_id.as_deref())?;
    let path = state.preview_dir().join("preview.tmp.tif");
    let args = scan_args(
        &state,
        &request.params,
        device,
        Some(path.clone()),
        Some(100.0),
        None,
    )?;
    run(&state.scanimage, &args).await?;
    tokio::fs::rename(path, state.preview_dir().join("preview.tif")).await?;
    Ok(Json(json!({})))
}

pub async fn delete_preview(
    State(state): State<std::sync::Arc<AppState>>,
) -> Result<Json<Value>, ApiError> {
    let path = state.preview_dir().join("preview.tif");
    let _ = tokio::fs::remove_file(path).await;
    Ok(Json(json!({})))
}

pub async fn scan(
    State(state): State<std::sync::Arc<AppState>>,
    Json(request): Json<ScanRequest>,
) -> Result<Json<Value>, ApiError> {
    let _guard = state.scan_lock.lock().await;
    let devices = device_list(&state).await?;
    let device = selected_device(&devices, request.params.device_id.as_deref())?;
    let batch = request.batch.as_deref().unwrap_or("none");
    let index = request.index.unwrap_or(1);
    let pipeline = request
        .pipeline
        .as_deref()
        .unwrap_or(device.settings.pipeline.default.as_str().unwrap_or(""));
    if !device
        .settings
        .pipeline
        .options
        .contains(&pipeline.to_owned())
    {
        return Err(ApiError::bad_request("Unknown output format"));
    }
    if !device
        .settings
        .batch_mode
        .options
        .iter()
        .any(|item| item == batch)
    {
        return Err(ApiError::bad_request("Unknown batch mode"));
    }
    let filters = &request.filters;
    filter_args(filters)?;

    let first_pass = index == 1;
    let perform_scan = index > 0;
    let finish_up = ["none", "auto"].contains(&batch)
        || (batch == "manual" && index < 0)
        || (matches!(batch, "auto-collate-standard" | "auto-collate-reverse") && index == 2);

    if first_pass {
        clear_scan_files(&state.temp_dir()).await?;
    }
    if perform_scan {
        let output = if matches!(
            batch,
            "auto" | "auto-collate-standard" | "auto-collate-reverse"
        ) {
            None
        } else {
            Some(state.temp_dir().join(format!("~tmp-scan-0-{index:04}.tif")))
        };
        let pattern = if output.is_none() {
            Some(state.temp_dir().join(format!("~tmp-scan-{index}-%04d.tif")))
        } else {
            None
        };
        let args = scan_args(&state, &request.params, device, output, None, pattern)?;
        run(&state.scanimage, &args).await?;
    }

    if !finish_up {
        let mut response = json!({"index": index});
        if batch == "manual" {
            let page = state.temp_dir().join(format!("~tmp-scan-0-{index:04}.tif"));
            let converted = run(
                &state.convert,
                &[
                    page.display().to_string(),
                    "-quality".to_owned(),
                    "75".to_owned(),
                    "jpg:-".to_owned(),
                ],
            )
            .await?;
            response["image"] = Value::String(BASE64.encode(converted.stdout));
        }
        return Ok(Json(response));
    }

    let mut pages = scan_files(&state.temp_dir()).await?;
    if batch == "auto-collate-standard" || batch == "auto-collate-reverse" {
        pages = collate(pages, batch == "auto-collate-standard");
    }
    if pages.is_empty() {
        return Err(ApiError::server("The scanner did not create an image"));
    }
    update_preview(&state, device, &request.params, &pages[0]).await?;

    if !filters.is_empty() {
        let args = filter_args(filters)?;
        let mut filtered = Vec::with_capacity(pages.len());
        for (index, page) in pages.iter().enumerate() {
            let destination = state.temp_dir().join(format!("filtered-{index:04}.tif"));
            let mut command_args = vec![page.display().to_string()];
            command_args.extend(args.clone());
            command_args.push(destination.display().to_string());
            run(&state.convert, &command_args).await?;
            filtered.push(destination);
        }
        pages = filtered;
    }

    let output = finish_output(&state, pages, pipeline).await?;
    let metadata = tokio::fs::metadata(&output).await?;
    let info = file_info(&output, metadata, None);
    clear_scan_files(&state.temp_dir()).await?;
    Ok(Json(json!({"file": info})))
}

fn selected_device<'a>(
    devices: &'a [scanner::Device],
    id: Option<&str>,
) -> Result<&'a scanner::Device, ApiError> {
    if let Some(id) = id {
        devices
            .iter()
            .find(|device| device.id == id)
            .ok_or_else(|| ApiError::bad_request("Scanner not found"))
    } else {
        devices
            .first()
            .ok_or_else(|| ApiError::server("No scanners found"))
    }
}

fn scan_args(
    state: &AppState,
    params: &ScanParams,
    device: &scanner::Device,
    output: Option<PathBuf>,
    resolution_override: Option<f64>,
    batch_pattern: Option<PathBuf>,
) -> Result<Vec<String>, ApiError> {
    let mut args = vec!["-d".to_owned(), device.id.clone()];
    for (name, value) in [
        ("--source", params.source.as_ref()),
        ("--mode", params.mode.as_ref()),
        ("--adf-mode", params.adf_mode.as_ref()),
        ("--ald", params.ald.as_ref()),
    ] {
        if let Some(value) = value {
            validate_choice(device, name, value)?;
            args.push(name.to_owned());
            args.push(value.clone());
        }
    }
    let resolution = resolution_override
        .or(params.resolution)
        .or_else(|| {
            device
                .features
                .get("--resolution")
                .and_then(|f| f.default.as_f64())
        })
        .unwrap_or(300.0);
    if let Some(options) = device
        .features
        .get("--resolution")
        .and_then(|f| f.options.as_ref())
    {
        if !options.iter().any(|option| {
            option
                .as_f64()
                .is_some_and(|v| (v - resolution).abs() < f64::EPSILON)
        }) {
            if !device
                .features
                .get("--resolution")
                .and_then(|f| f.limits.as_ref())
                .is_some_and(|limits| resolution >= limits[0] && resolution <= limits[1])
            {
                return Err(ApiError::bad_request("Unsupported scan resolution"));
            }
        }
    }
    args.extend(["--resolution".to_owned(), number(resolution)]);

    for (feature, value) in [
        ("--page-width", params.page_width),
        ("--page-height", params.page_height),
        ("-l", params.left),
        ("-t", params.top),
        ("-x", params.width),
        ("-y", params.height),
        ("--brightness", params.brightness),
        ("--contrast", params.contrast),
    ] {
        if let (Some(value), Some(option)) = (value, device.features.get(feature)) {
            let value = clamp(value, option.limits.as_deref());
            args.extend([feature.to_owned(), number(value)]);
        }
    }
    if params.mode.as_deref() == Some("Lineart")
        && params.dynamic_lineart == Some(false)
        && device.features.contains_key("--disable-dynamic-lineart")
    {
        args.push("--disable-dynamic-lineart=yes".to_owned());
    }
    args.extend(["--format".to_owned(), "tiff".to_owned()]);
    if let Some(pattern) = batch_pattern {
        args.push(format!("--batch={}", pattern.display()));
    } else if let Some(output) = output {
        args.extend(["-o".to_owned(), output.display().to_string()]);
    }
    let _ = state;
    Ok(args)
}

fn validate_choice(
    device: &scanner::Device,
    feature_name: &str,
    value: &str,
) -> Result<(), ApiError> {
    let feature = device
        .features
        .get(feature_name)
        .ok_or_else(|| ApiError::bad_request(format!("Scanner does not support {feature_name}")))?;
    let options = feature.options.as_ref().ok_or_else(|| {
        ApiError::bad_request(format!("Scanner has no options for {feature_name}"))
    })?;
    if !options.iter().any(|option| option.as_str() == Some(value)) {
        return Err(ApiError::bad_request(format!(
            "Unsupported {feature_name} value"
        )));
    }
    Ok(())
}

fn number(value: f64) -> String {
    format!("{value}")
}

fn clamp(value: f64, limits: Option<&[f64]>) -> f64 {
    if let Some([minimum, maximum]) = limits {
        value.clamp(*minimum, *maximum)
    } else {
        value
    }
}

fn filter_args(filters: &[String]) -> Result<Vec<String>, ApiError> {
    let mut args = Vec::new();
    for filter in filters {
        match filter.as_str() {
            "filter.auto-contrast" => args.push("-equalize".to_owned()),
            "filter.auto-level" => args.push("-auto-level".to_owned()),
            "filter.threshold" => args.extend([
                "-channel".to_owned(),
                "RGB".to_owned(),
                "-threshold".to_owned(),
                "80%".to_owned(),
            ]),
            "filter.blur" => args.extend(["-blur".to_owned(), "1".to_owned()]),
            "filter.more-contrast" => args.extend([
                "+contrast".to_owned(),
                "+contrast".to_owned(),
                "+contrast".to_owned(),
            ]),
            _ => {
                return Err(ApiError::bad_request(format!(
                    "Unknown image filter: {filter}"
                )));
            }
        }
    }
    Ok(args)
}

async fn update_preview(
    state: &AppState,
    device: &scanner::Device,
    params: &ScanParams,
    source: &Path,
) -> Result<(), ApiError> {
    let mut args = vec![source.display().to_string()];
    if let (Some(x), Some(y)) = (
        device
            .features
            .get("-x")
            .and_then(|f| f.limits.as_ref())
            .and_then(|limits| limits.get(1)),
        device
            .features
            .get("-y")
            .and_then(|f| f.limits.as_ref())
            .and_then(|limits| limits.get(1)),
    ) {
        let scale = 868.0 / x;
        let height = (y * scale).round();
        let width = (params.width.unwrap_or(*x) * scale).round();
        let left = (params.left.unwrap_or(0.0) * scale).round();
        let top = (params.top.unwrap_or(0.0) * scale).round();
        args.extend([
            "-scale".to_owned(),
            format!("{width}"),
            "-background".to_owned(),
            "#808080".to_owned(),
            "-extent".to_owned(),
            format!("868x{height}-{left}-{top}"),
        ]);
    } else {
        args.extend(["-scale".to_owned(), "868".to_owned()]);
    }
    let path = state.preview_dir().join("preview.tmp.tif");
    args.push(path.display().to_string());
    run(&state.convert, &args).await?;
    tokio::fs::rename(path, state.preview_dir().join("preview.tif")).await?;
    Ok(())
}

async fn finish_output(
    state: &AppState,
    pages: Vec<PathBuf>,
    pipeline: &str,
) -> Result<PathBuf, ApiError> {
    let extension = if pipeline.contains("text-file") {
        "txt"
    } else if pipeline.contains("PDF") {
        "pdf"
    } else if pipeline.starts_with("PNG") {
        "png"
    } else if pipeline.starts_with("TIF") {
        "tif"
    } else {
        "jpg"
    };
    let stamp = Local::now().format("%Y-%m-%d %H.%M.%S");
    let output = state.output_dir().join(format!("scan_{stamp}.{extension}"));
    let quality = if pipeline.contains("high-quality") {
        "92"
    } else if pipeline.contains("low-quality") {
        "50"
    } else {
        "75"
    };
    let lzw = pipeline.contains("lzw-compressed");
    let ocr = pipeline.contains("pipeline.ocr");

    if extension == "txt" {
        let mut text = Vec::new();
        for page in pages {
            let result = run(
                &PathBuf::from("tesseract"),
                &[
                    page.display().to_string(),
                    "stdout".to_owned(),
                    "-l".to_owned(),
                    state.ocr_language.clone(),
                    "txt".to_owned(),
                ],
            )
            .await?;
            text.extend(result.stdout);
            text.push(b'\n');
        }
        tokio::fs::write(&output, text).await?;
    } else if extension == "pdf" && ocr {
        let mut parts = Vec::new();
        for (index, page) in pages.iter().enumerate() {
            let prefix = state.temp_dir().join(format!("ocr-{index:04}"));
            run(
                &PathBuf::from("tesseract"),
                &[
                    page.display().to_string(),
                    prefix.display().to_string(),
                    "-l".to_owned(),
                    state.ocr_language.clone(),
                    "pdf".to_owned(),
                ],
            )
            .await?;
            parts.push(prefix.with_extension("pdf"));
        }
        let mut args = parts
            .iter()
            .map(|path| path.display().to_string())
            .collect::<Vec<_>>();
        args.push(output.display().to_string());
        run(&PathBuf::from("pdfunite"), &args).await?;
    } else if extension == "pdf" {
        let mut args = Vec::new();
        if pipeline.contains("JPG") {
            let mut converted = Vec::new();
            for (index, page) in pages.iter().enumerate() {
                let image = state.temp_dir().join(format!("converted-{index:04}.jpg"));
                run(
                    &state.convert,
                    &[
                        page.display().to_string(),
                        "-quality".to_owned(),
                        quality.to_owned(),
                        image.display().to_string(),
                    ],
                )
                .await?;
                converted.push(image);
            }
            args.extend(converted.iter().map(|path| path.display().to_string()));
        } else {
            args.extend(pages.iter().map(|path| path.display().to_string()));
        }
        if lzw {
            args.extend(["-compress".to_owned(), "lzw".to_owned()]);
        }
        args.push(output.display().to_string());
        run(&state.convert, &args).await?;
    } else {
        let mut converted = Vec::new();
        for (index, page) in pages.iter().enumerate() {
            let image = state
                .temp_dir()
                .join(format!("output-{index:04}.{extension}"));
            let mut args = vec![page.display().to_string()];
            if extension == "jpg" || pipeline.contains("JPG") {
                args.extend(["-quality".to_owned(), quality.to_owned()]);
            }
            if extension == "tif" {
                args.extend([
                    "-compress".to_owned(),
                    if lzw { "lzw" } else { "none" }.to_owned(),
                ]);
            }
            args.push(image.display().to_string());
            run(&state.convert, &args).await?;
            converted.push(image);
        }
        if converted.len() == 1 {
            tokio::fs::rename(&converted[0], &output).await?;
        } else {
            write_zip(&output, &converted)?;
        }
    }
    Ok(output)
}

fn write_zip(destination: &Path, files: &[PathBuf]) -> Result<(), ApiError> {
    let file = File::create(destination)?;
    let mut archive = ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    let mut buffer = Vec::new();
    for path in files {
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("scan");
        archive
            .start_file(name, options)
            .map_err(|error| ApiError::server(error.to_string()))?;
        File::open(path)?.read_to_end(&mut buffer)?;
        archive.write_all(&buffer)?;
        buffer.clear();
    }
    archive
        .finish()
        .map_err(|error| ApiError::server(error.to_string()))?;
    Ok(())
}

async fn scan_files(directory: &Path) -> Result<Vec<PathBuf>, ApiError> {
    let mut entries = tokio::fs::read_dir(directory).await?;
    let mut files = Vec::new();
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default();
        if name.starts_with("~tmp-scan-") && name.ends_with(".tif") {
            files.push(path);
        }
    }
    files.sort();
    Ok(files)
}

async fn clear_scan_files(directory: &Path) -> Result<(), ApiError> {
    let mut entries = tokio::fs::read_dir(directory).await?;
    while let Some(entry) = entries.next_entry().await? {
        let name = entry.file_name();
        if name.to_string_lossy().starts_with("~tmp-scan-")
            || name.to_string_lossy().starts_with("filtered-")
            || name.to_string_lossy().starts_with("converted-")
            || name.to_string_lossy().starts_with("output-")
            || name.to_string_lossy().starts_with("ocr-")
        {
            let _ = tokio::fs::remove_file(entry.path()).await;
        }
    }
    Ok(())
}

fn collate(files: Vec<PathBuf>, standard: bool) -> Vec<PathBuf> {
    let mut odd = files
        .iter()
        .filter(|path| {
            path.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .contains("~tmp-scan-1-")
        })
        .cloned()
        .collect::<Vec<_>>();
    let mut even = files
        .iter()
        .filter(|path| {
            path.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .contains("~tmp-scan-2-")
        })
        .cloned()
        .collect::<Vec<_>>();
    odd.sort();
    even.sort();
    if standard {
        even.reverse();
    }
    let mut output = Vec::new();
    for (index, page) in odd.into_iter().enumerate() {
        output.push(page);
        if let Some(back) = even.get(index) {
            output.push(back.clone());
        }
    }
    output
}

pub async fn system_info() -> Json<Value> {
    let mem_available = std::fs::read_to_string("/proc/meminfo")
        .ok()
        .and_then(|text| {
            text.lines()
                .find(|line| line.starts_with("MemAvailable:"))
                .and_then(|line| line.split_whitespace().nth(1))
                .and_then(|value| value.parse::<u64>().ok())
        })
        .unwrap_or_default()
        / 1024;
    let os_release = std::fs::read_to_string("/etc/os-release").unwrap_or_default();
    let version = os_release
        .lines()
        .find_map(|line| line.strip_prefix("PRETTY_NAME="))
        .unwrap_or("Unavailable")
        .trim_matches('"');
    Json(json!({
        "os": {
            "arch": std::env::consts::ARCH,
            "freemem": mem_available,
            "platform": std::env::consts::OS,
            "release": version,
            "type": "Linux",
            "version": version,
        },
        "rust": env!("CARGO_PKG_VERSION"),
        "containerType": if Path::new("/.dockerenv").exists() { "docker" } else { "none" },
    }))
}

async fn run(binary: &Path, args: &[String]) -> Result<Output, ApiError> {
    let output = Command::new(binary)
        .args(args)
        .output()
        .await
        .map_err(|error| ApiError::server(format!("{}: {error}", binary.display())))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
        return Err(ApiError::server(if stderr.is_empty() {
            format!("{} exited with {}", binary.display(), output.status)
        } else {
            stderr
        }));
    }
    Ok(output)
}

fn file_info(path: &Path, metadata: std::fs::Metadata, deleted: Option<bool>) -> FileInfo {
    let modified = metadata.modified().unwrap_or(UNIX_EPOCH);
    let last_modified: DateTime<Utc> = modified.into();
    let size = metadata.len();
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_owned();
    FileInfo {
        extension: path
            .extension()
            .map(|ext| format!(".{}", ext.to_string_lossy()))
            .unwrap_or_default(),
        name,
        path: path
            .parent()
            .unwrap_or(Path::new("."))
            .display()
            .to_string(),
        last_modified: last_modified.to_rfc3339(),
        size,
        size_string: size_string(size),
        is_directory: metadata.is_dir(),
        deleted,
    }
}

fn size_string(size: u64) -> String {
    if size == 1 {
        "1 Byte".to_owned()
    } else if size < 2048 {
        format!("{size} Bytes")
    } else if size < 2 * 1024 * 1024 {
        format!("{} KB", (size as f64 / 1024.0).round() as u64)
    } else {
        format!("{:.2} MB", size as f64 / 1_048_576.0)
    }
}

fn validate_filename(name: &str) -> Result<(), ApiError> {
    if name.is_empty()
        || name == "."
        || name == ".."
        || name
            .chars()
            .any(|c| !(c.is_alphanumeric() || " ._-".contains(c)))
    {
        return Err(ApiError::bad_request(
            "Filename may contain letters, numbers, spaces, dots, underscores, and hyphens",
        ));
    }
    Ok(())
}

fn safe_path(directory: &Path, name: &str) -> Result<PathBuf, ApiError> {
    validate_filename(name)?;
    Ok(directory.join(name))
}
