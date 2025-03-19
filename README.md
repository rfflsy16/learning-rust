# Learning Rust - Product Management API

A RESTful API built with Rust for managing product inventory. This project demonstrates Rust's capabilities for building high-performance, type-safe web services using modern libraries like Axum, SQLx, and Tokio.

## Project Overview

This application provides a complete product management system with the following features:

- RESTful API for CRUD operations on products
- PostgreSQL database integration
- Structured error handling
- Data validation
- Filtering and pagination
- Database seeding
- Comprehensive logging

## Technology Stack

- **Framework**: Axum (web framework)
- **Runtime**: Tokio (async runtime)
- **Database**: PostgreSQL with SQLx (async database access)
- **Serialization**: Serde (JSON serialization/deserialization)
- **Error Handling**: Thiserror, Anyhow
- **Logging**: Tracing
- **Configuration**: dotenv
- **UUID Handling**: uuid
- **Date/Time**: Chrono
- **Decimal Support**: BigDecimal

## Getting Started

### Prerequisites

- Rust (latest stable version)
- PostgreSQL (running instance)
- Docker (optional, for containerized database)

### Environment Setup

Create a `.env` file in the project root with the following variables:
DATABASE_URL=postgres://username:password@localhost:5432/learning_rust
SERVER_HOST=127.0.0.1
SERVER_PORT=3000
RUST_LOG=info



### Database Setup

Run the migrations to set up the database schema:

```bash
cargo install sqlx-cli
sqlx database create
sqlx migrate run

```

### Running the Application
```bash
# Run in development mode 
cargo run --bin learning_rust

# Run the seeder explicitly
cargo run --bin seed
```

## API Documentation
### Product Endpoints List Products
```plaintext
GET /api/products
 ```

Query Parameters:

- name (optional): Filter by product name (partial match)
- category (optional): Filter by category (exact match)
- min_price (optional): Minimum price filter
- max_price (optional): Maximum price filter
- is_active (optional): Filter by active status (true/false)
- limit (optional): Maximum number of results to return
- offset (optional): Number of results to skip (for pagination)
Response:

```json
{
    "id": "cb78e70c-2555-4caf-ab4a-0904f7a7c733",
    "name": "Smartphone Ultra",
    "description": "Smartphone dengan kamera 108MP dan layar AMOLED",
    "price": 12000000.0,
    "stock": 25,
    "category": "Elektronik",
    "is_active": true,
    "created_at": "2025-03-18T13:18:09.787359Z",
    "updated_at": "2025-03-18T13:18:09.787359Z"
}
 ```
```
 Get Product by ID
GET /api/products/{id}
 ```

Response:

```json
{
    "id": "7bc33990-985f-4709-aedd-6f52fc9a5a9d",
    "name": "Webcam 4K",
    "description": "Webcam resolusi 4K dengan autofocus dan mikrofon dual",
    "price": 1300000.0,
    "stock": 20,
    "category": "Aksesoris",
    "is_active": true,
    "created_at": "2025-03-18T13:18:09.796231Z",
    "updated_at": "2025-03-18T13:18:09.796231Z"
}

```
 Create Product
```plaintext
POST /api/products
 ```

Request Body:

```json
{
  "name": "New Product",
  "description": "Product description",
  "price": 1000.00,
  "stock": 10,
  "category": "Category"
}

```

Response:


```json
{
  "id": "ce28c9c2-3bad-493b-860d-d9139c93647b",
  "name": "New Product",
  "description": "Product description",
  "price": 1000.00,
  "stock": 10,
  "category": "Category",
  "is_active": true,
  "created_at": "2023-01-01T00:00:00Z",
  "updated_at": "2023-01-01T00:00:00Z"
}

 ```
```
 Update Product

PUT /api/products/{id}
 ```

Request Body (all fields optional):

```json
{
  "name": "Updated Product",
  "description": "Updated description",
  "price": 1500.00,
  "stock": 20,
  "category": "Updated Category",
  "is_active": true
}
```

Response:

```json
{
  "id": "ce28c9c2-3bad-493b-860d-d9139c93647b",
  "name": "Updated Product",
  "description": "Updated description",
  "price": 1500.00,
  "stock": 20,
  "category": "Updated Category",
  "is_active": true,
  "created_at": "2023-01-01T00:00:00Z",
  "updated_at": "2023-01-01T00:00:00Z"
}
 ```
```
 Delete Product

DELETE /api/products/{id}
 ```

Response:

```json
{
  "success": true,
  "message": "Product with ID {id} successfully deleted"
}
```

### Error Responses
The API returns appropriate HTTP status codes and error messages:

```json
{
  "error": "Error message"
}
 ```

Common status codes:

- 400 Bad Request : Invalid input data
- 404 Not Found : Resource not found
- 500 Internal Server Error : Server-side error
## Project Structure
```plaintext
learning-rust/
├── data/                  # Seed data
│   └── products.json      # Product seed data
├── migrations/            # Database migrations
├── src/
│   ├── bin/               # Binary executables
│   │   └── seed.rs        # Database seeder
│   ├── core/              # Core application components
│   │   ├── config.rs      # Configuration management
│   │   ├── db.rs          # Database connection
│   │   ├── error.rs       # Error handling
│   │   ├── mod.rs         # Module exports
│   │   └── server.rs      # HTTP server setup
│   ├── domain/            # Business domain modules
│   │   ├── product/       # Product domain
│   │   │   ├── model.rs   # Data models
│   │   │   ├── repository.rs # Database operations
│   │   │   ├── routes.rs  # HTTP routes
│   │   │   ├── service.rs # Business logic
│   │   │   └── mod.rs     # Module exports
│   │   ├── user/          # User domain (future)
│   │   └── mod.rs         # Module exports
│   ├── seeder/            # Database seeders
│   │   ├── product.rs     # Product seeder
│   │   └── mod.rs         # Module exports
│   ├── utils/             # Utility functions
│   │   └── mod.rs         # Utility functions
│   ├── lib.rs             # Library exports
│   └── main.rs            # Application entry point
└── Cargo.toml             # Project dependencies
 ```
```

## Development
### Adding New Features
1. Create appropriate models in the domain module
2. Implement repository for database operations
3. Create service layer for business logic
4. Define routes for HTTP endpoints
5. Register routes in the server module
### Running Tests
```bash
cargo test
 ```

## Future Enhancements
- User authentication and authorization
- API rate limiting
- Caching layer
- More comprehensive validation
- Additional filtering options
- Swagger/OpenAPI documentation
- GraphQL API
## License
This project is licensed under the MIT License - see the LICENSE file for details.

```plaintext

This README provides a comprehensive overview of your Rust product management API, including detailed API documentation, project structure, setup instructions, and future enhancement possibilities. It should give users and developers a clear understanding of what your application does and how to use it.
 ```
```