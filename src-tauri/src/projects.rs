use rusqlite::{params, Connection, ErrorCode};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
use tauri::{AppHandle, Manager};

const DEFAULT_STATUS: &str = "not_started";

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TrainingProject {
    id: i64,
    name: String,
    material_directory: String,
    training_status: String,
    created_at: i64,
    updated_at: i64,
    last_opened_at: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateProjectInput {
    name: String,
    material_directory: String,
}

fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

fn validate_name(name: &str) -> Result<String, String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("项目名称不能为空".to_string());
    }
    if name.chars().count() > 80 {
        return Err("项目名称不能超过 80 个字符".to_string());
    }
    Ok(name.to_string())
}

fn validate_material_directory(directory: &str) -> Result<String, String> {
    let directory = directory.trim();
    if directory.is_empty() {
        return Err("素材目录不能为空".to_string());
    }
    Ok(directory.to_string())
}

fn database_path(app: &AppHandle) -> Result<PathBuf, String> {
    let directory = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("无法定位应用数据目录：{error}"))?;
    fs::create_dir_all(&directory).map_err(|error| format!("无法创建应用数据目录：{error}"))?;
    Ok(directory.join("voice-trainer.sqlite3"))
}

fn connect(path: &Path) -> Result<Connection, String> {
    let connection = Connection::open(path).map_err(database_error)?;
    connection
        .execute_batch(
            "PRAGMA foreign_keys = ON;
             PRAGMA journal_mode = WAL;
             CREATE TABLE IF NOT EXISTS training_projects (
               id INTEGER PRIMARY KEY AUTOINCREMENT,
               name TEXT NOT NULL COLLATE NOCASE UNIQUE,
               material_directory TEXT NOT NULL,
               training_status TEXT NOT NULL DEFAULT 'not_started',
               created_at INTEGER NOT NULL,
               updated_at INTEGER NOT NULL,
               last_opened_at INTEGER
             );",
        )
        .map_err(database_error)?;
    Ok(connection)
}

fn database_error(error: rusqlite::Error) -> String {
    match &error {
        rusqlite::Error::SqliteFailure(details, _)
            if details.code == ErrorCode::ConstraintViolation =>
        {
            "已存在同名项目".to_string()
        }
        _ => format!("项目数据库操作失败：{error}"),
    }
}

fn project_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<TrainingProject> {
    Ok(TrainingProject {
        id: row.get(0)?,
        name: row.get(1)?,
        material_directory: row.get(2)?,
        training_status: row.get(3)?,
        created_at: row.get(4)?,
        updated_at: row.get(5)?,
        last_opened_at: row.get(6)?,
    })
}

fn get_project(connection: &Connection, id: i64) -> Result<TrainingProject, String> {
    connection
        .query_row(
            "SELECT id, name, material_directory, training_status, created_at, updated_at, last_opened_at
             FROM training_projects WHERE id = ?1",
            [id],
            project_from_row,
        )
        .map_err(|error| match error {
            rusqlite::Error::QueryReturnedNoRows => "项目不存在或已被删除".to_string(),
            other => database_error(other),
        })
}

fn list(connection: &Connection) -> Result<Vec<TrainingProject>, String> {
    let mut statement = connection
        .prepare(
            "SELECT id, name, material_directory, training_status, created_at, updated_at, last_opened_at
             FROM training_projects ORDER BY COALESCE(last_opened_at, created_at) DESC, id DESC",
        )
        .map_err(database_error)?;
    let projects = statement
        .query_map([], project_from_row)
        .map_err(database_error)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(database_error)?;
    Ok(projects)
}

fn create(connection: &Connection, input: CreateProjectInput) -> Result<TrainingProject, String> {
    let name = validate_name(&input.name)?;
    let material_directory = validate_material_directory(&input.material_directory)?;
    let timestamp = now();
    connection
        .execute(
            "INSERT INTO training_projects (name, material_directory, training_status, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?4)",
            params![name, material_directory, DEFAULT_STATUS, timestamp],
        )
        .map_err(database_error)?;
    get_project(connection, connection.last_insert_rowid())
}

