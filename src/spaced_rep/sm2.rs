/// SM-2 Spaced Repetition Algorithm Implementation
/// Based on the SuperMemo 2 algorithm by Piotr Wozniak

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Quality {
    CompleteBlackout = 0,
    Incorrect = 1,
    IncorrectButRemembered = 2,
    CorrectWithDifficulty = 3,
    Perfect = 4,
}

#[derive(Debug, Clone)]
pub struct SM2Item {
    /// Repetition number (0 = first time, 1 = first repetition, etc.)
    pub repetition: u32,
    /// Interval in days
    pub interval: f64,
    /// Ease factor (difficulty multiplier)
    pub ease_factor: f64,
    /// Last quality rating (0-4)
    pub quality: u8,
    /// Next review date (Unix timestamp)
    pub next_review: u64,
}

impl Default for SM2Item {
    fn default() -> Self {
        Self {
            repetition: 0,
            interval: 1.0,
            ease_factor: 2.5,
            quality: 0,
            next_review: 0,
        }
    }
}

pub struct SM2Algorithm;

impl SM2Algorithm {
    /// Calculate the next review parameters based on the quality of recall
    ///
    /// # Arguments
    /// * `item` - The SM2 item to update
    /// * `quality` - Quality of response (0-4)
    /// * `current_time` - Current Unix timestamp
    ///
    /// # Returns
    /// Updated SM2 item
    pub fn calculate_next_review(item: SM2Item, quality: Quality, current_time: u64) -> SM2Item {
        let q = quality as u8;

        let (new_interval, new_repetition, new_ease_factor) = if q < 3 {
            // If quality is below 3, start over
            (1.0, 0, item.ease_factor.max(1.3))
        } else {
            // Correct response
            let interval = if item.repetition == 0 {
                1.0
            } else if item.repetition == 1 {
                6.0
            } else {
                item.interval * item.ease_factor
            };

            let ease_factor = item.ease_factor
                + (0.1 - (5.0 - q as f64) * (0.08 + (5.0 - q as f64) * 0.02));

            let ease_factor = ease_factor.max(1.3);

            (interval, item.repetition + 1, ease_factor)
        };

        // Calculate next review date (interval is in days)
        let next_review = current_time + (new_interval as u64 * 24 * 60 * 60);

        SM2Item {
            repetition: new_repetition,
            interval: new_interval,
            ease_factor: new_ease_factor,
            quality: q,
            next_review,
        }
    }

    /// Check if an item is due for review
    pub fn is_due(item: &SM2Item, current_time: u64) -> bool {
        current_time >= item.next_review
    }

    /// Get items sorted by priority (most overdue first)
    pub fn get_review_priority<'a>(items: &'a [SM2Item], current_time: u64) -> Vec<&'a SM2Item> {
        let mut due_items: Vec<&SM2Item> = items
            .iter()
            .filter(|item| Self::is_due(item, current_time))
            .collect();

        due_items.sort_by(|a, b| a.next_review.cmp(&b.next_review));
        due_items
    }
}

pub struct SpacedRepetition {
    items: Vec<SM2Item>,
}

impl SpacedRepetition {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
        }
    }

    /// Add a new item to the review queue
    pub fn add_item(&mut self) -> usize {
        let id = self.items.len();
        self.items.push(SM2Item::default());
        id
    }

    /// Get items due for review
    pub fn get_due_items(&self, current_time: u64) -> Vec<usize> {
        self.items
            .iter()
            .enumerate()
            .filter(|(_, item)| SM2Algorithm::is_due(item, current_time))
            .map(|(id, _)| id)
            .collect()
    }

    /// Review an item
    pub fn review(&mut self, item_id: usize, quality: Quality, current_time: u64) -> Option<SM2Item> {
        if item_id < self.items.len() {
            let updated = SM2Algorithm::calculate_next_review(
                self.items[item_id].clone(),
                quality,
                current_time,
            );
            self.items[item_id] = updated.clone();
            Some(updated)
        } else {
            None
        }
    }

    /// Get the total number of items
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

impl Default for SpacedRepetition {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_item() {
        let item = SM2Item::default();
        assert_eq!(item.repetition, 0);
        assert_eq!(item.interval, 1.0);
        assert_eq!(item.ease_factor, 2.5);
    }

    #[test]
    fn test_first_review_perfect() {
        let item = SM2Item::default();
        let current_time = 1000;
        let updated = SM2Algorithm::calculate_next_review(item, Quality::Perfect, current_time);

        assert_eq!(updated.repetition, 1);
        assert_eq!(updated.interval, 1.0);
        assert!(updated.next_review > current_time);
    }

    #[test]
    fn test_failed_review() {
        let item = SM2Item::default();
        let current_time = 1000;
        let updated = SM2Algorithm::calculate_next_review(item, Quality::Incorrect, current_time);

        assert_eq!(updated.repetition, 0);
        assert_eq!(updated.interval, 1.0);
        assert!(updated.next_review > current_time);
    }

    #[test]
    fn test_is_due() {
        let item = SM2Item::default();
        assert!(SM2Algorithm::is_due(&item, 0));

        let future_item = SM2Item {
            next_review: 10000,
            ..Default::default()
        };
        assert!(!SM2Algorithm::is_due(&future_item, 5000));
        assert!(SM2Algorithm::is_due(&future_item, 10001));
    }
}
