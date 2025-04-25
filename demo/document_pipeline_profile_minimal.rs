/*[toml]
[dependencies]
backtrace = "0.3"
# thag_profiler = { git = "https://github.com/durbanlegend/thag_rs", branch = "develop", features = ["full_profiling"] }
# thag_profiler = { version = "0.1", features = ["full_profiling"] }
thag_profiler = { path = "/Users/donf/projects/thag_rs/thag_profiler", features=["full_profiling"] }
tokio = { version = "1", features = ["full"] }

[profile.dev]
debug-assertions = true
*/

/// Test async program (minimalist instrumented version) for `thag_profiler` debugging.
/// See also `demo/document_pipeline.rs` and `demo/document_pipeline_profile.rs`.
///
//# Purpose: Test and debug profiling using `thag_profiler`.
//# Categories: prototype, testing
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::time::sleep;

use thag_profiler::{
    self, /*, disable_profiling*/
    enable_profiling, end, profile, profiled, /*, ProfileType */
};

struct Document {
    id: usize,
    content: String,
    word_count: HashMap<String, usize>,
    sentiment_score: f64,
    is_processed: bool,
}

impl Document {
    #[profiled(imp = "Document")]
    fn new(id: usize, content: String) -> Self {
        // Fixed duration for predictability
        std::thread::sleep(Duration::from_millis(10));

        let _create_something = vec![
            "Hello".to_string(),
            "world".to_string(),
            "testing".to_string(),
            "testing".to_string(),
        ];

        Document {
            id,
            content,
            word_count: HashMap::new(),
            sentiment_score: 0.0,
            is_processed: false,
        }
    }

    // #[profiled]
    #[profiled(both, mem_detail)]
    fn count_words(&mut self) {
        // Simulate CPU-intensive operation with fixed duration
        std::thread::sleep(Duration::from_millis(20));

        let words = self.content.split_whitespace();
        for word in words {
            let word = word
                .to_lowercase()
                .chars()
                .filter(|c| c.is_alphabetic())
                .collect::<String>();

            if !word.is_empty() {
                *self.word_count.entry(word).or_insert(0) += 1;
            }
        }
    }

    #[profiled]
    fn calculate_sentiment(&mut self) -> f64 {
        // Fixed duration for predictability
        std::thread::sleep(Duration::from_millis(30));

        let positive_words = ["good", "great", "excellent", "happy", "positive"];
        let negative_words = ["bad", "awful", "terrible", "sad", "negative"];

        let mut score = 0.0;

        for (word, count) in &self.word_count {
            if positive_words.contains(&word.as_str()) {
                score += 1.0 * *count as f64;
            } else if negative_words.contains(&word.as_str()) {
                score -= 1.0 * *count as f64;
            }
        }

        // Normalize
        let total_words: usize = self.word_count.values().sum();
        if total_words > 0 {
            score /= total_words as f64;
        }

        let _create_something = vec![
            "Hello".to_string(),
            "world".to_string(),
            "testing".to_string(),
            "testing".to_string(),
        ];

        self.sentiment_score = score;
        score
    }
}

#[profiled]
async fn fetch_document(id: usize) -> Document {
    // Fixed async delay
    sleep(Duration::from_millis(40)).await;

    // Generate deterministic content
    let content = format!(
        "This is document {} with test content. It has good and bad words.",
        id
    );

    Document::new(id, content)
}

#[profiled]
async fn process_document(mut doc: Document) -> Document {
    // Process document with fixed timing
    doc.count_words();
    doc.calculate_sentiment();

    // Small async delay
    profile!("delay", both, async_fn);
    let _dummy = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    sleep(Duration::from_millis(15)).await;
    end!("delay");

    doc.is_processed = true;
    doc
}

#[profiled]
async fn generate_and_process_documents(count: usize) -> Vec<Document> {
    // Process documents one by one to make tracing easier
    let mut documents = Vec::with_capacity(count);

    for id in 0..count {
        let doc = fetch_document(id).await;
        let processed_doc = process_document(doc).await;
        documents.push(processed_doc);
    }

    documents
}

#[profiled]
async fn run_batch(count: usize) {
    // Fixed duration for predictability
    println!(
        "is_profiling_enabled()? {}, get_global_profile_type(): {:?}",
        thag_profiler::is_profiling_enabled(),
        thag_profiler::get_global_profile_type()
    );

    let start = Instant::now();
    let docs = generate_and_process_documents(count).await;

    println!(
        "Processed {} documents in {:?}",
        docs.len(),
        start.elapsed()
    );

    // Print results for verification
    profile!("print_docs", time, mem_summary, async_fn, unbounded);
    for doc in &docs {
        let _dummy = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        // Small async delay
        // profile!("print_docs", time, mem_detail, async_fn);
        sleep(Duration::from_millis(150)).await;

        println!(
            "Doc #{}: Word count: {}, Sentiment: {:.2}",
            doc.id,
            doc.word_count.len(),
            doc.sentiment_score
        );
    }
    // end!("print_docs");
}

#[tokio::main]
#[cfg_attr(debug_assertions, enable_profiling(runtime))]
async fn main() {
    println!(
        "thag_profiler::PROFILING_MUTEX.is_locked()? {}",
        thag_profiler::PROFILING_MUTEX.is_locked()
    );

    println!("Starting simplified document processing example");

    // Only process small batches of different sizes for easy tracing
    run_batch(3).await;

    // println!("Switching profiling off");
    // disable_profiling();

    // profile!("second_batch", global, async_fn);
    profile!("second_batch", time, mem_summary, async_fn);
    // Only process small batches of documents for easy tracing
    run_batch(2).await;

    // println!("Switching only time profiling back on");
    // enable_profiling(true, Some(ProfileType::Time)).unwrap();
    end!("second_batch");

    profile!("last_batch", time, mem_summary, async_fn);
    // Only process small batches of documents for easy tracing
    run_batch(1).await;
    end!("last_batch");

    println!("Profiling data written to folded files in current directory");

    println!(
        "thag_profiler::PROFILING_MUTEX.is_locked()? {}",
        thag_profiler::PROFILING_MUTEX.is_locked()
    );
}