fn rename(connection: &Connection, id: i64, name: &str) -> Result<TrainingProject, String> {
    let name = validate_name(name)?;
    let changed = connection
        .execute(
            "UPDATE training_projects SET name = ?1, updated_at = ?2 WHERE id = ?3",
            params![name, now(), id],
        )
        .map_err(database_error)?;
    if changed == 0 {
        return Err("项目不存在或已被删除".to_string());
    }
    get_project(connection, id)
}

fn open(connection: &Connection, id: i64) -> Result<TrainingProject, String> {
    let timestamp = now();
    let changed = connection
        .execute(
            "UPDATE training_projects SET last_opened_at = ?1, updated_at = ?1 WHERE id = ?2",
            params![timestamp, id],
        )
        .map_err(database_error)?;
    if changed == 0 {
        return Err("项目不存在或已被删除".to_string());
    }
    get_project(connection, id)
}

fn delete(connection: &Connection, id: i64) -> Result<(), String> {
    let changed = connection
        .execute("DELETE FROM training_projects WHERE id = ?1", [id])
        .map_err(database_error)?;
    if changed == 0 {
        return Err("项目不存在或已被删除".to_string());
    }
    Ok(())
}

pub fn initialize(app: &AppHandle) -> Result<(), String> {
    connect(&database_path(app)?)?;
    Ok(())
}

#[tauri::command]
pub fn list_projects(app: AppHandle) -> Result<Vec<TrainingProject>, String> {
    list(&connect(&database_path(&app)?)?)
}

#[tauri::command]
pub fn create_project(
    app: AppHandle,
    input: CreateProjectInput,
) -> Result<TrainingProject, String> {
    create(&connect(&database_path(&app)?)?, input)
}

#[tauri::command]
pub fn open_project(app: AppHandle, id: i64) -> Result<TrainingProject, String> {
    open(&connect(&database_path(&app)?)?, id)
}

#[tauri::command]
pub fn rename_project(app: AppHandle, id: i64, name: String) -> Result<TrainingProject, String> {
    rename(&connect(&database_path(&app)?)?, id, &name)
}

#[tauri::command]
pub fn delete_project(app: AppHandle, id: i64) -> Result<(), String> {
    delete(&connect(&database_path(&app)?)?, id)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_connection() -> Connection {
        connect(Path::new(":memory:")).expect("in-memory database should initialize")
    }

    #[test]
    fn project_crud_round_trip() {
        let connection = test_connection();
        let created = create(
            &connection,
            CreateProjectInput {
                name: "  主唱 A  ".into(),
                material_directory: "D:\\audio\\lead".into(),
            },
        )
        .expect("project should be created");
        assert_eq!(created.name, "主唱 A");
        assert_eq!(created.training_status, DEFAULT_STATUS);
        assert_eq!(list(&connection).unwrap().len(), 1);

        let renamed = rename(&connection, created.id, "主唱 B").unwrap();
        assert_eq!(renamed.name, "主唱 B");
        let opened = open(&connection, created.id).unwrap();
        assert!(opened.last_opened_at.is_some());

        delete(&connection, created.id).unwrap();
        assert!(list(&connection).unwrap().is_empty());
    }

    #[test]
    fn duplicate_and_invalid_names_are_rejected() {
        let connection = test_connection();
        create(
            &connection,
            CreateProjectInput {
                name: "Demo".into(),
                material_directory: "D:\\audio".into(),
            },
        )
        .unwrap();
        let duplicate = create(
            &connection,
            CreateProjectInput {
                name: "demo".into(),
                material_directory: "D:\\audio2".into(),
            },
        );
        assert_eq!(duplicate.unwrap_err(), "已存在同名项目");
        assert_eq!(
            rename(&connection, 1, "  ").unwrap_err(),
            "项目名称不能为空"
        );
    }
}
