/// Aggregates ASR interim, definite, and final results into a single final transcript.
/// Also collects interim revision history to help identify uncertain segments.
pub struct TranscriptAggregator {
    interim_text: String,
    definite_text: String,
    final_text: String,
    best_text_cache: String,
    has_final: bool,
    has_definite: bool,
    interim_history: Vec<String>,
}

impl TranscriptAggregator {
    pub fn new() -> Self {
        Self {
            interim_text: String::new(),
            definite_text: String::new(),
            final_text: String::new(),
            best_text_cache: String::new(),
            has_final: false,
            has_definite: false,
            interim_history: Vec::new(),
        }
    }

    /// Update with an interim result (replaces previous interim).
    pub fn update_interim(&mut self, text: &str) {
        if !text.is_empty() {
            if self.interim_history.last().map(|s| s.as_str()) != Some(text) {
                self.interim_history.push(text.to_string());
            }
            self.interim_text = text.to_string();
            self.refresh_best_text();
        }
    }

    /// Update with a definite result from two-pass recognition.
    pub fn update_definite(&mut self, text: &str) {
        if !text.is_empty() {
            self.has_definite = true;
            self.definite_text = text.to_string();
            self.refresh_best_text();
            log::info!("definite segment confirmed: {} chars", text.len());
        }
    }

    /// Update with a final result (appends to final text).
    pub fn update_final(&mut self, text: &str) {
        self.has_final = true;
        if !text.is_empty() {
            if !self.final_text.is_empty() {
                self.final_text.push_str(text);
            } else {
                self.final_text = text.to_string();
            }
            self.refresh_best_text();
        }
    }

    /// Get the best available text.
    /// Priority: final > definite > interim.
    pub fn best_text(&self) -> &str {
        &self.best_text_cache
    }

    pub fn has_final_result(&self) -> bool {
        self.has_final
    }

    pub fn has_any_text(&self) -> bool {
        !self.final_text.is_empty()
            || !self.definite_text.is_empty()
            || !self.interim_text.is_empty()
    }

    /// Return the interim revision history.
    /// Keeps only the last `max_entries` to avoid bloating prompts.
    pub fn interim_history(&self, max_entries: usize) -> &[String] {
        let len = self.interim_history.len();
        if len <= max_entries {
            &self.interim_history
        } else {
            &self.interim_history[len - max_entries..]
        }
    }
}

impl TranscriptAggregator {
    fn refresh_best_text(&mut self) {
        self.best_text_cache = if self.has_final && !self.final_text.is_empty() {
            Self::merge_text(&self.final_text, &self.interim_text)
        } else if self.has_definite && !self.definite_text.is_empty() {
            self.definite_text.clone()
        } else {
            self.interim_text.clone()
        };
    }

    fn merge_text(base: &str, candidate: &str) -> String {
        if base.is_empty() {
            return candidate.to_string();
        }
        if candidate.is_empty() {
            return base.to_string();
        }
        if candidate.starts_with(base) {
            return candidate.to_string();
        }
        if base.starts_with(candidate) {
            return base.to_string();
        }

        let mut overlap_start = base.len();
        for (idx, _) in base.char_indices() {
            if candidate.starts_with(&base[idx..]) {
                overlap_start = idx;
                break;
            }
        }

        if overlap_start < base.len() {
            let mut merged = base.to_string();
            merged.push_str(&candidate[base.len() - overlap_start..]);
            merged
        } else {
            base.to_string()
        }
    }
}

impl Default for TranscriptAggregator {
    fn default() -> Self {
        Self::new()
    }
}
