use std::clone::Clone;
use std::cmp::{Ordering, max};
use std::fmt::Debug;
use std::collections::HashMap;
use std::cmp::Reverse;
use std::time::{SystemTime, UNIX_EPOCH};

use log::info;
use ahash::RandomState;
use priority_queue::core_iterators::Iter;
use rand::{Rng, SeedableRng};
use rand::rngs::SmallRng;
use priority_queue::PriorityQueue;
use pyo3::{
    pyclass, pymethods,
    types::PyList,
    PyObject, PyResult, Python,
    Py
};

use crate::topk::holt;


#[derive(Clone, Debug)]
pub struct Item {
    pub hashkey: u64,
    pub n: u32,
    // In seconds
    pub d: u16,
    // In milliseconds, default to `16_000` ms
    pub last_delta: u16,
    // u64::MAX => 18_446_744_073_709_551_615
    // (1970-01-01, 2025-01-01) => 1_735_668_000_000
    pub last_ts: u64,
}

impl Item {
    pub fn default(default_delta: u16) -> Self {
        Item {
            hashkey: 0,
            n: 0,
            d: 0,
            last_delta: default_delta,
            last_ts: 0,
        }
    }

    fn reset(&mut self, default_delta: u16) {
        self.hashkey = 0;
        self.n = 0;
        self.d = 0;
        self.last_delta = default_delta;
        self.last_ts = 0;
    }
}


#[pyclass]
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Node {
    pub flow_key: String,
    pub m: u32,
    pub to: bool,
}

#[pymethods]
impl Node {
    #[new]
    fn new(flow_key: String, m: u32, to: bool) -> Self {
        Node {flow_key, m, to}
    }

    fn __repr__(&self) -> String {
        format!("Node(flow_key={:?}, m={}, to={}", self.flow_key, self.m, self.to)
    }
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        other.m.cmp(&self.m) // Reverse ordering for min-heap
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}


pub struct MinHeap {
    // pub heap: BinaryHeap<Node>,
    pub heap: PriorityQueue<String, Reverse<u32>>,
    // NOTE: Only for dev
    pub indexer: HashMap<String, (u32, bool)>,
}

