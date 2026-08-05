# rusta-di

Dependency injection container for the [Rusta](https://github.com/beejay141/rusta) framework.

## Usage

```rust
use rusta_di::Container;

let mut container = Container::new();
container.register::<dyn UserRepository, InMemoryUserRepository>();
container.register::<dyn UserService, UserServiceImpl>();

let svc: Arc<dyn UserService> = container.resolve();
```

## License

Dual-licensed under MIT or Apache-2.0.
