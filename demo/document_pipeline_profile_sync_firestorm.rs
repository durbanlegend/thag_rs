/*[toml]
[dependencies]
firestorm = { version="0.5", features=["enable_system_time"] }
*/

// Test sync program instrumented version for `firestorm` profiling. You can use this script and
/// `demo/document_pipeline_profile_sync.rs` to compare `firestorm` with `thag_profiler`.
/// Use the `-t` flag to get timings.
///
/// Note that `thag_profiler`'s `Individual Sequential Execution Timeline` option is equivalent to `firestorm`'s `Timeline`
/// option, while `thag_profiler`'s `Aggregated Execution Timeline` option is equivalent to `firestorm`'s `Merged` option.
/// `thag_profiler`'s `Show Statistics By Total Time` report is equivalent to  `firestorm`'s `Own Time` option.
///
/// Firestorm does an internal warm-up AFAICS, so runs twice, and therefore almost twice as long. So is it apples with apples?
/// Discuss.
///
/// E.g.:
///
/// `thag demo/document_pipeline_profile_sync_firestorm.rs -t`
///
///
/// See all `demo/document_pipeline*.rs` and in particular `demo/document_pipeline_profile_sync.rs`.
///
//# Purpose: Test profiling using `firestorm`.
//# Categories: prototype, testing
use firestorm::{profile_fn, profile_method};
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;
use std::thread::sleep;
use std::time::{Duration, Instant};

struct Document {
    id: usize,
    content: String,
    word_count: HashMap<String, usize>,
    sentiment_score: f64,
    is_processed: bool,
}

impl Document {
    fn new(id: usize, content: String) -> Self {
        profile_method!(new);
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
    fn count_words(&mut self) {
        profile_method!(count_words);
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

    fn calculate_sentiment(&mut self) -> f64 {
        profile_method!(calculate_sentiment);
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

    fn summarize(&self) -> String {
        profile_method!(summarize);
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

fn fetch_document(id: usize) -> Document {
    profile_fn!(fetch_document);
    // Simulate network delay
    sleep(Duration::from_millis(50 + (id % 10 * 5) as u64));

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

fn process_document(mut doc: Document) -> Document {
    profile_fn!(process_document);
    // Process document asynchronously
    doc.count_words();
    doc.calculate_sentiment();

    sleep(Duration::from_millis(20));

    doc.is_processed = true;
    doc
}

fn batch_process_documents(documents: &mut [Document]) {
    profile_fn!(batch_process_documents);
    for doc in documents.iter_mut() {
        doc.count_words();
        doc.calculate_sentiment();
        doc.is_processed = true;
    }
}

fn analyze_sentiment_distribution(documents: &[Document]) -> HashMap<String, usize> {
    profile_fn!(analyze_sentiment_distribution);
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

fn write_reports(documents: &[Document], path: &Path) -> io::Result<()> {
    profile_fn!(write_reports);
    let mut file = File::create(path)?;

    for doc in documents {
        if doc.is_processed {
            writeln!(file, "{}", doc.summarize())?;
        }
    }

    Ok(())
}

#[allow(dead_code)]
fn load_documents_from_file(path: &Path) -> io::Result<Vec<Document>> {
    profile_fn!(load_documents_from_file);
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

fn generate_and_process_documents(count: usize) -> Vec<Document> {
    profile_fn!(generate_and_process_documents);
    let mut tasks = Vec::new();

    for id in 0..count {
        tasks.push(process_document(fetch_document(id)));
    }

    tasks
}

fn main() {
    let closure = || {
        profile_fn!(main);

        // Process a batch of documents, originally asynchronously
        let docs = generate_and_process_documents(50);

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
        let _ = write_reports(&docs, Path::new("async_docs_report.txt")).unwrap();
        let _ = write_reports(&sync_docs, Path::new("sync_docs_report.txt")).unwrap();

        println!("Reports written successfully");
    };
    if firestorm::enabled() {
        firestorm::bench("./flames/", closure).unwrap();
        // Ok(())
    } else {
        closure();
    }
}
