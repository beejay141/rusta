use testcontainers::clients::Cli;
use testcontainers::core::{GenericContainer, WaitFor};
use testcontainers::images::generic::GenericImage;
use testcontainers::Container;
use testcontainers_modules::mongo;

/// Starts the service container along with its dependencies (MongoDB).
///
/// This function:
/// 1. Starts a MongoDB container
/// 2. Starts the rusta-example service container with the MongoDB connection string
/// 3. Waits for the service to print its startup log ("Listening on ...")
/// 4. Returns both containers so the caller can stop them when done
///
/// The caller is responsible for stopping the containers when done.
pub fn startServiceContainer() -> (Container<mongo::Mongo>, GenericContainer) {
    let docker = Cli::default();

    // Start MongoDB container
    let mongo_container = mongo::Mongo::default().start(&docker);

    // Get the MongoDB host port
    let mongo_port = mongo_container.get_host_port_ipv4(27017);
    let mongo_uri = format!("mongodb://localhost:{}", mongo_port);

    // Generate a unique database name for test isolation
    let db_name = format!("blog_test_{}", uuid::Uuid::new_v4().simple());

    // Build the service container image with environment variables.
    // Use testcontainers' built-in wait strategy to block until the service
    // prints its startup log ("Listening on ..."), which is emitted by main.rs
    // once the HTTP server is bound and ready to accept connections.
    let service_image = GenericImage::new("rusta-example", "test")
        .with_exposed_port(3001u16)
        .with_env_var("MONGO_URI", mongo_uri)
        .with_env_var("MONGO_DB", db_name)
        .with_env_var("JWT_SECRET", "test_secret_for_integration")
        .with_env_var("JWT_EXPIRY_SECONDS", "3600")
        .with_env_var("SERVER_PORT", "0.0.0.0:3001")
        .with_wait_for(WaitFor::message_on_stdout("Listening on"));

    // Start the service container (blocks until the wait strategy resolves)
    let service_container = service_image.start(&docker);

    (mongo_container, service_container)
}