impl MinHeap {
    pub fn new() -> Self {
        MinHeap {
            // heap: BinaryHeap::new(),
            heap: PriorityQueue::new(),
            indexer: HashMap::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        MinHeap {
            // heap: BinaryHeap::with_capacity(capacity),
            heap: PriorityQueue::with_capacity(capacity),
            indexer: HashMap::with_capacity(capacity),
        }
    }

    pub fn push(&mut self, node: Node) {
        self.heap.push(node.flow_key.clone(), Reverse(node.m));
        self.indexer.insert(node.flow_key.clone(), (node.m, node.to));
    }

    // Check the smallest `Node`
    pub fn peek(&self) -> u32 {
        match self.heap.peek() {
            Some(v) => {
                v.1.0
            }
            None => {
                0
            }
        }
    }

    // Remove the smallest `Node`
    pub fn pop(&mut self) {
        if let Some(node) = self.heap.pop() {
            self.heap.remove(&node.0);
            self.indexer.remove(&node.0);
        }
    }

    pub fn remove(&mut self, flow_key: &str) {
        if self.heap.get(flow_key).is_some() {
            self.heap.remove(flow_key);
            self.indexer.remove(flow_key);
        }
    }
    
    pub fn gc(&mut self, elephant_length: &u32) -> Vec<(String, u32)> {
        // Find all `flow_keys` with m > 1000
        let keys_to_remove: Vec<(String, u32)> = self.heap
            .iter()
            .filter(|&(_, &m)| m.0 > *elephant_length)
            .map(|(flow_key, m)| (flow_key.clone(), m.0))
            .collect();

        // Remove these keys from indexer
        for (flow_key, _m) in &keys_to_remove {
            self.heap.remove(flow_key);
            self.indexer.remove(flow_key);
        }

        keys_to_remove
    }

    pub fn update(
        &mut self,
        node: Node,
    ) {
        if self.heap.get(&node.flow_key).is_some() {
            self.heap.change_priority(&node.flow_key, Reverse(node.m));
            self.indexer.insert(node.flow_key.to_string(), (node.m, node.to));
        }
    }

    pub fn get(&self, flow_key: &str) -> Option<&u32> {
        self.heap.get(flow_key).map(|m| &m.1.0)
    }

    pub fn get_efs(&self) -> Vec<&String> {
        self.heap.iter().map(|(flow_key, _m)| flow_key).collect::<Vec<_>>()
    }

    pub fn get_efs_iter(&self) -> Iter<String, Reverse<u32>> {
        self.heap.iter()
    }

    pub fn get_efs_detailed(&self) -> Vec<(&String, u32, bool)> {
        self.indexer.iter().map(|(flow_key, m)| (flow_key, m.0, m.1)).collect::<Vec<_>>()
    }

    pub fn get_all_nodes(&self) -> Vec<Node> {
        let nodes: Vec<Node> = self.indexer.iter().map(|(flow_key, (m, to))| Node {
            flow_key: flow_key.clone(),
            m: *m,
            to: *to,
        }).collect();

        nodes
    }

    pub fn len(&self) -> usize {
        self.heap.len()
    }
}


#[pyclass]
pub struct Sampler {
    pub default_delta: u16,  // 16_000, 32_000, 64_000
    pub elephant_length: u32,  // 100, 300, 1_000
    pub mouse_length: u32,  // 0, 10, 30
    pub holt: holt::LinearTrend,
    pub hasher: RandomState,
    pub random: SmallRng,
    pub k: usize,
    pub w: usize,
    pub h: usize,
    pub p_1: f64,
    pub p_2: f64,
    pub decay_occ4delta: Vec<u32>,
    pub decay_occ4n: Vec<u32>,
    pub hash_counter: Vec<Vec<Item>>,
    pub efp_counter: MinHeap,
    // <flow_key, (n, d, to)>
    pub non_target: HashMap<String, (u32, u16, bool)>,
}

#[pymethods]
impl Sampler {
    #[new]
    pub fn new(
        default_delta: u16,
        elephant_length: u32,
        mouse_length: u32,
        w: usize,
        h: usize,
        k: usize,
        p_1: f64,
        p_2: f64,
    ) -> PyResult<Self> {
        assert!(w > 0, "w must be greater than 0");
        assert!(h > 0, "h must be greater than 0");
        assert!(k > 0, "k must be greater than 0");
        assert!(p_1 >= 0.0 && p_1 <= 1.0, "p_1 must be between 0 and 1");
        assert!(p_2 >= 0.0 && p_2 <= 1.0, "p_2 must be between 0 and 1");

        let holt = holt::LinearTrend::new();
        let hash_counter: Vec<Vec<Item>> = vec![vec![Item::default(default_delta); w]; h];
        let efp_counter: MinHeap = MinHeap::with_capacity(k);
        let non_target: HashMap<String, (u32, u16, bool)> = HashMap::new();
        let seed = SystemTime::now().duration_since(
            UNIX_EPOCH
        ).unwrap().as_nanos() as u64;

        let mut sampler = Sampler {
            default_delta,
            elephant_length,
            mouse_length,
            holt,
            hasher: RandomState::with_seeds(seed, seed, seed, seed),
            random: SmallRng::from_entropy(),
            k,
            w,
            h,
            p_1,
            p_2,
            decay_occ4delta: Vec::with_capacity(k),
            decay_occ4n: Vec::with_capacity(k),
            hash_counter,
            efp_counter,
            non_target,
        };

        sampler.decay_occ4delta = sampler.p_occurrences(p_1, k, true);
        sampler.decay_occ4n = sampler.p_occurrences(p_2, k, false);

        Ok(sampler)
    }

    pub fn p_occurrences(&self, p: f64, steps: usize, increment: bool) -> Vec<u32> {
        assert!(p >= 0.0 && p <= 1.0, "p must be between 0 and 1");
        let mut occurrences = Vec::with_capacity(steps);

        for _step in 0..steps {
            let probability = p.powf(_step as f64);

            // let threshold = (decay_factor * (1u32 << 31) as f64) as u32;
            let tier = if increment {
                (1. - probability) * (1u32 << 31) as f64 // 1 - (p)^d
            } else {
                probability * (1u32 << 31) as f64 // (p)^n
            };

            occurrences.push(tier as u32);
        }

        occurrences
    }

    pub fn on_pkt_received_raw(
        &mut self,
        src_ip: &str,
        dst_ip: &str,
        src_port: &str,
        dst_port: &str,
        ts: u64  // millisecond
    ) {
        let flow_key_upstream = format!(
            "{}, {}, {}, {}", src_ip, dst_ip, src_port, dst_port
        );
        let flow_key_downstream = format!(
            "{}, {}, {}, {}", dst_ip, src_ip, dst_port, src_port
        );
        self.on_pkt_received(&flow_key_upstream, &flow_key_downstream, ts);
    }

