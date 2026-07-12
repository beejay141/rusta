use proc_macro::TokenStream;

mod controller;
mod injectable;
mod routes;

/// Register an `impl` block as a controller.
///
/// The attribute argument is the base path (e.g. `"/users"`).  Every method
/// inside the block that carries a `#[get]`, `#[post]`, `#[put]`,
/// `#[delete]`, or `#[patch]` attribute is registered as an HTTP route via
/// `inventory::submit!`.
///
/// Optional `#[middleware(fn_path)]` on a method wraps that route with the
/// given `axum::middleware::from_fn` middleware function.
///
/// # Example
/// ```ignore
/// #[controller("/users")]
/// impl UserController {
///     #[get("/")]
///     async fn list(Inject(svc): Inject<Arc<dyn UserService>>) -> AxumResponse {
///         Response::json(svc.find_all().await)
///     }
///
///     #[get("/:id")]
///     #[middleware(auth_guard)]
///     async fn get(
///         Path(id): Path<Uuid>,
///         Inject(svc): Inject<Arc<dyn UserService>>,
///     ) -> AxumResponse {
///         Response::json(svc.find_by_id(id).await)
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn controller(args: TokenStream, input: TokenStream) -> TokenStream {
    controller::expand(args, input)
}

/// Mark a handler method as an HTTP GET route.
/// Must be used inside a `#[controller]` block.
#[proc_macro_attribute]
pub fn get(args: TokenStream, input: TokenStream) -> TokenStream {
    routes::noop_marker(args, input)
}

/// Mark a handler method as an HTTP POST route.
/// Must be used inside a `#[controller]` block.
#[proc_macro_attribute]
pub fn post(args: TokenStream, input: TokenStream) -> TokenStream {
    routes::noop_marker(args, input)
}

/// Mark a handler method as an HTTP PUT route.
/// Must be used inside a `#[controller]` block.
#[proc_macro_attribute]
pub fn put(args: TokenStream, input: TokenStream) -> TokenStream {
    routes::noop_marker(args, input)
}

/// Mark a handler method as an HTTP DELETE route.
/// Must be used inside a `#[controller]` block.
#[proc_macro_attribute]
pub fn delete(args: TokenStream, input: TokenStream) -> TokenStream {
    routes::noop_marker(args, input)
}

/// Mark a handler method as an HTTP PATCH route.
/// Must be used inside a `#[controller]` block.
#[proc_macro_attribute]
pub fn patch(args: TokenStream, input: TokenStream) -> TokenStream {
    routes::noop_marker(args, input)
}

/// Attach a tower middleware function to a specific handler.
/// Must be used inside a `#[controller]` block, directly above a route attribute.
#[proc_macro_attribute]
pub fn middleware(args: TokenStream, input: TokenStream) -> TokenStream {
    routes::noop_marker(args, input)
}

/// Derive `rusta_di::Injectable` for a struct.
///
/// Fields annotated with `#[inject]` are resolved from the DI container.
/// All other fields fall back to `Default::default()`.
///
/// # Example
/// ```ignore
/// #[injectable]
/// pub struct UserServiceImpl {
///     #[inject]
///     pub repo: Arc<dyn UserRepository>,
/// }
/// ```
#[proc_macro_attribute]
pub fn injectable(args: TokenStream, input: TokenStream) -> TokenStream {
    injectable::expand(args, input)
}