use std::path::PathBuf;
use std::time::Instant;

use chrono::{Datelike, NaiveDate};
use serde::{Deserialize, Serialize};

// ── Serde types (matching calce-data, for bincode loading) ──────────────

#[derive(Debug, Serialize, Deserialize)]
struct CachedMarketData {
    metadata: CacheMetadata,
    prices: Vec<CachedPrice>,
    fx_rates: Vec<CachedFxRate>,
    instruments: Vec<CachedInstrument>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CacheMetadata {
    fetched_at: chrono::DateTime<chrono::Utc>,
    date_from: NaiveDate,
    date_to: NaiveDate,
    price_count: usize,
    fx_rate_count: usize,
    instrument_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct CachedPrice {
    ticker: String,
    date: NaiveDate,
    close: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct CachedFxRate {
    from: String,
    to: String,
    date: NaiveDate,
    rate: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct CachedInstrument {
    ticker: String,
    currency: Option<String>,
    name: Option<String>,
    isin: Option<String>,
    instrument_type: Option<String>,
}

// ── Rkyv types (mirror with primitive date representations) ─────────────

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Debug)]
struct RkyvMarketData {
    fetched_at_millis: i64,
    date_from_days: i32,
    date_to_days: i32,
    price_count: u64,
    fx_rate_count: u64,
    instrument_count: u64,
    prices: Vec<RkyvPrice>,
    fx_rates: Vec<RkyvFxRate>,
    instruments: Vec<RkyvInstrument>,
}

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Debug)]
struct RkyvPrice {
    ticker: String,
    date_days: i32,
    close: f64,
}

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Debug)]
struct RkyvFxRate {
    from: String,
    to: String,
    date_days: i32,
    rate: f64,
}

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Debug)]
struct RkyvInstrument {
    ticker: String,
    currency: Option<String>,
    name: Option<String>,
    isin: Option<String>,
    instrument_type: Option<String>,
}

// ── Conversion ──────────────────────────────────────────────────────────

fn date_to_days(d: NaiveDate) -> i32 {
    d.num_days_from_ce()
}

fn convert_to_rkyv(data: &CachedMarketData) -> RkyvMarketData {
    RkyvMarketData {
        fetched_at_millis: data.metadata.fetched_at.timestamp_millis(),
        date_from_days: date_to_days(data.metadata.date_from),
        date_to_days: date_to_days(data.metadata.date_to),
        price_count: data.metadata.price_count as u64,
        fx_rate_count: data.metadata.fx_rate_count as u64,
        instrument_count: data.metadata.instrument_count as u64,
        prices: data
            .prices
            .iter()
            .map(|p| RkyvPrice {
                ticker: p.ticker.clone(),
                date_days: date_to_days(p.date),
                close: p.close,
            })
            .collect(),
        fx_rates: data
            .fx_rates
            .iter()
            .map(|r| RkyvFxRate {
                from: r.from.clone(),
                to: r.to.clone(),
                date_days: date_to_days(r.date),
                rate: r.rate,
            })
            .collect(),
        instruments: data
            .instruments
            .iter()
            .map(|i| RkyvInstrument {
                ticker: i.ticker.clone(),
                currency: i.currency.clone(),
                name: i.name.clone(),
                isin: i.isin.clone(),
                instrument_type: i.instrument_type.clone(),
            })
            .collect(),
    }
}

// ── Helpers ─────────────────────────────────────────────────────────────

fn fmt_size(bytes: u64) -> String {
    if bytes >= 1_073_741_824 {
        format!("{:.2} GB", bytes as f64 / 1_073_741_824.0)
    } else if bytes >= 1_048_576 {
        format!("{:.2} MB", bytes as f64 / 1_048_576.0)
    } else {
        format!("{:.2} KB", bytes as f64 / 1024.0)
    }
}

fn fmt_duration(d: std::time::Duration) -> String {
    if d.as_secs() > 0 {
        format!("{:.3}s", d.as_secs_f64())
    } else {
        format!("{:.1}ms", d.as_secs_f64() * 1000.0)
    }
}

// ── Row for summary table ───────────────────────────────────────────────

struct Row {
    label: &'static str,
    bincode: String,
    rkyv: String,
    rkyv_lz4: String,
    rkyv_zstd: String,
}

