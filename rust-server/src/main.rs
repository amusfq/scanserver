mod api;
mod printer;
mod scanner;

use std::{env, net::SocketAddr, path::PathBuf, sync::Arc};

use axum::{
    Router,
    routing::{get, post},
};
use tokio::sync::Mutex;
use tower_http::services::{ServeDir, ServeFile};

#[derive(Clone)]
struct AppState {
    data_dir: PathBuf,
    static_dir: PathBuf,
    scanimage: PathBuf,
    convert: PathBuf,
    printer: printer::Printer,
    ocr_language: String,
    scan_lock: Arc<Mutex<()>>,
    devices: Arc<tokio::sync::RwLock<Option<Vec<scanner::Device>>>>,
}

impl AppState {
    fn from_env() -> Self {
        let data_dir = env::var_os("SCANSERV_DATA_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("./data"));
        let static_dir = env::var_os("SCANSERV_STATIC_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("./client"));

        Self {
            data_dir,
            static_dir,
            scanimage: env::var_os("SCANIMAGE")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("/usr/bin/scanimage")),
            convert: env::var_os("CONVERT")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("/usr/bin/convert")),
            printer: printer::Printer::from_env(),
            ocr_language: env::var("OCR_LANG")
                .ok()
                .filter(|language| !language.is_empty())
                .unwrap_or_else(|| "eng".to_owned()),
            scan_lock: Arc::new(Mutex::new(())),
            devices: Arc::new(tokio::sync::RwLock::new(None)),
        }
    }

    fn output_dir(&self) -> PathBuf {
        self.data_dir.join("output")
    }
    fn preview_dir(&self) -> PathBuf {
        self.data_dir.join("preview")
    }
    fn temp_dir(&self) -> PathBuf {
        self.data_dir.join("temp")
    }
    fn thumbnail_dir(&self) -> PathBuf {
        self.data_dir.join("thumbnail")
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let state = AppState::from_env();
    for directory in [
        state.output_dir(),
        state.preview_dir(),
        state.temp_dir(),
        state.thumbnail_dir(),
    ] {
        tokio::fs::create_dir_all(directory).await?;
    }

    let index = ServeFile::new(state.static_dir.join("index.html"));
    let app = Router::new()
        .route(
            "/api/v1/context",
            get(api::read_context).delete(api::delete_context),
        )
        .route("/api/v1/files", get(api::file_list))
        .route("/api/v1/files/{filename}/print", post(api::print_file))
        .route(
            "/api/v1/files/{filename}/actions/{action}",
            post(api::file_action),
        )
        .route("/api/v1/files/{filename}/thumbnail", get(api::thumbnail))
        .route(
            "/api/v1/files/{filename}",
            get(api::download)
                .delete(api::delete_file)
                .put(api::rename_file),
        )
        .route(
            "/api/v1/preview",
            get(api::read_preview)
                .post(api::create_preview)
                .delete(api::delete_preview),
        )
        .route("/api/v1/scan", post(api::scan))
        .route("/api/v1/print/capabilities", get(api::print_capabilities))
        .route(
            "/api/v1/print/upload",
            post(api::print_upload).layer(axum::extract::DefaultBodyLimit::max(82 * 1024 * 1024)),
        )
        .route("/api/v1/system", get(api::system_info))
        .route_service("/about", index.clone())
        .route_service("/files", index.clone())
        .route_service("/print", index.clone())
        .route_service("/scan", index.clone())
        .route_service("/settings", index)
        .fallback_service(
            ServeDir::new(state.static_dir.clone()).append_index_html_on_directories(true),
        )
        .with_state(Arc::new(state));

    let host = env::var("SCANSERV_HOST").unwrap_or_else(|_| "0.0.0.0".to_owned());
    let port = env::var("SCANSERV_PORT").unwrap_or_else(|_| "8080".to_owned());
    let address: SocketAddr = format!("{host}:{port}").parse()?;
    let listener = tokio::net::TcpListener::bind(address).await?;
    eprintln!("scanserver listening on http://{address}");
    axum::serve(listener, app).await?;
    Ok(())
}
