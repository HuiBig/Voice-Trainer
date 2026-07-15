mod environment;
mod projects;
mod python_worker;
mod runtime_paths;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(python_worker::PythonWorkerState::default())
        .invoke_handler(tauri::generate_handler![
            environment::check_environment,
            python_worker::start_python_worker,
            python_worker::python_worker_request,
            python_worker::stop_python_worker,
            projects::list_projects,
            projects::create_project,
            projects::open_project,
            projects::rename_project,
            projects::delete_project,
        ])
        .setup(|app| {
            projects::initialize(app.handle())?;
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