fn print_table(rows: &[Row]) {
    println!(
        "  {:>32} {:>12} {:>12} {:>12} {:>12}",
        "", "bincode", "rkyv", "rkyv+lz4", "rkyv+zstd"
    );
    for row in rows {
        println!(
            "  {:>32} {:>12} {:>12} {:>12} {:>12}",
            row.label, row.bincode, row.rkyv, row.rkyv_lz4, row.rkyv_zstd,
        );
    }
}

// ── Main ────────────────────────────────────────────────────────────────

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // `cache-bench convert` mode: one-off migration from bincode → rkyv+lz4
    if args.get(1).map(|s| s.as_str()) == Some("convert") {
        convert_cache();
        return;
    }

    let cache_path = args.get(1).map(PathBuf::from).unwrap_or_else(|| {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../.cache/njorda/njorda_market_data.bin")
    });

    println!("Cache file: {}", cache_path.display());
    let file_size = std::fs::metadata(&cache_path)
        .expect("cache file not found")
        .len();
    println!("File size:  {}\n", fmt_size(file_size));

    // ── Bincode ─────────────────────────────────────────────────────────

    println!("=== BINCODE (current) ===");

    let t = Instant::now();
    let bytes = std::fs::read(&cache_path).expect("failed to read cache file");
    let bc_read_time = t.elapsed();
    println!("  File read:         {}", fmt_duration(bc_read_time));

    let t = Instant::now();
    let config = bincode::config::standard();
    let (data, _): (CachedMarketData, _) =
        bincode::serde::decode_from_slice(&bytes, config).expect("bincode decode failed");
    let bc_deser_time = t.elapsed();
    let bc_load_deser = bc_read_time + bc_deser_time;
    println!("  Deserialize:       {}", fmt_duration(bc_deser_time));
    println!("  Load+Deserialize:  {}", fmt_duration(bc_load_deser));
    println!(
        "  Records: {} prices, {} fx_rates, {} instruments",
        data.prices.len(),
        data.fx_rates.len(),
        data.instruments.len(),
    );

    let t = Instant::now();
    let bc_bytes = bincode::serde::encode_to_vec(&data, config).expect("bincode encode failed");
    let bc_ser_time = t.elapsed();
    println!("  Serialize:         {}", fmt_duration(bc_ser_time));
    println!("  Encoded size:      {}", fmt_size(bc_bytes.len() as u64));

    // ── Convert to rkyv types ───────────────────────────────────────────

    let t = Instant::now();
    let rkyv_data = convert_to_rkyv(&data);
    let convert_time = t.elapsed();

    drop(bytes);
    drop(bc_bytes);
    drop(data);

    // ── Rkyv: serialize ─────────────────────────────────────────────────

    println!("\n=== RKYV (uncompressed) ===");
    println!("  Conversion:        {}", fmt_duration(convert_time));

    let t = Instant::now();
    let rkyv_bytes =
        rkyv::to_bytes::<rkyv::rancor::Error>(&rkyv_data).expect("rkyv serialize failed");
    let rkyv_ser_time = t.elapsed();
    let rkyv_size = rkyv_bytes.len() as u64;
    println!("  Serialize:         {}", fmt_duration(rkyv_ser_time));
    println!("  Encoded size:      {}", fmt_size(rkyv_size));

    // Write uncompressed rkyv
    let rkyv_path = cache_path.with_extension("rkyv");
    let t = Instant::now();
    std::fs::write(&rkyv_path, &rkyv_bytes).expect("failed to write rkyv file");
    let rkyv_write_time = t.elapsed();
    println!("  File write:        {}", fmt_duration(rkyv_write_time));

    // ── Rkyv + LZ4 ─────────────────────────────────────────────────────

    println!("\n=== RKYV + LZ4 ===");

    let t = Instant::now();
    let lz4_bytes = lz4_flex::compress_prepend_size(&rkyv_bytes);
    let lz4_compress_time = t.elapsed();
    let lz4_size = lz4_bytes.len() as u64;
    println!("  Compress:          {}", fmt_duration(lz4_compress_time));
    println!(
        "  Compressed size:   {} ({:.1}% of rkyv)",
        fmt_size(lz4_size),
        lz4_size as f64 / rkyv_size as f64 * 100.0,
    );

    let lz4_path = cache_path.with_extension("rkyv.lz4");
    let t = Instant::now();
    std::fs::write(&lz4_path, &lz4_bytes).expect("failed to write lz4 file");
    let lz4_write_time = t.elapsed();
    println!("  File write:        {}", fmt_duration(lz4_write_time));
    println!(
        "  Ser+Compress+Write:{}",
        fmt_duration(rkyv_ser_time + lz4_compress_time + lz4_write_time),
    );
    drop(lz4_bytes);

    // ── Rkyv + Zstd (level 1 — fast) ───────────────────────────────────

    println!("\n=== RKYV + ZSTD (level 1) ===");

    let t = Instant::now();
    let zstd_bytes = zstd::encode_all(rkyv_bytes.as_ref(), 1).expect("zstd compress failed");
    let zstd_compress_time = t.elapsed();
    let zstd_size = zstd_bytes.len() as u64;
    println!("  Compress:          {}", fmt_duration(zstd_compress_time));
    println!(
        "  Compressed size:   {} ({:.1}% of rkyv)",
        fmt_size(zstd_size),
        zstd_size as f64 / rkyv_size as f64 * 100.0,
    );

    let zstd_path = cache_path.with_extension("rkyv.zst");
    let t = Instant::now();
    std::fs::write(&zstd_path, &zstd_bytes).expect("failed to write zstd file");
    let zstd_write_time = t.elapsed();
    println!("  File write:        {}", fmt_duration(zstd_write_time));
    println!(
        "  Ser+Compress+Write:{}",
        fmt_duration(rkyv_ser_time + zstd_compress_time + zstd_write_time),
    );
    drop(zstd_bytes);
    drop(rkyv_bytes);
    drop(rkyv_data);

    // ── Load benchmarks ─────────────────────────────────────────────────

    println!("\n=== LOAD BENCHMARKS ===");

    // Rkyv uncompressed: load + zero-copy access
    let t = Instant::now();
    let rkyv_bytes = std::fs::read(&rkyv_path).expect("failed to read rkyv file");
    let rkyv_read_time = t.elapsed();

    let t = Instant::now();
    let archived =
        rkyv::access::<ArchivedRkyvMarketData, rkyv::rancor::Error>(&rkyv_bytes)
            .expect("rkyv validation failed");
    let rkyv_access_time = t.elapsed();
    let rkyv_load_access = rkyv_read_time + rkyv_access_time;
    println!(
        "  rkyv     — Read: {} | Access: {} | Total: {}",
        fmt_duration(rkyv_read_time),
        fmt_duration(rkyv_access_time),
        fmt_duration(rkyv_load_access),
    );

    // Rkyv: full deserialize to owned
    let t = Instant::now();
    let _deserialized: RkyvMarketData =
        rkyv::deserialize::<RkyvMarketData, rkyv::rancor::Error>(archived)
            .expect("rkyv deserialize failed");
    let rkyv_deser_time = t.elapsed();
    let rkyv_load_deser = rkyv_read_time + rkyv_access_time + rkyv_deser_time;
    println!(
        "  rkyv     — Read+Access+Deser (owned): {}",
        fmt_duration(rkyv_load_deser),
    );
    drop(_deserialized);
    drop(rkyv_bytes);

    // Rkyv + LZ4: load + decompress + zero-copy access
    let t = Instant::now();
    let lz4_bytes = std::fs::read(&lz4_path).expect("failed to read lz4 file");
    let lz4_read_time = t.elapsed();

    let t = Instant::now();
    let decompressed = lz4_flex::decompress_size_prepended(&lz4_bytes).expect("lz4 decompress failed");
    let lz4_decompress_time = t.elapsed();
    drop(lz4_bytes);

    let t = Instant::now();
    let _archived =
        rkyv::access::<ArchivedRkyvMarketData, rkyv::rancor::Error>(&decompressed)
            .expect("rkyv validation failed");
    let lz4_access_time = t.elapsed();
    let lz4_load_access = lz4_read_time + lz4_decompress_time + lz4_access_time;
    println!(
        "  rkyv+lz4 — Read: {} | Decompress: {} | Access: {} | Total: {}",
        fmt_duration(lz4_read_time),
        fmt_duration(lz4_decompress_time),
        fmt_duration(lz4_access_time),
        fmt_duration(lz4_load_access),
    );
    drop(decompressed);

    // Rkyv + Zstd: load + decompress + zero-copy access
    let t = Instant::now();
    let zstd_bytes = std::fs::read(&zstd_path).expect("failed to read zstd file");
    let zstd_read_time = t.elapsed();

    let t = Instant::now();
    let decompressed = zstd::decode_all(zstd_bytes.as_slice()).expect("zstd decompress failed");
    let zstd_decompress_time = t.elapsed();
    drop(zstd_bytes);

    let t = Instant::now();
    let _archived =
        rkyv::access::<ArchivedRkyvMarketData, rkyv::rancor::Error>(&decompressed)
            .expect("rkyv validation failed");
    let zstd_access_time = t.elapsed();
    let zstd_load_access = zstd_read_time + zstd_decompress_time + zstd_access_time;
    println!(
        "  rkyv+zst — Read: {} | Decompress: {} | Access: {} | Total: {}",
        fmt_duration(zstd_read_time),
        fmt_duration(zstd_decompress_time),
        fmt_duration(zstd_access_time),
        fmt_duration(zstd_load_access),
    );
    drop(decompressed);

    // ── Summary table ───────────────────────────────────────────────────

    println!("\n=== SUMMARY ===");
    let rows = vec![
        Row {
            label: "File size:",
            bincode: fmt_size(file_size),
            rkyv: fmt_size(rkyv_size),
            rkyv_lz4: fmt_size(lz4_size),
            rkyv_zstd: fmt_size(zstd_size),
        },
        Row {
            label: "Serialize:",
            bincode: fmt_duration(bc_ser_time),
            rkyv: fmt_duration(rkyv_ser_time),
            rkyv_lz4: fmt_duration(rkyv_ser_time + lz4_compress_time),
            rkyv_zstd: fmt_duration(rkyv_ser_time + zstd_compress_time),
        },
        Row {
            label: "Load+Deser (zero-copy):",
            bincode: "n/a".into(),
            rkyv: fmt_duration(rkyv_load_access),
            rkyv_lz4: fmt_duration(lz4_load_access),
            rkyv_zstd: fmt_duration(zstd_load_access),
        },
        Row {
            label: "Load+Deser (owned):",
            bincode: fmt_duration(bc_load_deser),
            rkyv: fmt_duration(rkyv_load_deser),
            rkyv_lz4: "—".into(),
            rkyv_zstd: "—".into(),
        },
    ];
    print_table(&rows);

    // Clean up temp files
    let _ = std::fs::remove_file(&rkyv_path);
    let _ = std::fs::remove_file(&lz4_path);
    let _ = std::fs::remove_file(&zstd_path);
    println!("\nCleaned up temp files.");
}

