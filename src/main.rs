use dotenv::dotenv;
use postgres::NoTls;
use std::path::Path;
use std::{env, fs};

const ORIGIN_DATABASE_URL: &str = "ORIGIN_DATABASE_URL";
const DESTINATION_DATABASE_URL: &str = "DESTINATION_DATABASE_URL";
const DATABASE_NAME: &str = "DATABASE_NAME";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().expect("Failed to load .env file");

    let conn_string_origin = env::var(ORIGIN_DATABASE_URL)
        .expect("Origin: environment variable ORIGIN_DATABASE_URL not set");
    let conn_string_destination = env::var(DESTINATION_DATABASE_URL)
        .expect("Destination: environment variable DESTINATION_DATABASE_URL not set");
    let database_name: String =
        env::var(DATABASE_NAME).expect("environment variable DATABASE_NAME not set");

    let (_, conn_origin) = tokio_postgres::connect(conn_string_origin.as_str(), NoTls).await?;

    tokio::spawn(async move {
        if let Err(e) = conn_origin.await {
            eprintln!("Origin connection error: {}", e);
        }
    });
    eprintln!("Origin connection success");

    let (client_destination, conn_destination) =
        tokio_postgres::connect(conn_string_destination.as_str(), NoTls).await?;

    tokio::spawn(async move {
        if let Err(e) = conn_destination.await {
            eprintln!("Destination connection error: {}", e);
        }
    });
    eprintln!("Destination connection success");

    // Check if the destination database already exists
    let rows = client_destination
        .query(
            format!(
                "SELECT datname FROM pg_database WHERE datistemplate = false AND datname = '{}'",
                database_name
            )
            .as_str(),
            &[],
        )
        .await?;
    let consumed_rows: Vec<_> = rows.into_iter().collect();

    if !consumed_rows.is_empty() {
        client_destination
            .execute(
                format!(
                    "SELECT pg_terminate_backend(pg_stat_activity.pid)
                     FROM pg_stat_activity
                     WHERE pg_stat_activity.datname = '{}' AND pid <> pg_backend_pid()",
                    database_name
                )
                .as_str(),
                &[],
            )
            .await?;

        client_destination
            .execute(format!("DROP DATABASE {}", database_name).as_str(), &[])
            .await?;
        eprintln!("Drop database: {}", database_name);
    }

    // Create the destination database
    client_destination
        .execute(format!("CREATE DATABASE {}", database_name).as_str(), &[])
        .await?;
    eprintln!("Recreated database: {}", database_name);

    // Create a backup of the entire original database
    let file_path = "/tmp/database.sql";
    if Path::new(file_path).exists() {
        fs::remove_file(file_path)?;
    }

    let backup_sql: String = format!(
        "pg_dump -F p --inserts -f {} {}/{}",
        file_path, conn_string_origin, database_name
    );
    let output_origin = std::process::Command::new("sh")
        .arg("-c")
        .arg(&backup_sql)
        .output()?;

    let file_metadata = fs::metadata(&file_path)?;

    if !file_metadata.is_file() && !output_origin.status.success() {
        panic!("pg_dump failed: {:?}", output_origin);
    }
    eprintln!(
        "Destination: pg_dump executed success, pg_dump_status: {}, file length: {}",
        output_origin.status.to_string(),
        file_metadata.len()
    );

    // Restore the backup to the new database
    eprintln!("Restore origin backup");
    let restore_sql = vec![
        "DROP SCHEMA IF EXISTS public CASCADE;",
        "CREATE SCHEMA public;",
        "SET search_path TO public;",
    ]
    .join("");
    client_destination.batch_execute(&restore_sql).await?;
    drop(client_destination);

    let (db_client_destination, db_conn_destination) = tokio_postgres::connect(
        format!("{}/{}", conn_string_destination, database_name).as_str(),
        NoTls,
    )
    .await?;
    tokio::spawn(async move {
        if let Err(e) = db_conn_destination.await {
            eprintln!(
                "Destination database {} connection error: {}",
                database_name, e
            );
        }
    });

    let sql_contents = fs::read_to_string(file_path)?;
    db_client_destination.batch_execute(&sql_contents).await?;

    eprintln!("Finish");

    Ok(())
}
