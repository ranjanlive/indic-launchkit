#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use base64::{engine::general_purpose::STANDARD, Engine};
use launchkit_core::{self as core, Project};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};
struct AppEngine(Arc<Mutex<()>>);
#[derive(serde::Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
enum Request {
    Init {
        path: PathBuf,
        title: String,
    },
    Load {
        path: PathBuf,
    },
    Save {
        path: PathBuf,
        project: Project,
    },
    Import {
        path: PathBuf,
        source: PathBuf,
        kind: String,
        caption: String,
    },
    Preview {
        path: PathBuf,
        project: Project,
        kind: String,
        index: usize,
    },
    Export {
        path: PathBuf,
        project: Project,
        output: PathBuf,
        dry_run: bool,
        include_apk: bool,
    },
    Inspect {
        source: PathBuf,
    },
}
#[tauri::command]
async fn operation(
    request: Request,
    state: tauri::State<'_, AppEngine>,
) -> Result<serde_json::Value, String> {
    let engine = state.0.clone();
    tauri::async_runtime::spawn_blocking(move || {
      let _guard=engine.try_lock().map_err(|_|"Another operation is running; please wait".to_string())?;
      match request {
        Request::Init{path,title}=>serde_json::to_value(core::init(&path,&title,"","aurora")?).map_err(|e|e.to_string()),
        Request::Load{path}=>serde_json::to_value(core::load(&path)?).map_err(|e|e.to_string()),
        Request::Save{path,project}=>{core::save(&path,&project)?;Ok(serde_json::json!({"saved":true}))},
        Request::Import{path,source,kind,caption}=>serde_json::to_value(core::import(&path,&source,&kind,&caption)?).map_err(|e|e.to_string()),
        Request::Preview{path,project,kind,index}=>{core::validate(&path,&project,true)?;Ok(serde_json::json!({"image":format!("data:image/png;base64,{}",STANDARD.encode(core::render::png(&path,&project,&kind,index,true)?))}))},
        Request::Export{path,project,output,dry_run,include_apk}=>serde_json::to_value(core::export(&path,&project,&output,dry_run,include_apk)?).map_err(|e|e.to_string()),
        Request::Inspect{source}=>serde_json::to_value(core::apk::inspect(&source)?).map_err(|e|e.to_string()),
      }
    }).await.map_err(|e|e.to_string())?
}
fn main() {
    tauri::Builder::default()
        .manage(AppEngine(Arc::new(Mutex::new(()))))
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![operation])
        .run(tauri::generate_context!())
        .expect("Application runtime failed");
}