    pub fn on_pkt_received(
        &mut self,
        flow_key_upstream: &str,
        flow_key_downstream: &str,
        ts: u64  // millisecond
    ) {
        if self.non_target.contains_key(
            flow_key_upstream
        ) || self.non_target.contains_key(
            flow_key_downstream
        ) {
            return;
        }

        // NOTE: check whether the `flow_key` is upstream or downstream
        let flow_key = if let Some(_m) = self.efp_counter.get(flow_key_upstream) {
            flow_key_upstream
        } else if let Some(_m) = self.efp_counter.get(flow_key_downstream) {
            flow_key_downstream
        } else {
            flow_key_upstream
        };

        let item_hashkey = self.hasher.hash_one(flow_key);
        let mut _hash_counter_global_max_n: u32 = 0;

        for i in 0..self.h {
            // Combine item hashkey and hash index (h-hash) to generate a unique item index
            let _ith_item_hashkey = (item_hashkey, i);
            let _ith_item_idx = self.hasher.hash_one(&_ith_item_hashkey) % self.w as u64;
            let _ith_item_idx = _ith_item_idx as usize;
            let _cur_item = &mut self.hash_counter[i][_ith_item_idx];

            let _is_hashkey_matches = _cur_item.hashkey == item_hashkey;
            let _is_item_empty = _cur_item.n == 0;

            if _is_hashkey_matches || _is_item_empty {
                _cur_item.hashkey = item_hashkey;
                _cur_item.n += 1;

                let mut tau_t: u64 = 0;
                if _cur_item.last_ts != 0 {
                    // Since the maximum value of the timeout threshold `delta` 
                    // is less than 16,000 (with a very small upward fluctuation),
                    // if a stream has not timed out, the difference between the timestamp
                    // of the current packet and the timestamp of the last packet 
                    // in that stream will definitely be within the range of `u16::MAX`,
                    // preventing any overflow.

                    // NOTE: Possible overflow due to timestamp precision
                    // (microsecond_u128 -> millisecond_u64):
                    // The converted result may be smaller
                    tau_t = ts.saturating_sub(_cur_item.last_ts);
                    // milliseconds to seconds
                    let rounded_seconds = (tau_t as f64 / 1000.).round();
                    _cur_item.d = _cur_item.d.saturating_add(rounded_seconds as u16);
                }

                // Accelerate elephant flow determination & distinguish between old and new flows
                let tau_t = tau_t as f32;
                let l_b = self.holt.forecast(tau_t);

                let default_delta = self.default_delta as f32;

                // FIXME: Hash collision -> Decay of delta immediately
                // if tau_t <= -default_delta {
                //     _cur_item.last_delta = _cur_item.last_delta.saturating_sub(1_000);
                //     _cur_item.d += 1;
                //     return;
                // }
                let last_delta = _cur_item.last_delta as f32;
                let cur_delta = last_delta + l_b.max(-default_delta);
                if tau_t > cur_delta && _cur_item.n > self.mouse_length {
                    self.non_target.insert(
                        flow_key.to_string(),
                        (_cur_item.n, _cur_item.d, true)
                    );
                    _cur_item.reset(self.default_delta);
                    self.efp_counter.remove(flow_key);

                    return;
                } else {
                    // Decay of delta according to a certain probability
                    let _cur_decay_occ4delta = if (_cur_item.d as usize) < self.decay_occ4delta.len() {
                        self.decay_occ4delta[_cur_item.d as usize]
                    } else {
                        self.decay_occ4delta.last().cloned().unwrap_or_default()
                    };

                    let rand = self.random.gen::<u32>();
                    if rand < _cur_decay_occ4delta {
                        // milliseconds
                        _cur_item.last_delta = _cur_item.last_delta.saturating_sub(1_000);
                    }
                }

                _cur_item.last_ts = ts;
                _hash_counter_global_max_n = max(_hash_counter_global_max_n, _cur_item.n);
            } else {
                // Decay of n
                let _cur_decay_occ4n = if (_cur_item.n as usize) < self.decay_occ4n.len() {
                    self.decay_occ4n[_cur_item.n as usize]
                } else {
                    self.decay_occ4n.last().cloned().unwrap_or_default()
                };

                // Apply bitwise decay based on the decay tier
                let rand = self.random.gen::<u32>();
                if rand < _cur_decay_occ4n {
                    _cur_item.n = _cur_item.n.saturating_sub(1);
                }
            }
        }

        let _efp_not_full = self.efp_counter.len() < self.k;
        let _cur_efp_min = self.efp_counter.peek();
        // No-op if max_count is less than the smallest count in the min-heap
        if _hash_counter_global_max_n < _cur_efp_min && !_efp_not_full {
            return;
        }

        if let Some(m) = self.efp_counter.get(flow_key) {
            if *m > self.elephant_length {
                let gc_list = self.efp_counter.gc(&self.elephant_length);
                for (_flow_key, m) in gc_list {
                    self.non_target.insert(
                        _flow_key.clone(),
                        (m, 0, false)
                    );
                    self.efp_counter.remove(_flow_key.as_str());
                    for i in 0..self.h {
                        let _ith_item_hashkey = (item_hashkey, i);
                        let _ith_item_idx = self.hasher.hash_one(&_ith_item_hashkey) % self.w as u64;
                        let _ith_item_idx = _ith_item_idx as usize;
                        let _cur_item = &mut self.hash_counter[i][_ith_item_idx];
                        _cur_item.reset(self.default_delta);
                    }
                }
                return;
            } else {
                self.efp_counter.update(
                    Node { flow_key: flow_key.to_string(), m: _hash_counter_global_max_n, to: false }
                );
            }
        } else if _efp_not_full || _hash_counter_global_max_n >= _cur_efp_min {
            if self.efp_counter.len() >= self.k {
                self.efp_counter.pop();
            }

            self.efp_counter.push(
                Node{
                    flow_key: flow_key.to_string(),
                    m: _hash_counter_global_max_n,
                    to: false,
                }
            );
        }
    }

