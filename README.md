# pgdb-copypaste-rs

pgdb-copypaste-rs contains a silly script that migrates data from one PostgreSQL database to another. The script first checks if the destination database exists, and if it does, it terminates connections to the database and drops it. Then it creates a new database with the same name. The script proceeds to make a backup of the original database using pg_dump and restores it to the newly created database.

## Prerequisites

To run this script, you will need:

- **Rust**: You can install Rust from the official website [here](https://www.rust-lang.org/tools/install).
- **PostgreSQL**: You should have PostgreSQL installed on your system and have the necessary permissions to perform database operations. You can download PostgreSQL from the official website [here](https://www.postgresql.org/download/).

```rs
cargo build
cargo run
```

## Usage

1- Set up your environment variables. You will need to specify:

- `ORIGIN_DATABASE_URL`: The connection string for the source database.
- `DESTINATION_DATABASE_URL`: The connection string for the destination database.
- `DATABASE_NAME`: The name of the database.
  Create a `.env` file in the root of your project and add these variables like so:

  ```
  ORIGIN_DATABASE_URL=your_origin_database_url
  DESTINATION_DATABASE_URL=your_destination_database_url
  DATABASE_NAME=your_database_name
  ```

2- Use `cargo` to build and run the script:

    ```
    cargo build
    cargo run
    ```

## Dependencies

This script use `tokio`, `tokio-postgres`, `dotenv`, and `postgres` Rust crates.

## Disclaimer

This script is intended for use as is, and does not come with any warranties. Always test scripts in a controlled environment before using them in production.