fn convert_cache() {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.cache/njorda");
    let old_path = base.join("njorda_market_data.bin");
    let new_path = base.join("njorda_market_data.rkyv.lz4");

    println!("Reading old bincode cache: {}", old_path.display());
    let t = Instant::now();
    let bytes = std::fs::read(&old_path).expect("failed to read old cache");
    let old_size = bytes.len();
    let config = bincode::config::standard();
    let (data, _): (CachedMarketData, _) =
        bincode::serde::decode_from_slice(&bytes, config).expect("bincode decode failed");
    println!(
        "  Loaded in {:.1}s — {} prices, {} fx_rates, {} instruments",
        t.elapsed().as_secs_f64(),
        data.prices.len(),
        data.fx_rates.len(),
        data.instruments.len(),
    );
    drop(bytes);

    println!("Converting to rkyv+lz4...");
    let t = Instant::now();
    let rkyv_data = convert_to_rkyv(&data);
    let rkyv_bytes =
        rkyv::to_bytes::<rkyv::rancor::Error>(&rkyv_data).expect("rkyv serialize failed");
    let compressed = lz4_flex::compress_prepend_size(&rkyv_bytes);
    println!("  Converted in {:.1}s", t.elapsed().as_secs_f64());

    println!("Writing new cache: {}", new_path.display());
    std::fs::write(&new_path, &compressed).expect("failed to write new cache");

    println!("\nDone!");
    println!(
        "  Old (bincode):     {:.2} GB",
        old_size as f64 / 1_073_741_824.0,
    );
    println!(
        "  New (rkyv+lz4):    {:.2} MB",
        compressed.len() as f64 / 1_048_576.0,
    );
    println!(
        "  Reduction:         {:.1}x smaller",
        old_size as f64 / compressed.len() as f64,
    );
}
