use std::{
    any::{type_name, Any, TypeId},
    collections::HashMap,
    sync::Arc,
};

use axum::extract::FromRef;

use crate::error::FrameworkError;

/// A shared, reference-counted handle to a built [`Container`].
pub type ContainerRef = Arc<Container>;

/// Singleton dependency-injection container.
///
/// Register services with [`Container::register`] and resolve them with
/// [`Container::resolve`]. The type key `T` is typically an `Arc<dyn Trait>`
/// (which is `Sized`, `Clone`, `Send`, `Sync`, and `'static`).
///
/// # Example
/// ```no_run
/// # use ravix::Container;
/// # use std::sync::Arc;
/// let mut c = Container::new();
/// // c.register(Arc::new(InMemoryUserRepository::new()) as Arc<dyn UserRepository>);
/// // let repo: Arc<dyn UserRepository> = c.resolve();
/// ```
pub struct Container {
    singletons: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl Container {
    pub fn new() -> Self {
        Self {
            singletons: HashMap::new(),
        }
    }

    /// Register a singleton value for type key `T`.
    ///
    /// `T` is typically `Arc<dyn MyTrait>`. The concrete instance is stored
    /// and returned (cloned) on every call to `resolve::<T>()`.
    pub fn register<T: Clone + Send + Sync + 'static>(&mut self, instance: T) {
        self.singletons
            .insert(TypeId::of::<T>(), Box::new(instance));
    }

    /// Resolve the registered singleton for type key `T`.
    ///
    /// # Panics
    /// Panics with a descriptive message when no binding exists for `T`.
    pub fn resolve<T: Clone + Send + Sync + 'static>(&self) -> T {
        let type_id = TypeId::of::<T>();
        self.singletons
            .get(&type_id)
            .unwrap_or_else(|| {
                panic!(
                    "[ravix] No binding registered for `{}`. \
                     Call container.register::<{0}>(...) before App::build().",
                    type_name::<T>()
                )
            })
            .downcast_ref::<T>()
            .cloned()
            .unwrap_or_else(|| {
                panic!(
                    "[ravix] Type mismatch resolving `{}`. This is a bug in the DI container.",
                    type_name::<T>()
                )
            })
    }
}

impl Default for Container {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait implemented by `#[injectable]` structs.
///
/// The proc-macro generates an implementation that resolves each
/// `#[inject]`-annotated field from the container and constructs the type.
pub trait Injectable: Sized + Send + Sync + 'static {
    fn construct(container: &Container) -> Arc<Self>;
}

/// Axum extractor that resolves a registered `T` from [`ContainerRef`] state.
///
/// `T` is typically `Arc<dyn MyTrait>` — an owned, cloneable handle to a
/// service registered with [`Container::register`].
///
/// # Example
/// ```no_run
/// # use ravix::{Inject, Response};
/// # use std::sync::Arc;
/// // async fn handler(Inject(svc): Inject<Arc<dyn UserService>>) -> impl IntoResponse {
/// //     Response::json(svc.find_all().await)
/// // }
/// ```
pub struct Inject<T>(pub T);

#[async_trait::async_trait]
impl<T, S> axum::extract::FromRequestParts<S> for Inject<T>
where
    T: Clone + Send + Sync + 'static,
    S: Send + Sync,
    ContainerRef: FromRef<S>,
{
    type Rejection = FrameworkError;

    async fn from_request_parts(
        _parts: &mut axum::http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let container = ContainerRef::from_ref(state);
        Ok(Inject(container.resolve::<T>()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    trait Greeter: Send + Sync {
        fn greet(&self) -> &'static str;
    }

    struct HelloGreeter;
    impl Greeter for HelloGreeter {
        fn greet(&self) -> &'static str {
            "hello"
        }
    }

    #[test]
    fn bind_and_resolve_roundtrip() {
        let mut c = Container::new();
        c.register(Arc::new(HelloGreeter) as Arc<dyn Greeter>);
        let g: Arc<dyn Greeter> = c.resolve();
        assert_eq!(g.greet(), "hello");
    }

    #[test]
    fn resolve_returns_same_arc() {
        let mut c = Container::new();
        c.register(Arc::new(HelloGreeter) as Arc<dyn Greeter>);
        let a: Arc<dyn Greeter> = c.resolve();
        let b: Arc<dyn Greeter> = c.resolve();
        assert!(Arc::ptr_eq(&a, &b));
    }

    #[test]
    #[should_panic(expected = "No binding registered")]
    fn resolve_missing_panics() {
        let c = Container::new();
        let _: Arc<dyn Greeter> = c.resolve();
    }
}
