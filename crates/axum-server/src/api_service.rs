use grpc_api::api::*;
use crate::errors::CustomError;
use db::queries;
use db::Pool;

use tonic::{Request, Response, Status};

pub struct UsersService {
    pub pool: Pool,
}

#[tonic::async_trait]
impl grpc_api::api::users_server::Users for UsersService {
    async fn get_users(
        &self,
        _request: Request<GetUsersRequest>,
    ) -> Result<Response<GetUsersResponse>, Status> {
        // Get a client from our database pool
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| CustomError::Database(e.to_string())).unwrap();

        // Get the fortunes from the database
        let fortunes = queries::users::get_users()
            .bind(&client)
            .all()
            .await
            .map_err(|e| CustomError::Database(e.to_string()))
            .unwrap();

        // Map the structs we get from cornucopia to the structs
        // we need for our gRPC reply.
        let users = fortunes
            .into_iter()
            .map(|user| User {
                id: user.id as i32,
                email: user.email,
            })
            .collect();

        let response = GetUsersResponse {
            users,
        };

        return Ok(Response::new(response));
    }
}