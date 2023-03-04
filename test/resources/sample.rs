use crate::data::AppState;
use crate::error::ServiceError;
use crate::model::proto::user::*;
use crate::model::token::*;
use crate::model::user::*;
use actix_web::web::{Data, HttpResponse, Json, Path, Query, ServiceConfig};

#[derive(Deserialize)]
pub struct UserPathInfo {
    pub id: String,
}

#[derive(Deserialize)]
pub struct UserQueryParams {
    pub alias: String,
}

#[derive(Deserialize)]
pub struct BatchUserQueryParams {
    pub id: String,
}

impl From<CreatePlayerRequest> for CreateUser {
    fn from(req: CreatePlayerRequest) -> Self {
        CreateUser {
            user_id: req.user_id,
            alias: req.alias,
            email: req.email,
            full_name: req.full_name,
            dci: req.dci,
        }
    }
}

impl UpdateUser {
    fn from(req: UpdatePlayerRequest, uid: String) -> Self {
        UpdateUser {
            uid,
            full_name: req.full_name,
            thumbnail: req.avatar,
            dci: req.dci,
        }
    }
}

#[post("/users")]
async fn create(
    params: Json<CreatePlayerRequest>,
    state: Data<AppState>,
) -> Result<Json<UserResponse>, ServiceError> {
    let req = params.into_inner();
    let create_user = CreateUser::from(req);
    let res = state.get_ref().db.send(create_user).await??;
    Ok(Json(res))
}

#[get("/users/{id}")]
async fn retrieve(
    path: Path<UserPathInfo>,
    _token: Token,
    state: Data<AppState>,
) -> Result<Json<UserResponse>, ServiceError> {
    let id = path.into_inner().id;
    let res = state.get_ref().db.send(FindUserByUid(id)).await??;
    Ok(Json(res))
}

#[put("/users/{id}")]
async fn update(
    path: Path<UserPathInfo>,
    params: Json<UpdatePlayerRequest>,
    _token: Token,
    state: Data<AppState>,
) -> Result<Json<UserResponse>, ServiceError> {
    let id = path.into_inner().id;
    let req = params.into_inner();

    let res = state.get_ref().db.send(UpdateUser::from(req, id)).await??;
    Ok(Json(res))
}

#[get("/users")]
async fn get_users_by_id(
    params: Query<BatchUserQueryParams>,
    _token: Token,
    state: Data<AppState>,
) -> Result<HttpResponse, ServiceError> {
    let ids = params
        .into_inner()
        .id
        .split(',')
        .map(|s| s.to_string())
        .collect::<Vec<String>>();

    let users = state.get_ref().db.send(FindUsersByUid { ids }).await??;
    Ok(HttpResponse::Ok().json(BatchUserDetailResponse { users }))
}

#[get("/users/search")]
async fn search(
    params: Query<UserQueryParams>,
    _token: Token,
    state: Data<AppState>,
) -> Result<HttpResponse, ServiceError> {
    let res = state
        .get_ref()
        .db
        .send(SearchForUserByName(params.into_inner().alias))
        .await??;

    Ok(HttpResponse::Ok().json(res))
}

