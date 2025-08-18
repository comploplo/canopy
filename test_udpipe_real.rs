use canopy_parser::udpipe::{UDPipeEngine, UDPipeParser};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Test real UDPipe parsing with actual model
    let model_path = "/Users/gabe/projects/canopy/models/test.model";
    
    println!("Loading UDPipe engine from: {}", model_path);
    let engine = UDPipeEngine::load(model_path)?;
    let parser = UDPipeParser::new_with_engine(engine);
    
    let test_sentences = vec![
        "The cat sat on the mat.",
        "She gave him a book.",
        "John loves Mary.",
        "The quick brown fox jumps over the lazy dog.",
    ];
    
    for sentence in test_sentences {
        println!("\n--- Parsing: {} ---", sentence);
        
        let start = std::time::Instant::now();
        let result = parser.parse_document(sentence)?;
        let duration = start.elapsed();
        
        println!("Parse time: {:?}", duration);
        println!("Sentences: {}", result.sentences.len());
        
        for (i, sent) in result.sentences.iter().enumerate() {
            println!("Sentence {}: {} words", i, sent.words.len());
            for word in &sent.words {
                println!("  {}: {} [{}] -> {} (head: {})", 
                    word.id, word.text, word.upos, word.lemma, word.head);
            }
        }
    }
    
    println!("\nReal UDPipe FFI integration working!");
    Ok(())
}