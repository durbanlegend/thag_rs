/*[toml]
[dependencies]
backtrace = "0.3"
# thag_profiler = { git = "https://github.com/durbanlegend/thag_rs", branch = "develop", features = ["full_profiling"] }
# thag_profiler = { version = "0.1", features = ["full_profiling"] }
thag_profiler = { path = "/Users/donf/projects/thag_rs/thag_profiler", features=["full_profiling"] }
tokio = { version = "1", features = ["full"] }
*/

/// Test async program (minimalist instrumented version) for `thag_profiler` debugging.
/// See also `demo/document_pipeline.rs` and `demo/document_pipeline_profile.rs`.
///
//# Purpose: Test and debug profiling using `thag_profiler`.
//# Categories: prototype, testing
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::time::sleep;

use thag_profiler::*;

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

    #[profiled]
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
    sleep(Duration::from_millis(15)).await;

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

#[tokio::main]
#[enable_profiling(runtime)]
async fn main() {
    println!(
        "is_profiling_enabled()? {}, get_global_profile_type(): {:?}",
        thag_profiler::is_profiling_enabled(),
        thag_profiler::get_global_profile_type()
    );
    // Enable profiling manually at the start
    // profiling::enable_profiling(true, ProfileType::Time).unwrap();
    println!("Starting simplified document processing example");

    // Only process 3 documents for easy tracing
    let start = Instant::now();
    let docs = generate_and_process_documents(3).await;

    println!(
        "Processed {} documents in {:?}",
        docs.len(),
        start.elapsed()
    );

    // Print results for verification
    let section = profile!("section::print_docs");
    for doc in &docs {
        // Small async delay
        sleep(Duration::from_millis(15)).await;

        println!(
            "Doc #{}: Word count: {}, Sentiment: {:.2}",
            doc.id,
            doc.word_count.len(),
            doc.sentiment_score
        );
    }
    section.end();

    println!("Profiling data written to folded files in current directory");
}
