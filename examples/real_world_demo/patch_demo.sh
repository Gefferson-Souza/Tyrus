#!/bin/bash

# Define Target File
TARGET="examples/real_world_demo/output/src/users/users_service.rs"

echo "🔧 Patching $TARGET..."

# 1. Fix 'create' method: Replace serde_json::json! with User struct instantiation
# Note: Unwrap fields from DTO (Arc<Mutex<String>>) to User (String)
sed -i 's/let new_user = serde_json :: json ! .*/let new_user = User { id: *self.id_counter.lock().unwrap(), name: create_user_dto.name.lock().unwrap().clone(), email: create_user_dto.email.lock().unwrap().clone(), is_active: true };/' "$TARGET"

# 2. Fix 'find_all' method: Convert MutexGuard to Vec using .to_vec()
sed -i 's/return self.users.lock().unwrap();/return self.users.lock().unwrap().to_vec();/' "$TARGET"

# 3. Fix 'find_one' method: Replace todo!() filter with proper iter().find()
sed -i 's/let found = self.users.lock().unwrap().filter(todo!());/return self.users.lock().unwrap().iter().find(|u| u.id == id).cloned();/' "$TARGET"

# 4. Remove unreachable/invalid code in 'find_one' (the if block and following todo)
# We delete lines from 'if found.length' down to 'unsupported literal'
sed -i '/if found.length > 0f64 {/,/return todo!("unsupported literal");/d' "$TARGET"

# 5. Fix 'push' ownership move: Clone new_user before push
sed -i 's/self.users.lock().unwrap().push(new_user);/self.users.lock().unwrap().push(new_user.clone());/' "$TARGET"

echo "✅ Patch applied successfully."

# 6. Rewrite main.rs to have valid entry point and manual wiring
# We need to wire UsersService, create Controller, and add it as Extension for extraction.
cat > examples/real_world_demo/output/src/main.rs <<EOF
mod app_module;
mod users;

use users::users_service::UsersService;
use users::users_controller::UsersController;
use axum::{Server, Extension};
use std::sync::Arc;
use std::net::SocketAddr;

#[derive(Debug)]
pub struct AppError(Box<dyn std::error::Error + Send + Sync>);

impl axum::response::IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            self.0.to_string(),
        )
            .into_response()
    }
}

impl<E> From<E> for AppError
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn from(err: E) -> Self {
        Self(Box::new(err))
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[tokio::main]
async fn main() {
    println!("🚀 Starting server on 0.0.0.0:3000");

    // 1. Instantiate Service
    let users_service = Arc::new(UsersService::new());

    // 2. Instantiate Controller
    // Controller takes Arc<UsersService> in constructor
    let users_controller = UsersController::new(users_service);
    let users_controller_arc = Arc::new(users_controller);

    // 3. Create Router and Add Extension Layer
    // The generated FromRequestParts expects Extension<Arc<UsersController>>
    let app = UsersController::router()
        .layer(Extension(users_controller_arc));

    // 4. Bind and Serve (Axum 0.6 style)
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
EOF

# 8. Remove PartialEq from CreateUserDto (Mutex doesn't support it)
sed -i 's/PartialEq, //' examples/real_world_demo/output/src/users/dto/create_user_dto.rs

echo "✅ main.rs rewritten, UsersController patched, and DTO fixed."
