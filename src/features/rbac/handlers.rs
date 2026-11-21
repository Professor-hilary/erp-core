//! Admin RBAC HTTP surface — membership, roles, modules, /me
use crate::{
    features::rbac::{Actor, Module, RbacService},
    features::rbac::service::UpsertMembershipRequest,
    interface::api::{errors::AppError, json_errors::AppJson, responses::ApiResponse},
    middleware::auth::AuthenticatedTenant,
    state::AppState,
};
use axum::{
    Extension, Router,
    extract::{Path, State},
    response::Response,
    routing::{get, post, put},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/me", get(me))
        .route("/roles", get(list_roles))
        .route("/permissions", get(list_permissions))
        .route("/memberships", get(list_memberships))
        .route("/memberships", post(upsert_membership))
        .route("/memberships/{user_uuid}/roles", put(set_roles))
        .route("/memberships/{user_uuid}/modules", put(set_modules))
        .route("/memberships/{user_uuid}/department", put(set_department))
}

fn require_user_admin(actor: &Actor) -> Result<(), AppError> {
    actor.require_module(Module::Settings)?;
    actor.require("settings.manage_users")
}

#[derive(Debug, Serialize)]
struct MeResponse {
    user_id: Uuid,
    company_id: Uuid,
    is_tenant_admin: bool,
    department_uuid: Option<Uuid>,
    authz_version: i64,
    roles: Vec<String>,
    modules: Vec<String>,
    permissions: Vec<String>,
}

async fn me(Extension(actor): Extension<Actor>) -> Result<Response, AppError> {
    let mut roles: Vec<_> = actor.roles.iter().cloned().collect();
    let mut modules: Vec<_> = actor.modules.iter().cloned().collect();
    let mut permissions: Vec<_> = actor.permissions.iter().cloned().collect();
    roles.sort();
    modules.sort();
    permissions.sort();
    Ok(ApiResponse::success(
        MeResponse {
            user_id: actor.user_id,
            company_id: actor.company_id,
            is_tenant_admin: actor.is_tenant_admin,
            department_uuid: actor.department_uuid,
            authz_version: actor.authz_version,
            roles,
            modules,
            permissions,
        },
        "Current authorization context",
    ))
}

async fn list_roles(
    Extension(user): Extension<AuthenticatedTenant>,
    Extension(actor): Extension<Actor>,
) -> Result<Response, AppError> {
    require_user_admin(&actor)?;
    let rows = RbacService::list_roles(&user.tenant_pool).await?;
    Ok(ApiResponse::success(rows, "Roles"))
}

async fn list_permissions(
    Extension(user): Extension<AuthenticatedTenant>,
    Extension(actor): Extension<Actor>,
) -> Result<Response, AppError> {
    require_user_admin(&actor)?;
    let rows = RbacService::list_permissions(&user.tenant_pool).await?;
    Ok(ApiResponse::success(rows, "Permissions"))
}

async fn list_memberships(
    Extension(user): Extension<AuthenticatedTenant>,
    Extension(actor): Extension<Actor>,
) -> Result<Response, AppError> {
    require_user_admin(&actor)?;
    let rows = RbacService::list_memberships(&user.tenant_pool).await?;
    Ok(ApiResponse::success(rows, "Memberships"))
}



async fn upsert_membership(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    Extension(actor): Extension<Actor>,
    AppJson(body): AppJson<UpsertMembershipRequest>,
) -> Result<Response, AppError> {
    require_user_admin(&actor)?;
    RbacService::upsert_membership(
        &user.tenant_pool,
        &state.authz_cache,
        user.company_id,
        body,
    )
    .await?;
    Ok(ApiResponse::success(serde_json::json!({}), "Membership upserted"))
}

#[derive(Debug, Deserialize)]
pub struct SetRolesRequest {
    pub roles: Vec<String>,
}

async fn set_roles(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    Extension(actor): Extension<Actor>,
    Path(user_uuid): Path<Uuid>,
    AppJson(body): AppJson<SetRolesRequest>,
) -> Result<Response, AppError> {
    require_user_admin(&actor)?;
    RbacService::set_roles(
        &user.tenant_pool,
        &state.authz_cache,
        user.company_id,
        user_uuid,
        &body.roles,
    )
    .await?;
    Ok(ApiResponse::success(serde_json::json!({}), "Roles updated"))
}

#[derive(Debug, Deserialize)]
pub struct SetModulesRequest {
    pub modules: Vec<String>,
}

async fn set_modules(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    Extension(actor): Extension<Actor>,
    Path(user_uuid): Path<Uuid>,
    AppJson(body): AppJson<SetModulesRequest>,
) -> Result<Response, AppError> {
    require_user_admin(&actor)?;
    RbacService::set_modules(
        &user.tenant_pool,
        &state.authz_cache,
        user.company_id,
        user_uuid,
        &body.modules,
    )
    .await?;
    Ok(ApiResponse::success(serde_json::json!({}), "Modules updated"))
}

#[derive(Debug, Deserialize)]
pub struct SetDepartmentRequest {
    pub department_uuid: Option<Uuid>,
}

async fn set_department(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    Extension(actor): Extension<Actor>,
    Path(user_uuid): Path<Uuid>,
    AppJson(body): AppJson<SetDepartmentRequest>,
) -> Result<Response, AppError> {
    require_user_admin(&actor)?;
    RbacService::set_department(
        &user.tenant_pool,
        &state.authz_cache,
        user.company_id,
        user_uuid,
        body.department_uuid,
    )
    .await?;
    Ok(ApiResponse::success(serde_json::json!({}), "Department updated"))
}
