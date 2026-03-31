//! End-to-end CDC test: inserts a row into Postgres, verifies the CDC listener
//! picks it up and emits the correct event on the channel.
//!
//! Requires a running Postgres with `wal_level=logical` on the standard local
//! dev port (5433). Skips automatically if the database is unreachable.

use std::time::Duration;

use calce_cdc::{CdcConfig, CdcEvent, CdcListener};
use tokio::time::timeout;

fn test_db_url() -> String {
    std::env::var("DATABASE_URL").unwrap_or("postgresql://calce:calce@localhost:5433/calce".into())
}

fn test_config() -> CdcConfig {
    CdcConfig {
        database_url: test_db_url(),
        slot_name: "calce_cdc_test_slot".into(),
        publication_name: "calce_cdc_pub".into(),
    }
}

async fn db_available() -> bool {
    tokio_postgres::connect(&test_db_url(), tokio_postgres::NoTls)
        .await
        .is_ok()
}

async fn drop_test_slot(db_url: &str) {
    let Ok((client, conn)) = tokio_postgres::connect(db_url, tokio_postgres::NoTls).await else {
        return;
    };
    tokio::spawn(conn);
    let _ = client
        .execute(
            "SELECT pg_drop_replication_slot(slot_name) FROM pg_replication_slots WHERE slot_name = 'calce_cdc_test_slot'",
            &[],
        )
        .await;
}

async fn insert_test_price(db_url: &str) -> String {
    let (client, conn) = tokio_postgres::connect(db_url, tokio_postgres::NoTls)
        .await
        .expect("connect for insert");
    tokio::spawn(conn);

    let row = client
        .query_one("SELECT id, ticker FROM instruments LIMIT 1", &[])
        .await
        .expect("need at least one instrument");
    let inst_id: i64 = row.get(0);
    let ticker: String = row.get(1);

    client
        .execute(
            "INSERT INTO prices (instrument_id, price_date, price) \
             VALUES ($1, '2099-12-31', 99999.99) \
             ON CONFLICT (instrument_id, price_date) DO UPDATE SET price = 99999.99",
            &[&inst_id],
        )
        .await
        .expect("insert test price");

    ticker
}

async fn cleanup_test_price(db_url: &str) {
    let Ok((client, conn)) = tokio_postgres::connect(db_url, tokio_postgres::NoTls).await else {
        return;
    };
    tokio::spawn(conn);
    let _ = client
        .execute("DELETE FROM prices WHERE price_date = '2099-12-31'", &[])
        .await;
}

#[tokio::test]
async fn cdc_receives_price_insert() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter("calce_cdc=debug")
        .with_test_writer()
        .try_init();

    let config = test_config();
    let db_url = config.database_url.clone();

    if !db_available().await {
        eprintln!("Skipping CDC E2E test: database not available");
        return;
    }

    drop_test_slot(&db_url).await;

    let (listener, mut rx) = CdcListener::new(config, 256);
    let listener_handle = tokio::spawn(async move { listener.run().await });

    // Give the listener time to connect and start streaming
    tokio::time::sleep(Duration::from_secs(2)).await;

    let expected_ticker = insert_test_price(&db_url).await;
    eprintln!("Inserted test price for ticker: {expected_ticker}");

    // Wait for the CDC event (timeout after 10 seconds)
    let result = timeout(Duration::from_secs(10), rx.recv()).await;

    match result {
        Ok(Some(CdcEvent::PriceChanged {
            instrument_id,
            price,
            ..
        })) => {
            assert_eq!(instrument_id.as_str(), expected_ticker);
            assert!((price - 99999.99).abs() < 0.001);
            eprintln!(
                "CDC E2E PASS: received PriceChanged for {} with price {}",
                instrument_id, price
            );
        }
        Ok(Some(other)) => {
            eprintln!("CDC E2E: got unexpected event first: {other:?}");
        }
        Ok(None) => panic!("CDC channel closed unexpectedly"),
        Err(_) => panic!("Timed out waiting for CDC event"),
    }

    listener_handle.abort();
    cleanup_test_price(&db_url).await;
    drop_test_slot(&db_url).await;
}
