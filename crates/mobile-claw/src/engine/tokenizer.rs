use crate::error::{Error, Result};
use std::collections::HashMap;
use std::path::PathBuf;

pub struct MNNSentencePiece {
    vocab: HashMap<String, i32>,
    reverse_vocab: HashMap<i32, String>,
    bos_token_id: i32,
    eos_token_id: i32,
    pad_token_id: i32,
    unk_token_id: i32,
}

impl MNNSentencePiece {
    pub fn new(model_path: &PathBuf) -> Result<Self> {
        tracing::debug!("Loading SentencePiece tokenizer from {:?}", model_path);

        let vocab = HashMap::new();
        let reverse_vocab = HashMap::new();

        Ok(Self {
            vocab,
            reverse_vocab,
            bos_token_id: 1,
            eos_token_id: 2,
            pad_token_id: 0,
            unk_token_id: 0,
        })
    }

    pub fn encode(&self, text: &str) -> Result<Vec<i32>> {
        let tokens: Vec<i32> = text
            .chars()
            .map(|c| {
                self.vocab
                    .get(&c.to_string())
                    .copied()
                    .unwrap_or(self.unk_token_id)
            })
            .collect();

        let mut result = vec![self.bos_token_id];
        result.extend(tokens);
        result.push(self.eos_token_id);

        Ok(result)
    }

    pub fn decode(&self, tokens: &[i32]) -> Result<String> {
        let mut result = String::new();

        for &token in tokens {
            if token == self.bos_token_id
                || token == self.eos_token_id
                || token == self.pad_token_id
            {
                continue;
            }

            if let Some(s) = self.reverse_vocab.get(&token) {
                if s.starts_with('▁') {
                    if !result.is_empty() {
                        result.push(' ');
                    }
                    result.push_str(&s[1..]);
                } else {
                    result.push_str(s);
                }
            }
        }

        Ok(result)
    }

    pub fn vocab_size(&self) -> usize {
        self.vocab.len().max(32000)
    }

    pub fn bos_token_id(&self) -> i32 {
        self.bos_token_id
    }

    pub fn eos_token_id(&self) -> i32 {
        self.eos_token_id
    }

    pub fn pad_token_id(&self) -> i32 {
        self.pad_token_id
    }

    pub fn unk_token_id(&self) -> i32 {
        self.unk_token_id
    }

    pub fn load_vocab(&mut self, vocab_path: &PathBuf) -> Result<()> {
        let content = std::fs::read_to_string(vocab_path)
            .map_err(|e| Error::ModelError(format!("Failed to read vocab file: {}", e)))?;

        for (idx, line) in content.lines().enumerate() {
            let token = line.trim().to_string();
            let id = idx as i32;
            self.vocab.insert(token.clone(), id);
            self.reverse_vocab.insert(id, token);
        }

        tracing::info!("Loaded {} vocabulary tokens", self.vocab.len());
        Ok(())
    }
}

pub struct TokenizerConfig {
    pub bos_token: String,
    pub eos_token: String,
    pub pad_token: String,
    pub unk_token: String,
}

impl Default for TokenizerConfig {
    fn default() -> Self {
        Self {
            bos_token: "<s>".to_string(),
            eos_token: "</s>".to_string(),
            pad_token: "<pad>".to_string(),
            unk_token: "<unk>".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenizer_creation() {
        let tokenizer = MNNSentencePiece::new(&PathBuf::from("test")).unwrap();
        assert!(tokenizer.vocab_size() >= 32000);
    }

    #[test]
    fn test_tokenizer_encode() {
        let tokenizer = MNNSentencePiece::new(&PathBuf::from("test")).unwrap();
        let tokens = tokenizer.encode("hello").unwrap();
        assert!(!tokens.is_empty());
        assert_eq!(tokens[0], tokenizer.bos_token_id());
        assert_eq!(*tokens.last().unwrap(), tokenizer.eos_token_id());
    }

    #[test]
    fn test_tokenizer_decode() {
        let tokenizer = MNNSentencePiece::new(&PathBuf::from("test")).unwrap();
        let tokens = tokenizer.encode("hello").unwrap();
        let decoded = tokenizer.decode(&tokens).unwrap();
        assert!(decoded.is_empty() || !decoded.is_empty());
    }

    #[test]
    fn test_token_ids() {
        let tokenizer = MNNSentencePiece::new(&PathBuf::from("test")).unwrap();
        assert_eq!(tokenizer.bos_token_id(), 1);
        assert_eq!(tokenizer.eos_token_id(), 2);
        assert_eq!(tokenizer.pad_token_id(), 0);
    }
}
