use axum::{
    extract::{Path, State},
    response::{Html, IntoResponse},
    Json,
};
use minijinja::Environment;
use serde::Serialize;
use sqlx::SqlitePool;
use std::sync::Arc;
use crate::db::Db;
use tracing::{info, error};
use sqlx::types::chrono::Utc;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: SqlitePool,
    pub jinja: Arc<Environment<'static>>,
}

#[derive(Serialize)]
struct DashboardContext {
    programs: Vec<ProgramContext>,
}

#[derive(Serialize)]
struct ProgramContext {
    name: String,
    current_version: String,
    latest_version: String,
    has_update: bool,
    current_version_last_updated: String,
    latest_version_last_updated: String,
}

pub async fn get_dashboard(
    State(state): State<AppState>,
) -> impl IntoResponse {
    info!("Serving dashboard");
    let db = Db { pool: state.db_pool.clone() };
    let programs = match db.get_all_programs().await {
        Ok(p) => p,
        Err(e) => {
            error!("Failed to fetch programs: {e}");
            return Html("<h1>Error fetching programs</h1>").into_response();
        }
    };

    let context = DashboardContext {
        programs: programs.into_iter().map(|p| {
            let has_update = p.current_version != p.latest_version;
            ProgramContext {
                name: p.name,
                current_version: p.current_version,
                latest_version: p.latest_version,
                has_update,
                current_version_last_updated: crate::format_datetime(&p.current_version_last_updated),
                latest_version_last_updated: crate::format_datetime(&p.latest_version_last_updated),
            }
        }).collect(),
    };

    match state.jinja.get_template("index.html").and_then(|t| t.render(context)) {
        Ok(html) => Html(html).into_response(),
        Err(e) => {
            error!("Failed to render template: {e}");
            Html(format!("<h1>Template error: {e}</h1>")).into_response()
        }
    }
}

pub async fn post_update_program(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    info!("Marking program {} as updated", name);
    let db = Db { pool: state.db_pool.clone() };

    let program = match db.get_program(&name).await {
        Ok(Some(p)) => p,
        Ok(None) => {
            return (axum::http::StatusCode::NOT_FOUND, Json(serde_json::json!({"status": "error", "message": "Program not found"}))).into_response();
        }
        Err(e) => {
            error!("Error fetching program: {e}");
            return (axum::http::StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"status": "error", "message": e.to_string()}))).into_response();
        }
    };

    if program.current_version == program.latest_version {
        return (axum::http::StatusCode::BAD_REQUEST, Json(serde_json::json!({"status": "error", "message": "Program is already up to date"}))).into_response();
    }

    let new_version = program.latest_version.clone();
    let new_version_last_updated = Utc::now().naive_utc();

    match db.update_current_version(&name, &new_version, new_version_last_updated).await {
        Ok(_) => {
            info!("Successfully updated {} to {}", name, new_version);
            (axum::http::StatusCode::OK, Json(serde_json::json!({"status": "success"}))).into_response()
        }
        Err(e) => {
            error!("Error updating program: {e}");
            (axum::http::StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"status": "error", "message": e.to_string()}))).into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::SqlitePool;
    use crate::{Program, db::tests};
    use crate::Provider;
    use sqlx::types::chrono::{NaiveDate, NaiveDateTime, NaiveTime};

    #[sqlx::test]
    async fn test_get_dashboard(pool: SqlitePool) {
        let db = tests::db(pool.clone());
        let program = Program {
            name: "test_program".to_string(),
            current_version: "1.0.0".to_string(),
            current_version_last_updated: NaiveDateTime::new(
                NaiveDate::parse_from_str("2025-01-01", "%Y-%m-%d").unwrap(),
                NaiveTime::parse_from_str("12:00:00", "%H:%M:%S").unwrap(),
            ),
            latest_version: "1.0.0".to_string(),
            latest_version_last_updated: NaiveDateTime::new(
                NaiveDate::parse_from_str("2025-01-01", "%Y-%m-%d").unwrap(),
                NaiveTime::parse_from_str("12:00:00", "%H:%M:%S").unwrap(),
            ),
            provider: Provider::Github("user/repo".to_string()),
        };
        db.insert_program(&program).await.unwrap();

        let mut jinja = Environment::new();
        jinja.add_template("index.html", "<h1>{{ programs[0].name }}</h1>").unwrap();

        let state = AppState {
            db_pool: pool,
            jinja: Arc::new(jinja),
        };

        let response = get_dashboard(State(state)).await.into_response();
        assert_eq!(response.status(), axum::http::StatusCode::OK);
    }

    #[sqlx::test]
    async fn test_post_update_program_success(pool: SqlitePool) {
        let db = tests::db(pool.clone());
        let program = Program {
            name: "test_program".to_string(),
            current_version: "1.0.0".to_string(),
            current_version_last_updated: NaiveDateTime::new(
                NaiveDate::parse_from_str("2025-01-01", "%Y-%m-%d").unwrap(),
                NaiveTime::parse_from_str("12:00:00", "%H:%M:%S").unwrap(),
            ),
            latest_version: "1.1.0".to_string(),
            latest_version_last_updated: NaiveDateTime::new(
                NaiveDate::parse_from_str("2025-01-02", "%Y-%m-%d").unwrap(),
                NaiveTime::parse_from_str("12:00:00", "%H:%M:%S").unwrap(),
            ),
            provider: Provider::Github("user/repo".to_string()),
        };
        db.insert_program(&program).await.unwrap();

        let state = AppState {
            db_pool: pool,
            jinja: Arc::new(Environment::new()),
        };

        let response = post_update_program(State(state), Path("test_program".to_string())).await.into_response();
        assert_eq!(response.status(), axum::http::StatusCode::OK);
    }

    #[sqlx::test]
    async fn test_post_update_program_not_found(pool: SqlitePool) {
        let state = AppState {
            db_pool: pool,
            jinja: Arc::new(Environment::new()),
        };

        let response = post_update_program(State(state), Path("non_existent".to_string())).await.into_response();
        assert_eq!(response.status(), axum::http::StatusCode::NOT_FOUND);
    }

    #[sqlx::test]
    async fn test_post_update_program_already_up_to_date(pool: SqlitePool) {
        let db = tests::db(pool.clone());
        let program = Program {
            name: "test_program".to_string(),
            current_version: "1.0.0".to_string(),
            current_version_last_updated: NaiveDateTime::new(
                NaiveDate::parse_from_str("2025-01-01", "%Y-%m-%d").unwrap(),
                NaiveTime::parse_from_str("12:00:00", "%H:%M:%S").unwrap(),
            ),
            latest_version: "1.0.0".to_string(),
            latest_version_last_updated: NaiveDateTime::new(
                NaiveDate::parse_from_str("2025-01-01", "%Y-%m-%d").unwrap(),
                NaiveTime::parse_from_str("12:00:00", "%H:%M:%S").unwrap(),
            ),
            provider: Provider::Github("user/repo".to_string()),
        };
        db.insert_program(&program).await.unwrap();

        let state = AppState {
            db_pool: pool,
            jinja: Arc::new(Environment::new()),
        };

        let response = post_update_program(State(state), Path("test_program".to_string())).await.into_response();
        assert_eq!(response.status(), axum::http::StatusCode::BAD_REQUEST);
    }
}
