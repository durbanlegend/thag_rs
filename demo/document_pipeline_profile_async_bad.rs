/*[toml]
[dependencies]
thag_rs = { path = "/Users/donf/projects/thag_rs", default-features = false, features = ["core", "simplelog", "profiling"] }
tokio = { version = "1", features = ["full"] }
*/

use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::time::sleep;

use thag_rs::{profile, Profile, ProfileType};
use thag_rs::profiling;

struct Document {
    id: usize,
    content: String,
    word_count: HashMap<String, usize>,
    sentiment_score: f64,
    is_processed: bool,
}

impl Document {
    #[profile(imp = "Document")]
    fn new(id: usize, content: String) -> Self {
        // Fixed duration for predictability
        std::thread::sleep(Duration::from_millis(10));
        Document {
            id,
            content,
            word_count: HashMap::new(),
            sentiment_score: 0.0,
            is_processed: false,
        }
    }

    #[profile]
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

    #[profile]
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

        self.sentiment_score = score;
        score
    }
}

#[profile]
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

#[profile]
async fn process_document(mut doc: Document) -> Document {
    // Process document with fixed timing
    doc.count_words();
    doc.calculate_sentiment();

    // Small async delay
    sleep(Duration::from_millis(15)).await;

    doc.is_processed = true;
    doc
}

#[profile]
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
#[profile]
async fn main() {
    // Enable profiling manually at the start
    profiling::enable_profiling(true, ProfileType::Time).unwrap();
    println!("Starting simplified document processing example");
    
    // Only process 3 documents for easy tracing
    let start = Instant::now();
    let docs = generate_and_process_documents(3).await;
    
    println!("Processed {} documents in {:?}", docs.len(), start.elapsed());
    
    // Print results for verification
    for doc in &docs {
        println!(
            "Doc #{}: Word count: {}, Sentiment: {:.2}",
            doc.id,
            doc.word_count.len(),
            doc.sentiment_score
        );
    }
    
    println!("Profiling data written to folded files in current directory");
}