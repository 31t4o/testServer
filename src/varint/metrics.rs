use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::time::{Duration, Instant};
use tokio::time;
use prometheus::{Registry, Counter, Gauge, Histogram, IntCounter, labels};
use log::{info, warn};
use std::sync::Arc;

pub struct VarIntMetrics {
    // 基本的な操作カウンター
    encode_count: AtomicU64,
    decode_count: AtomicU64,
    batch_operations: AtomicU64,

    // キャッシュ関連
    cache_hits: AtomicU64,
    cache_misses: AtomicU64,
    cache_size: AtomicUsize,

    // パフォーマンスメトリクス
    encode_latency: Histogram,
    decode_latency: Histogram,

    // エラーカウンター
    encoding_errors: IntCounter,
    decoding_errors: IntCounter,

    // メモリ使用量
    memory_usage: Gauge,

    // スレッド関連
    active_threads: AtomicUsize,

    // Prometheusレジストリ
    registry: Registry,

    // 最後の収集時刻
    last_collection: parking_lot::Mutex<Instant>,
}

impl VarIntMetrics {
    pub fn new() -> Self {
        let registry = Registry::new();

        let encode_latency = Histogram::with_opts(
            prometheus::HistogramOpts::new(
                "varint_encode_latency",
                "VarInt encoding latency in microseconds"
            ).buckets(vec![0.5, 1.0, 2.0, 5.0, 10.0])
        ).unwrap();

        let decode_latency = Histogram::with_opts(
            prometheus::HistogramOpts::new(
                "varint_decode_latency",
                "VarInt decoding latency in microseconds"
            ).buckets(vec![0.5, 1.0, 2.0, 5.0, 10.0])
        ).unwrap();

        registry.register(Box::new(encode_latency.clone())).unwrap();
        registry.register(Box::new(decode_latency.clone())).unwrap();

        Self {
            encode_count: AtomicU64::new(0),
            decode_count: AtomicU64::new(0),
            batch_operations: AtomicU64::new(0),
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
            cache_size: AtomicUsize::new(0),
            encode_latency,
            decode_latency,
            encoding_errors: IntCounter::new("varint_encoding_errors", "Number of encoding errors").unwrap(),
            decoding_errors: IntCounter::new("varint_decoding_errors", "Number of decoding errors").unwrap(),
            memory_usage: Gauge::new("varint_memory_usage", "Memory usage in bytes").unwrap(),
            active_threads: AtomicUsize::new(0),
            registry,
            last_collection: parking_lot::Mutex::new(Instant::now()),
        }
    }

    async fn push_metrics(&self) -> Result<(), Box<dyn std::error::Error>> {
        let metrics = self.registry.gather();
        let client = reqwest::Client::new();

        let mut labels = HashMap::new();
        labels.insert("instance".to_string(), "varint_1".to_string());

        let encoded_metrics = prometheus::TextEncoder::new().encode_to_string(&metrics)?;

        client
            .post("http://localhost:9091/metrics/job/varint_metrics")
            .body(encoded_metrics)
            .send()
            .await?;

        Ok(())
    }


    pub async fn collect_and_report_metrics(&self) {
        let mut last_collection = self.last_collection.lock();
        let elapsed = last_collection.elapsed();
        *last_collection = Instant::now();

        // 基本的な統計情報の収集
        let total_operations = self.encode_count.load(Ordering::Relaxed) +
            self.decode_count.load(Ordering::Relaxed);
        let operations_per_second = total_operations as f64 / elapsed.as_secs_f64();

        // キャッシュヒット率の計算
        let cache_hits = self.cache_hits.load(Ordering::Relaxed);
        let cache_misses = self.cache_misses.load(Ordering::Relaxed);
        let cache_hit_rate = if cache_hits + cache_misses > 0 {
            cache_hits as f64 / (cache_hits + cache_misses) as f64 * 100.0
        } else {
            0.0
        };

        // メモリ使用量の更新
        let current_memory = self.cache_size.load(Ordering::Relaxed) * std::mem::size_of::<usize>();
        self.memory_usage.set(current_memory as f64);

        // レポート生成
        info!("VarInt Metrics Report:");
        info!("Operations per second: {:.2}", operations_per_second);
        info!("Cache hit rate: {:.2}%", cache_hit_rate);
        info!("Active threads: {}", self.active_threads.load(Ordering::Relaxed));
        info!("Memory usage: {} bytes", current_memory);

        // パフォーマンス警告
        if operations_per_second > 10_000.0 {
            warn!("High operation rate detected: {:.2} ops/sec", operations_per_second);
        }
        if cache_hit_rate < 50.0 {
            warn!("Low cache hit rate: {:.2}%", cache_hit_rate);
        }

        // Prometheusメトリクスのエクスポート
        let metrics = self.registry.gather();
        if let Err(e) = self.push_metrics().await {
            warn!("Failed to push metrics: {}", e);
        }
    }

    // メトリクス更新用のヘルパーメソッド
    pub fn record_encode(&self, latency: Duration) {
        self.encode_count.fetch_add(1, Ordering::Relaxed);
        self.encode_latency.observe(latency.as_micros() as f64);
    }

    pub fn record_decode(&self, latency: Duration) {
        self.decode_count.fetch_add(1, Ordering::Relaxed);
        self.decode_latency.observe(latency.as_micros() as f64);
    }

    pub fn record_cache_hit(&self) {
        self.cache_hits.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_cache_miss(&self) {
        self.cache_misses.fetch_add(1, Ordering::Relaxed);
    }

    pub fn update_cache_size(&self, size: usize) {
        self.cache_size.store(size, Ordering::Relaxed);
    }

    pub fn record_thread_start(&self) {
        self.active_threads.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_thread_end(&self) {
        self.active_threads.fetch_sub(1, Ordering::Relaxed);
    }

    pub fn record_encoding_error(&self) {
        self.encoding_errors.inc();
    }

    pub fn record_decoding_error(&self) {
        self.decoding_errors.inc();
    }
}

impl Default for VarIntMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_metrics_collection() {
        let metrics = VarIntMetrics::new();

        // テストデータの生成
        metrics.record_encode(Duration::from_micros(100));
        metrics.record_decode(Duration::from_micros(150));
        metrics.record_cache_hit();
        metrics.record_cache_miss();
        metrics.update_cache_size(1024);

        // メトリクス収集のテスト
        metrics.collect_and_report_metrics().await;

        // 値の検証
        assert_eq!(metrics.encode_count.load(Ordering::Relaxed), 1);
        assert_eq!(metrics.decode_count.load(Ordering::Relaxed), 1);
        assert_eq!(metrics.cache_hits.load(Ordering::Relaxed), 1);
        assert_eq!(metrics.cache_misses.load(Ordering::Relaxed), 1);
        assert_eq!(metrics.cache_size.load(Ordering::Relaxed), 1024);
    }
}