    pub fn should_sample(
        &self,
        src_ip: &str,
        dst_ip: &str,
        src_port: &str,
        dst_port: &str,
    ) -> bool {
        let flow_key_upstream = format!(
            "{}, {}, {}, {}", src_ip, dst_ip, src_port, dst_port
        );
        let flow_key_downstream = format!(
            "{}, {}, {}, {}", dst_ip, src_ip, dst_port, src_port
        );

        if self.non_target.contains_key(
            &flow_key_upstream
        ) || self.non_target.contains_key(
            &flow_key_downstream
        ) {
            false
        } else {
            true
        }
    }

    pub fn get_efp(
        &self,
        src_ip: &str,
        dst_ip: &str,
        src_port: &str,
        dst_port: &str,
    ) -> Option<u32> {
        let flow_key_upstream = format!(
            "{}, {}, {}, {}", src_ip, dst_ip, src_port, dst_port
        );
        let flow_key_downstream = format!(
            "{}, {}, {}, {}", dst_ip, src_ip, dst_port, src_port
        );

        if let Some(m) = self.efp_counter.get(&flow_key_upstream) {
            return Some(*m);
        }

        if let Some(m) = self.efp_counter.get(&flow_key_downstream) {
            return Some(*m);
        }

        None
    }

    pub fn get_all_nodes(&self, py: Python<'_>) -> PyResult<PyObject> {
        let nodes = self.efp_counter.get_all_nodes();

        let py_nodes: Vec<Py<Node>> = nodes.into_iter().map(
            |node| Py::new(py, node).unwrap()
        ).collect();
        Ok(PyList::new_bound(py, py_nodes).into())
    }

    pub fn get_efs(&self) -> Vec<&String> {
        self.efp_counter.get_efs()
    }

    pub fn summary(&self) {
        info!("Small Flow Sampler:");
        info!("k: {}", self.k);
        info!("w: {}", self.w);
        info!("h: {}", self.h);
        info!("p_1: {}", self.p_1);
        info!("p_2: {}", self.p_2);
        info!("decay_occ4delta: {:?}", self.decay_occ4delta);
        info!("decay_thresholds4n: {:?}", self.decay_occ4n);
        info!("level: {:?}", self.holt.last_level);
        info!("trend: {:?}", self.holt.last_trend);

        info!("Hash Counter:");
        let mut items: Vec<(&Item, usize, usize)> = self
            .hash_counter
            .iter()
            .enumerate()
            .flat_map(|(i, row)| {
                row.iter()
                    .enumerate()
                    .map(move |(j, bucket)| (bucket, i, j))
            })
            .filter(|(bucket, _, _)| bucket.n != 0)
            .collect();
        items.sort_by(|a, b| b.0.n.cmp(&a.0.n));
        for (item, i, j) in items {
            info!("{}, {}: {:?}", i, j, item);
        }

        info!("EFP Counter:");
        let nodes = self.efp_counter.get_all_nodes();
        for node in nodes {
            info!("get_all_nodes - flow_key: {:?}, m: {}", node.flow_key, node.m);
        }
        for (key, m, to) in self.efp_counter.get_efs_detailed() {
            info!("get_all_efs - {}, m: {}, to: {}", key, m, to);
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let default_delta: u16 = 16_000;
        let elephant_length: u32 = 1000;
        let mouse_length: u32 = 30;
        let w = 600;
        let h = 3;
        let k = 100;
        let p_1 = 0.98;
        let p_2 = 0.96;

        let topk: Sampler = Sampler::new(
            default_delta, 
            elephant_length,
            mouse_length,
            w, h, k, p_1, p_2
        ).expect(
            "Sampler(w: usize, h: usize, k: usize, p_1: f64, p_2: f64)"
        );

        assert_eq!(topk.hash_counter.len(), 3);
        assert_eq!(topk.hash_counter[0].len(), 600);
        assert_eq!(topk.efp_counter.len(), 0);
    }
}
