use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Copy, Deserialize, Serialize)]
pub struct NMSaveState {
    pub avg_score: f32,
    pub num_entries: u32,
}

impl NMSaveState {
    pub fn update(&mut self, score: u32) {
        self.avg_score = (self.avg_score * self.num_entries as f32 + score as f32)
            / (self.num_entries + 1) as f32;
        self.num_entries += 1;
    }
}