pub fn configure_resources(cfg: &mut ServiceConfig) {
    cfg.service(search) // must be before /users/{id} to attempt to match this route first
        .service(create)
        .service(retrieve)
        .service(update)
        .service(get_users_by_id);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::controller::tests::*;
    use crate::create_app_state;
    use crate::model::proto::types::User as ProtoUser;
    use actix_web::{test, App};

    fn build_create_player(user_id: &String) -> CreatePlayerRequest {
        CreatePlayerRequest {
            user_id: format!("{}", user_id),
            email: format!("{}@test.com", user_id),
            alias: format!("{}", user_id),
            full_name: format!("{}", user_id),
            dci: Some(format!("{}", user_id)),
        }
    }

    #[actix_rt::test]
    async fn test_create_user() {
        let mut app = test::init_service(
            App::new()
                .data(create_app_state())
                .configure(configure_resources),
        )
        .await;

        let test_req = build_create_player(&"potato_potato".to_string());

        let req = test::TestRequest::post()
            .uri("/users")
            .set_json(&test_req)
            .to_request();

        let resp = test::call_service(&mut app, req).await;
        assert!(resp.status().is_success());

        let body = resp.response().body().as_str();
        let res: UserResponse = serde_json::from_str(body).unwrap();

        assert_eq!(res.uid, test_req.user_id);
        assert_eq!(res.alias, test_req.alias);
        assert_eq!(res.full_name, test_req.full_name);
        assert_eq!(res.games_played, 0);
        assert_eq!(res.wins, 0);
        assert_eq!(res.wl_ratio, 0.0);
        assert_eq!(res.dci, test_req.dci);
        assert_eq!(res.decks.len(), 0);
        assert_eq!(res.metas.len(), 0);
    }

    #[actix_rt::test]
    async fn test_update_user() {
        let mut app = test::init_service(
            App::new()
                .data(create_app_state())
                .configure(configure_resources),
        )
        .await;

        let create_data = build_create_player(&"potato_potato".to_string());

        let create_req = test::TestRequest::post()
            .uri("/users")
            .set_json(&create_data)
            .to_request();
        let _ = test::call_service(&mut app, create_req).await;

        let update_req = UpdatePlayerRequest {
            full_name: Some("Some Other Name".to_string()),
            thumbnail: None,
            dci: None,
            avatar: None,
        };

        let req = test::TestRequest::put()
            .uri("/users/potato_potato")
            .set_json(&update_req)
            .to_request();

        let resp = test::call_service(&mut app, req).await;
        assert!(resp.status().is_success());

        let body = resp.response().body().as_str();
        let res: UserResponse = serde_json::from_str(body).unwrap();

        assert_eq!(res.uid, create_data.user_id);
        assert_eq!(res.alias, create_data.alias);
        assert_eq!(res.full_name, update_req.full_name.unwrap());
        assert_eq!(res.games_played, 0);
        assert_eq!(res.wins, 0);
        assert_eq!(res.wl_ratio, 0.0);
        assert_eq!(res.dci, create_data.dci);
        assert_eq!(res.decks.len(), 0);
        assert_eq!(res.metas.len(), 0);
    }

    #[actix_rt::test]
    async fn test_get_user_with_uid() {
        let mut app = test::init_service(
            App::new()
                .data(create_app_state())
                .configure(configure_resources),
        )
        .await;

        let create_data = build_create_player(&"potato_potato".to_string());

        let create_req = test::TestRequest::post()
            .uri("/users")
            .set_json(&create_data)
            .to_request();
        let _ = test::call_service(&mut app, create_req).await;

        let req = test::TestRequest::get()
            .uri("/users/potato_potato")
            .to_request();

        let resp = test::call_service(&mut app, req).await;
        assert!(resp.status().is_success());

        let body = resp.response().body().as_str();
        let res: UserResponse = serde_json::from_str(body).unwrap();

        assert_eq!(res.uid, create_data.user_id);
        assert_eq!(res.alias, create_data.alias);
        assert_eq!(res.full_name, create_data.full_name);
        assert_eq!(res.games_played, 0);
        assert_eq!(res.wins, 0);
        assert_eq!(res.wl_ratio, 0.0);
        assert_eq!(res.dci, create_data.dci);
        assert_eq!(res.decks.len(), 0);
        assert_eq!(res.metas.len(), 0);
    }

    #[actix_rt::test]
    async fn test_search_user_by_alias() {
        let mut app = test::init_service(
            App::new()
                .data(create_app_state())
                .configure(configure_resources),
        )
        .await;

        for i in 0..30 {
            let create_data = build_create_player(&format!("potato_potato{}", i));

            let create_req = test::TestRequest::post()
                .uri("/users")
                .set_json(&create_data)
                .to_request();
            let _ = test::call_service(&mut app, create_req).await;
        }

        let req = test::TestRequest::get()
            .uri("/users/search?alias=potato")
            .to_request();

        let resp = test::call_service(&mut app, req).await;
        assert!(resp.status().is_success());

        let body = resp.response().body().as_str();
        let res: Vec<ProtoUser> = serde_json::from_str(body).unwrap();

        assert_eq!(res.len(), 20);
    }
}
