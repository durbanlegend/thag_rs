/*[toml]
[dependencies]
# thag_profiler = { git = "https://github.com/durbanlegend/thag_rs", branch = "develop", features = ["full_profiling"] }
# thag_profiler = { version = "0.1", features = ["full_profiling"] }
thag_profiler = { path = "/Users/donf/projects/thag_rs/thag_profiler", features = ["full_profiling"] }
*/

/// Test async program (instrumented version) for `thag_profiler` testing.
/// See also `demo/document_pipeline.rs` and `demo/document_pipeline_profile_minimal.rs`.
///
//# Purpose: Test profiling using `thag_profiler`.
//# Categories: prototype, testing
use futures::future::join_all;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;
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
    // #[profiled(imp = "Document")]
    #[profiled]
    fn new(id: usize, content: String) -> Self {
        // let _ = sleep(Duration::from_millis(50 + (id % 10 * 5) as u64));
        Document {
            id,
            content,
            word_count: HashMap::new(),
            sentiment_score: 0.0,
            is_processed: false,
        }
    }

    // #[profiled]
    #[profiled(detailed_memory = true)]
    fn count_words(&mut self) {
        // Simulate CPU-intensive operation
        let start = Instant::now();
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

        // Artificial delay to simulate complexity
        while start.elapsed().as_millis() < 10 {
            // Spin wait to consume CPU
            std::hint::spin_loop();
        }
    }

    #[profiled]
    fn calculate_sentiment(&mut self) -> f64 {
        // Very simple "sentiment analysis" - just for demonstration
        let positive_words = ["good", "great", "excellent", "happy", "positive"];
        let negative_words = ["bad", "awful", "terrible", "sad", "negative"];

        let mut score = 0.0;

        // Simulate intensive calculation
        let start = Instant::now();
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

        // Artificial delay
        while start.elapsed().as_millis() < 15 {
            // Spin wait to consume CPU
            std::hint::spin_loop();
        }

        self.sentiment_score = score;
        score
    }

    #[profiled]
    fn summarize(&self) -> String {
        // Generate a simple summary based on most frequent words
        let start = Instant::now();

        let mut word_vec: Vec<(&String, &usize)> = self.word_count.iter().collect();
        word_vec.sort_by(|a, b| b.1.cmp(a.1));

        let summary = word_vec
            .iter()
            .take(5)
            .map(|(word, count)| format!("{} ({})", word, count))
            .collect::<Vec<String>>()
            .join(", ");

        // Artificial delay
        while start.elapsed().as_millis() < 5 {
            // Spin wait
            std::hint::spin_loop();
        }

        format!(
            "Doc #{}: [{}] - Sentiment: {:.2}",
            self.id, summary, self.sentiment_score
        )
    }
}

#[profiled]
async fn fetch_document(id: usize) -> Document {
    // Simulate network delay
    sleep(Duration::from_millis(50 + (id % 10 * 5) as u64)).await;

    // Generate some random content
    let content = format!(
        "This is document {} with some random content. \
                          It contains good words and sometimes bad words. \
                          The quality varies from excellent to terrible \
                          depending on the document.",
        id
    );

    Document::new(id, content)
}

#[profiled]
async fn process_document(mut doc: Document) -> Document {
    // Process document asynchronously
    doc.count_words();
    doc.calculate_sentiment();

    // Simulate some async processing
    sleep(Duration::from_millis(20)).await;

    doc.is_processed = true;
    doc
}

#[profiled]
fn batch_process_documents(documents: &mut [Document]) {
    for doc in documents.iter_mut() {
        doc.count_words();
        doc.calculate_sentiment();
        doc.is_processed = true;
    }
}

#[profiled]
fn analyze_sentiment_distribution(documents: &[Document]) -> HashMap<String, usize> {
    let mut distribution = HashMap::new();

    for doc in documents {
        let sentiment = match doc.sentiment_score {
            s if s > 0.5 => "very_positive",
            s if s > 0.0 => "positive",
            s if s == 0.0 => "neutral",
            s if s > -0.5 => "negative",
            _ => "very_negative",
        };

        *distribution.entry(sentiment.to_string()).or_insert(0) += 1;
    }

    distribution
}

#[profiled]
fn write_reports(documents: &[Document], path: &Path) -> io::Result<()> {
    let mut file = File::create(path)?;

    for doc in documents {
        if doc.is_processed {
            writeln!(file, "{}", doc.summarize())?;
        }
    }

    Ok(())
}

#[allow(dead_code)]
#[profiled]
fn load_documents_from_file(path: &Path) -> io::Result<Vec<Document>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let mut documents = Vec::new();
    let mut current_id = 0;
    let mut current_content = String::new();

    for line in reader.lines() {
        let line = line?;
        if line.starts_with("---") {
            if !current_content.is_empty() {
                documents.push(Document::new(current_id, current_content));
                current_content = String::new();
                current_id += 1;
            }
        } else {
            current_content.push_str(&line);
            current_content.push('\n');
        }
    }

    // Don't forget the last document
    if !current_content.is_empty() {
        documents.push(Document::new(current_id, current_content));
    }

    Ok(documents)
}

#[profiled]
async fn generate_and_process_documents(count: usize) -> Vec<Document> {
    let mut tasks = Vec::new();

    for id in 0..count {
        tasks.push(async move {
            let doc = fetch_document(id).await;
            process_document(doc).await
        });
    }

    join_all(tasks).await
}

#[tokio::main]
#[enable_profiling(runtime)]
async fn main() -> io::Result<()> {
    // Check if profiling is enabled
    println!(
        "PROFILING_FEATURE_ENABLED={}",
        thag_profiler::PROFILING_FEATURE_ENABLED
    );

    // unsafe { backtrace_on_stack_overflow::enable() };

    // Process a batch of documents asynchronously
    let docs = generate_and_process_documents(50).await;

    // Analyze the results
    let sentiment_distribution = analyze_sentiment_distribution(&docs);
    println!("Sentiment distribution: {:?}", sentiment_distribution);

    // Process another batch synchronously for comparison
    let doc_data: Vec<String> = (0..30)
        .map(|i| {
            format!(
                "Document {} with various contents that need processing. \
                Some documents are positive and happy, others are negative \
                and awful. This helps test our sentiment analysis.",
                i
            )
        })
        .collect();

    let mut sync_docs: Vec<Document> = doc_data
        .iter()
        .enumerate()
        .map(|(id, content)| Document::new(id, content.clone()))
        .collect();

    batch_process_documents(&mut sync_docs);

    // Count the total words processed
    let total_words: usize = docs
        .iter()
        .chain(sync_docs.iter())
        .flat_map(|doc| doc.word_count.values())
        .sum();

    println!(
        "Processed {} documents with {} total words",
        docs.len() + sync_docs.len(),
        total_words
    );

    // Write the reports
    write_reports(&docs, Path::new("async_docs_report.txt"))?;
    write_reports(&sync_docs, Path::new("sync_docs_report.txt"))?;

    println!("Reports written successfully");

    Ok(())
}
