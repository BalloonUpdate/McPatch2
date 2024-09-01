use std::collections::LinkedList;
use std::time::SystemTime;

use crate::utils::convert_bytes;

#[derive(Clone, Copy)]
struct Sample {
    bytes: u64,
    timestamp: SystemTime,
}

pub struct SpeedCalculator {
    period: u128,
    frames: LinkedList<Sample>,
}

impl SpeedCalculator {
    pub fn new(sampling_period: u32) -> Self {
        SpeedCalculator {
            period: sampling_period as u128,
            frames: LinkedList::new(),
        }
    }

    pub fn sample_speed2(&self) -> String {
        convert_bytes(self.sample_speed())
    }

    pub fn sample_speed(&self) -> u64 {
        if self.frames.is_empty() {
            return 0;
        }
        
        let now = SystemTime::now();

        let time_span = now.duration_since(self.frames.back().unwrap().timestamp).unwrap().as_millis();

        if time_span > 0 {
            let total_bytes = self.frames.iter().map(|e| e.bytes).sum::<u64>();

            (total_bytes as f64 / time_span as f64 * 1000.0f64)as u64
        } else {
            0
        }
    }

    pub fn feed(&mut self, bytes: usize) {
        let now = SystemTime::now();

        // 连续调用，没有间隔
        if let Some(sample) = self.frames.front_mut() {
            if sample.timestamp == now {
                sample.bytes += bytes as u64;
                return;
            }
        }

        // 记录一帧
        self.frames.push_front(Sample { bytes: bytes as u64, timestamp: now });

        // 清理多余数据
        let mut index = 0;
        let mut invalid = -1i32;
        for frame in &self.frames {
            let diff = now.duration_since(frame.timestamp).unwrap().as_millis();

            if diff > self.period && invalid == -1 {
                invalid = index;
            }

            index += 1;
        }

        if invalid != -1 && index - invalid > 1 {
            self.frames.split_off(invalid as usize);
        }

        assert!(self.frames.len() < 100000)
    }
}