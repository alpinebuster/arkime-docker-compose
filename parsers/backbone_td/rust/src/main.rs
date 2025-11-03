use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use std::env;
use std::mem;

use chrono::{FixedOffset, Utc};
use pcap::Capture;
use pnet::packet::Packet;
use pnet::packet::ethernet::{EthernetPacket, EtherTypes};
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::tcp::TcpPacket;
use pnet::packet::udp::UdpPacket;
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::ipv6::Ipv6Packet;
use ahash::RandomState;
use rand::SeedableRng;
use rand::rngs::SmallRng;
use regex::Regex;

mod topk;
mod util;

use crate::topk::holt;
use crate::topk::small_flow_sampling::{Item, MinHeap, Sampler, Node};
use crate::util::tor::FlowLabel;
use crate::util::helpers::update_flow_count;

const BEIJING_OFFSET: FixedOffset = FixedOffset::east_opt(
    8 * 3600
).expect("Invalid offset");


// `cargo run --no-default-features ELEPHANT_100-MOUSE_10-DELTA_8000 cetc30 .0010`
// `cargo build --no-default-features --release`
// `./target/release/src ELEPHANT_100-MOUSE_10-DELTA_8000 cetc30 .0010>"cf-cetc30-.0010.log" 2>&1 &`
fn main() {
    // setup_logging().expect("Failed to set up logging");
    // test_minheap();

    let valid_src_datasets: HashSet<&str> = [
        "cetc30", "cicdarknet", "tcub"
    ].iter().cloned().collect();
    let valid_ratios: HashSet<&str> = [
        ".1000", ".0500", ".0100", ".0050", ".0010", ".0005", ".0001"
    ].iter().cloned().collect();

    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        eprintln!("Usage: cargo run <global_exp_config> <tor_src_dataset> <tor_ratio>");
        std::process::exit(1);
    }
    let global_exp_config = &args[1];
    let tor_src_dataset = &args[2];
    let tor_ratio = &args[3];

    println!("\n{} &args[0]: {}", Utc::now().with_timezone(&BEIJING_OFFSET), &args[0]);
    println!("global_exp_config: {}", global_exp_config);
    println!("tor_src_dataset: {}", tor_src_dataset);
    println!("tor_ratio: {}", tor_ratio);

    if !valid_src_datasets.contains(tor_src_dataset.as_str()) {
        eprintln!(
            "Invalid src_dataset: {}. Valid options are: {:?}",
            tor_src_dataset, valid_src_datasets
        );
        std::process::exit(1);
    }
    if !valid_ratios.contains(tor_ratio.as_str()) {
        eprintln!(
            "Invalid ratio: {}. Valid options are: {:?}",
            tor_ratio, valid_ratios
        );
        std::process::exit(1);
    }
    let pattern = r"ELEPHANT_(\d+)-MOUSE_(\d+)-DELTA_(\d+)";
    let re = Regex::new(pattern).unwrap();

    if let Some(captures) = re.captures(global_exp_config) {
        let elephant_length: u32 = captures[1].parse().expect(
            "Failed to parse elephant length"
        );
        let mouse_length: u32 = captures[2].parse().expect(
            "Failed to parse mouse length"
        );
        let default_delta: u16 = captures[3].parse().expect(
            "Failed to parse delta length"
        );

        collaborative_filtering(
            default_delta, elephant_length, mouse_length,
            tor_src_dataset, tor_ratio
        );
        // run_memory_benchmark(
        //     default_delta, elephant_length, mouse_length,
        //     tor_src_dataset, tor_ratio
        // );
    } else {
        println!(
            "Invalid global_exp_config: {}. Valid options are: {}",
            global_exp_config, pattern
        );
        std::process::exit(1);
    }
}

fn collaborative_filtering(
    default_delta: u16, elephant_length: u32, mouse_length: u32,
    tor_src_dataset: &str, tor_ratio: &str
) {
    let w = 2usize.pow(16);  // 65536
    let h = 3;
    let k = 2usize.pow(10);  // 1024
    let p_1 = 0.99;
    let p_2 = 0.96;

    let start = Instant::now();

    let mut total_pkt_count_before: u32 = 0;
    let mut total_pkt_count_after: u32 = 0;
    let mut tor_pkt_count_before: u32 = 0;
    let mut tor_pkt_count_after: u32 = 0;

    let mut after_rule_based_pkt_count: u32 = 0;

    let mut total_flow_count_before: HashMap<String, u32> = HashMap::new();
    let mut total_flow_count_after: HashMap<String, u32> = HashMap::new();
    let mut tor_flow_count_before: HashMap<String, u32> = HashMap::new();
    let mut tor_flow_count_after: HashMap<String, u32> = HashMap::new();

    let mut nontcp_total_flow_count: HashMap<String, u32> = HashMap::new();

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
        hasher: RandomState::with_seeds(
            seed, seed, seed, seed
        ),
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

    // Specify the path to your PCAP file
    let base_dir = Path::new(
        "/"
    );
    // let pcap_file = base_dir + "_data/CETC30/2016YFE0206700-001-Meek_Tor/tor_meek_18.pcap";
    // let pcap_file = base_dir + "_data/CIC-Darknet2020/ISCXTor2016/PCAPs/Tor/mail_gateway_thunderbird_imap.pcap";
    let pcap_file = base_dir.join(
        format!(
            "_data/LOWPTOR/{}/lowptor-{}-{}.pcap",
            tor_src_dataset, tor_src_dataset, tor_ratio
        )
    );
    let label_file = base_dir.join(
        format!(
            "_code/backbone_traffic_detection/data/lowptor-{}-label-tor.csv",
            tor_src_dataset
        )
    );
    println!("{} pcap_file: {:?}", Utc::now().with_timezone(&BEIJING_OFFSET), pcap_file);
    println!("{} label_file: {:?}", Utc::now().with_timezone(&BEIJING_OFFSET), label_file);
    let mut pcap_obj = Capture::from_file(
        &pcap_file
    ).expect("Failed to open PCAP file");
    let mut label_checker = FlowLabel::new(
        label_file.to_str().unwrap()
    ).unwrap();

    // Iterate over packets
    while let Ok(pkt) = pcap_obj.next_packet() {
        total_pkt_count_before += 1;

        let timestamp = pkt.header.ts;
        let _seconds = timestamp.tv_sec as u64;
        let _microseconds = timestamp.tv_usec as u64;

        // u64::MAX => 18_446_744_073_709_551_615
        let _total_millis = (_seconds * 1_000) + _microseconds / 1_000; 
        // (1970-01-01, 2025-01-01) => 1_735_668_000_000
        // let years = 55;
        // let days_per_year = 365.25;
        // let hours_per_day = 24;
        // let minutes_per_hour = 60;
        // let seconds_per_minute = 60;
        // let millis_per_second = 1000;
        let unix_millis = std::time::Duration::from_millis(_total_millis).as_millis() as u64;

        let ethernet_pkt = EthernetPacket::new(pkt.data).unwrap();
        match ethernet_pkt.get_ethertype() {
            EtherTypes::Ipv4 => {
                if let Some(ip_pkt) = Ipv4Packet::new(
                    ethernet_pkt.payload()
                ) {
                    let src_ip_string = ip_pkt.get_source().to_string(); // Create a String
                    let src_ip: &str = &src_ip_string; // Create a reference to the String
                    let dst_ip_string = ip_pkt.get_destination().to_string();
                    let dst_ip = &dst_ip_string;

                    match ip_pkt.get_next_level_protocol() {
                        IpNextHeaderProtocols::Tcp => {
                            if let Some(tcp_pkt) = TcpPacket::new(
                                ip_pkt.payload()
                            ) {
                                let src_port_string = tcp_pkt.get_source().to_string();
                                let src_port = &src_port_string;
                                let dst_port_string = tcp_pkt.get_destination().to_string();
                                let dst_port = &dst_port_string;

                                stat(
                                    &mut sampler,
                                    &mut label_checker,
                                    src_ip, dst_ip, src_port, dst_port,
                                    unix_millis,
                                    &mut total_pkt_count_after,
                                    &mut tor_pkt_count_before,
                                    &mut tor_pkt_count_after,
                                    &mut after_rule_based_pkt_count,
                                    &mut total_flow_count_before,
                                    &mut total_flow_count_after,
                                    &mut tor_flow_count_before,
                                    &mut tor_flow_count_after,
                                );
                            }
                        }
                        IpNextHeaderProtocols::Udp => {
                            if let Some(udp_pkt) = UdpPacket::new(ip_pkt.payload()) {
                                let src_port_string = udp_pkt.get_source().to_string();
                                let src_port = &src_port_string;
                                let dst_port_string = udp_pkt.get_destination().to_string();
                                let dst_port = &dst_port_string;

                                update_flow_count(
                                    &mut nontcp_total_flow_count,
                                    src_ip,
                                    dst_ip, 
                                    src_port,
                                    dst_port
                                );
                            }
                        }
                        _ => {
                            update_flow_count(
                                &mut nontcp_total_flow_count,
                                src_ip,
                                dst_ip, 
                                "",
                                "",
                            );
                            continue;
                        }
                    }
                }
            }
            EtherTypes::Ipv6 => {
                if let Some(ipv6_pkt) = Ipv6Packet::new(
                    ethernet_pkt.payload()
                ) {
                    let src_ip_string = ipv6_pkt.get_source().to_string();
                    let src_ip: &str = &src_ip_string;
                    let dst_ip_string = ipv6_pkt.get_destination().to_string();
                    let dst_ip = &dst_ip_string;

                    match ipv6_pkt.get_next_header() {
                        IpNextHeaderProtocols::Tcp => {
                            if let Some(tcp_pkt) = TcpPacket::new(
                                ipv6_pkt.payload()
                            ) {
                                let src_port_string = tcp_pkt.get_source().to_string();
                                let src_port = &src_port_string;
                                let dst_port_string = tcp_pkt.get_destination().to_string();
                                let dst_port = &dst_port_string;

                                stat(
                                    &mut sampler,
                                    &mut label_checker,
                                    src_ip, dst_ip, src_port, dst_port,
                                    unix_millis,
                                    &mut total_pkt_count_after,
                                    &mut tor_pkt_count_before,
                                    &mut tor_pkt_count_after,
                                    &mut after_rule_based_pkt_count,
                                    &mut total_flow_count_before,
                                    &mut total_flow_count_after,
                                    &mut tor_flow_count_before,
                                    &mut tor_flow_count_after,
                                );
                            }
                        }
                        IpNextHeaderProtocols::Udp => {
                            if let Some(udp_pkt) = UdpPacket::new(ipv6_pkt.payload()) {
                                let src_port_string = udp_pkt.get_source().to_string();
                                let src_port = &src_port_string;
                                let dst_port_string = udp_pkt.get_destination().to_string();
                                let dst_port = &dst_port_string;

                                update_flow_count(
                                    &mut nontcp_total_flow_count,
                                    src_ip,
                                    dst_ip, 
                                    src_port,
                                    dst_port
                                );
                            }
                        }
                        _ => {
                            update_flow_count(
                                &mut nontcp_total_flow_count,
                                src_ip,
                                dst_ip, 
                                "",
                                "",
                            );
                            continue;
                        }
                    }
                }
            }
            _ => {
                continue;
            }
        }
    }

    println!(
        "\n{} Results:\nTotal time: {}",
        Utc::now().with_timezone(&BEIJING_OFFSET),
        start.elapsed().as_secs_f32()
    );

    // println!("\n{} Hash Counter:", Utc::now().with_timezone(&BEIJING_OFFSET));
    // let items: Vec<(&Item, usize, usize)> = sampler
    //     .hash_counter
    //     .iter()
    //     .enumerate()
    //     .flat_map(|(i, row)| {
    //         row.iter()
    //             .enumerate()
    //             .map(move |(j, bucket)| (bucket, i, j))
    //     })
    //     .filter(|(bucket, _, _)| bucket.n != 0)
    //     .collect();
    // for (item, i, j) in items {
    //     println!("{}, {}: {:?}", i, j, item);
    // }

    // println!("\n{} EFP Counter:", Utc::now().with_timezone(&BEIJING_OFFSET));
    // let nodes = sampler.efp_counter.get_all_nodes();
    // println!("get_all_nodes - len: {}", nodes.len());
    // for node in nodes {
    //     println!("get_all_nodes - flow_key: {:?}, m: {}", node.flow_key, node.m);
    // }
    // for (key, m, to) in sampler.efp_counter.get_efs_detailed() {
    //     println!("get_all_efs - {}, m: {}, to: {}", key, m, to);
    // }

    // println!("\n{} Sampler:", Utc::now().with_timezone(&BEIJING_OFFSET));
    // let non_target_list: Vec<_> = sampler.non_target.keys().collect();
    // println!("non_target - len: {}", non_target_list.len());
    // for key in non_target_list {
    //     if let Some(value) = sampler.non_target.get(key) {
    //         println!("non_target - {}: {:?}", key, value);
    //     }
    // }

    println!("\ntotal_pkt_count_before: {}", total_pkt_count_before);
    println!("total_pkt_count_after: {}", total_pkt_count_after);
    println!("tor_pkt_count_before: {}", tor_pkt_count_before);
    println!("tor_pkt_count_after: {}", tor_pkt_count_after);
    println!("after_rule_based_pkt_count: {}", after_rule_based_pkt_count);

    println!(
        "\nnontcp_total_flow_count, len: {}, data:",
        nontcp_total_flow_count.len()
    );
    // for (flow, count) in &nontcp_total_flow_count {
    //     println!("{}: {}", flow, count);
    // }

    println!(
        "\ntotal_flow_count_before, len: {}, data:",
        total_flow_count_before.len()
    );
    // for (flow, count) in &total_flow_count_before {
    //     println!("{}: {}", flow, count);
    // }
    println!(
        "\ntotal_flow_count_after, len: {}, data:",
        &total_flow_count_after.len()
    );
    // for (flow, count) in &total_flow_count_after {
    //     println!("{}: {}", flow, count);
    // }
    println!(
        "\ntor_flow_count_before, len: {}, data:",
        &tor_flow_count_before.len()
    );
    // for (flow, count) in &tor_flow_count_before {
    //     println!("{}: {}", flow, count);
    // }
    println!(
        "\ntor_flow_count_after, len: {}, data:",
        &tor_flow_count_after.len()
    );
    // for (flow, count) in &tor_flow_count_after {
    //     println!("{}: {}", flow, count);
    // }

    // let is_subset = tor_flow_count_after.keys().all(
    //     |key| tor_flow_count_before.contains_key(key)
    // );
    // if is_subset {
    //     // println!("`tor_flow_count_after` is a subset of `tor_flow_count_before`");
    //     let tor_flow_count_diff: HashMap<_, _> = tor_flow_count_before
    //         .iter()
    //         .filter(
    //             |(key, _)| !tor_flow_count_after.contains_key(*key)
    //         ).collect();

    //         println!(
    //             "\ntor_flow_count_diff, len: {}, data:",
    //             &tor_flow_count_diff.len()
    //         );
    //         for (flow, count) in &tor_flow_count_diff {
    //             println!("{}: {}", flow, count);
    //         }
    // } else {
    //     println!(
    //         "{} WARNING - `tor_flow_count_after` is NOT a subset of `tor_flow_count_before`",
    //         Utc::now().with_timezone(&BEIJING_OFFSET)
    //     );
    // }
}

fn stat(
    sampler: &mut Sampler,
    label_checker: &mut FlowLabel,
    src_ip: &str,
    dst_ip: &str,
    src_port: &str,
    dst_port: &str,
    unix_millis: u64,
    total_pkt_count_after: &mut u32,
    tor_pkt_count_before: &mut u32,
    tor_pkt_count_after: &mut u32,
    after_rule_based_pkt_count: &mut u32,
    total_flow_count_before: &mut HashMap<String, u32>,
    total_flow_count_after: &mut HashMap<String, u32>,
    tor_flow_count_before: &mut HashMap<String, u32>,
    tor_flow_count_after: &mut HashMap<String, u32>,
) {
    // println!(
    //     "Timestamp: {} | Source IP: {} | Destination IP: {} | Source Port: {} | Destination Port: {}",
    //     unix_millis, src_ip, dst_ip, src_port, dst_port
    // );
    *after_rule_based_pkt_count += 1;

    let is_tor = label_checker.is_tor(
        src_ip, dst_ip, src_port, dst_port
    );
    sampler.on_pkt_received_raw(
        src_ip,
        dst_ip, 
        src_port,
        dst_port,
        unix_millis,
    );

    // Before collaborative filtering
    update_flow_count(
        total_flow_count_before,
        src_ip,
        dst_ip, 
        src_port,
        dst_port
    );
    if is_tor {
        *tor_pkt_count_before += 1;
        update_flow_count(
            tor_flow_count_before,
            src_ip,
            dst_ip, 
            src_port,
            dst_port
        );
    }

    // After collaborative filtering
    if sampler.should_sample(
        src_ip, dst_ip, src_port, dst_port
    ) {
        *total_pkt_count_after += 1;
        update_flow_count(
            total_flow_count_after,
            src_ip,
            dst_ip, 
            src_port,
            dst_port
        );
        if is_tor {
            *tor_pkt_count_after += 1;
            update_flow_count(
                tor_flow_count_after,
                src_ip,
                dst_ip, 
                src_port,
                dst_port
            );
        }
    }
}

fn test_minheap() {
    let elephant_length: u32 = 1000;
    let mut h = MinHeap::with_capacity(20);
    h.push(Node{
        flow_key: "1".to_string(),
        m: 43,
        to: false
    });
    h.push(Node{
        flow_key: "31".to_string(),
        m: 343,
        to: false
    });
    h.push(Node{
        flow_key: "51".to_string(),
        m: 343,
        to: false
    });
    h.push(Node{
        flow_key: "1345".to_string(),
        m: 343,
        to: false
    });
    h.push(Node{
        flow_key: "134".to_string(),
        m: 323,
        to: false
    });
    h.push(Node{
        flow_key: "51".to_string(),
        m: 324335,
        to: false
    });
    h.push(Node{
        flow_key: "12".to_string(),
        m: 3243345,
        to: false
    });
    h.push(Node{
        flow_key: "12".to_string(),
        m: 32,
        to: true
    });

    h.gc(&elephant_length);
    println!("{:?}", h.peek());
    println!("{:?}", h.heap);
    println!("{:?}", h.get_efs());
    println!("{:?}", h.get_efs_detailed());
    println!("{:?}", h.get_all_nodes());
    println!("{:?}", h.len());
    println!("{:?}", h.get("134"));
    println!("{:?}", h.get("12"));
    println!("{:?}", h.pop());
    println!("{:?}", h.get_efs());
    println!("{:?}", h.get_efs_detailed());
    println!("{:?}", h.get_all_nodes());
    println!("{:?}", h.len());
    println!("{:?}", h.get("134"));
    println!("{:?}", h.get("12"));
    println!("{:?}", h.update(
        Node {flow_key: "31".to_string(), m: 23423, to: false}
    ));
    println!("{:?}", h.heap);
}


/// Comprehensive memory usage statistics for benchmarking
#[derive(Debug, Clone)]
pub struct MemoryStats {
    /// Total number of active Items across all hash tables
    pub active_items_count: usize,
    /// Total memory used by active Items (excluding empty slots)
    pub active_items_memory: usize,
    /// Total number of Nodes in EFP counter
    pub active_nodes_count: usize,
    /// Total memory used by Nodes
    pub active_nodes_memory: usize,
    /// Memory used by non_target HashMap
    pub non_target_memory: usize,
    /// Total allocated memory for hash counter (including empty slots)
    pub hash_counter_allocated_memory: usize,
    /// Memory efficiency ratio (active/allocated for hash counter)
    pub hash_counter_efficiency: f64,
    /// Average memory per flow (considering both Items and Nodes)
    pub avg_memory_per_flow: f64,
    /// Number of unique flows processed so far
    pub unique_flows_processed: usize,
    /// Peak memory usage observed
    pub peak_total_memory: usize,
}

impl MemoryStats {
    pub fn new() -> Self {
        MemoryStats {
            active_items_count: 0,
            active_items_memory: 0,
            active_nodes_count: 0,
            active_nodes_memory: 0,
            non_target_memory: 0,
            hash_counter_allocated_memory: 0,
            hash_counter_efficiency: 0.0,
            avg_memory_per_flow: 0.0,
            unique_flows_processed: 0,
            peak_total_memory: 0,
        }
    }

    /// Calculate total active memory usage
    pub fn total_active_memory(&self) -> usize {
        self.active_items_memory + self.active_nodes_memory + self.non_target_memory
    }

    /// Update peak memory if current usage is higher
    pub fn update_peak(&mut self) {
        let current_total = self.total_active_memory();
        if current_total > self.peak_total_memory {
            self.peak_total_memory = current_total;
        }
    }
}

/// Detailed benchmark runner that tracks memory usage throughout packet processing
pub struct MemoryBenchmark {
    pub sampler: Sampler,
    pub stats_history: Vec<MemoryStats>,
    pub flow_tracker: HashSet<String>,
    pub snapshot_interval: usize, // Take snapshot every N packets
    pub packet_count: usize,
}

impl MemoryBenchmark {
    pub fn new(
        default_delta: u16,
        elephant_length: u32,
        mouse_length: u32,
        w: usize,
        h: usize,
        k: usize,
        p_1: f64,
        p_2: f64,
        snapshot_interval: usize,
    ) -> Self {
        // Initialize sampler similar to your original code
        let mut sampler = Sampler::new(
            default_delta, elephant_length, mouse_length, w, h, k, p_1, p_2
        ).expect("Failed to create sampler");

        MemoryBenchmark {
            sampler,
            stats_history: Vec::new(),
            flow_tracker: HashSet::new(),
            snapshot_interval,
            packet_count: 0,
        }
    }

    /// Calculate detailed memory statistics for current sampler state
    pub fn calculate_memory_stats(&mut self) -> MemoryStats {
        let mut stats = MemoryStats::new();

        // Calculate memory usage for hash counter (Items)
        let item_size = mem::size_of::<Item>();
        stats.hash_counter_allocated_memory = self.sampler.w * self.sampler.h * item_size;

        let mut active_items = 0;
        for row in &self.sampler.hash_counter {
            for item in row {
                if item.n > 0 {
                    active_items += 1;
                }
            }
        }
        
        stats.active_items_count = active_items;
        stats.active_items_memory = active_items * item_size;

        // Calculate memory usage for EFP counter (Nodes)
        stats.active_nodes_count = self.sampler.efp_counter.len();
        
        // Estimate Node memory: String (flow_key) + u32 (m) + bool (to) + HashMap overhead
        let base_node_size = mem::size_of::<u32>() + mem::size_of::<bool>();
        let mut total_node_memory = 0;
        
        for (flow_key, (m, to)) in &self.sampler.efp_counter.indexer {
            // Estimate String memory: capacity * char size + String struct overhead
            let string_memory = flow_key.capacity() + mem::size_of::<String>();
            let entry_memory = string_memory + base_node_size + mem::size_of::<(u32, bool)>();
            total_node_memory += entry_memory;
        }
        stats.active_nodes_memory = total_node_memory;

        // Calculate memory usage for non_target HashMap
        let mut non_target_memory = 0;
        for (flow_key, (n, d, to)) in &self.sampler.non_target {
            let string_memory = flow_key.capacity() + mem::size_of::<String>();
            let tuple_memory = mem::size_of::<(u32, u16, bool)>();
            non_target_memory += string_memory + tuple_memory;
        }
        stats.non_target_memory = non_target_memory;

        // Calculate efficiency and averages
        stats.hash_counter_efficiency = if stats.hash_counter_allocated_memory > 0 {
            stats.active_items_memory as f64 / stats.hash_counter_allocated_memory as f64
        } else {
            0.0
        };

        stats.unique_flows_processed = self.flow_tracker.len();
        stats.avg_memory_per_flow = if stats.unique_flows_processed > 0 {
            stats.total_active_memory() as f64 / stats.unique_flows_processed as f64
        } else {
            0.0
        };

        stats.update_peak();
        stats
    }

    /// Process a single packet and update statistics if needed
    pub fn process_packet(
        &mut self,
        src_ip: &str,
        dst_ip: &str,
        src_port: &str,
        dst_port: &str,
        timestamp: u64,
    ) {
        // Track unique flows
        let flow_key_upstream = format!("{}, {}, {}, {}", src_ip, dst_ip, src_port, dst_port);
        let flow_key_downstream = format!("{}, {}, {}, {}", dst_ip, src_ip, dst_port, src_port);

        let flow_key = if let Some(_m) = self.sampler.efp_counter.get(&flow_key_upstream) {
            flow_key_upstream
        } else if let Some(_m) = self.sampler.efp_counter.get(&flow_key_downstream) {
            flow_key_downstream
        } else {
            flow_key_upstream
        };
        self.flow_tracker.insert(flow_key);

        // Process packet through sampler
        self.sampler.on_pkt_received_raw(src_ip, dst_ip, src_port, dst_port, timestamp);
        
        self.packet_count += 1;

        // Take memory snapshot at specified intervals
        if self.packet_count % self.snapshot_interval == 0 {
            let stats = self.calculate_memory_stats();
            self.stats_history.push(stats);
        }
    }

    /// Generate comprehensive benchmark report
    pub fn generate_report(&mut self) -> String {
        // Take final snapshot
        let final_stats = self.calculate_memory_stats();
        self.stats_history.push(final_stats.clone());

        let mut report = String::new();
        
        report.push_str(&format!("=== Memory Usage Benchmark Report ===\n"));
        report.push_str(&format!("Total packets processed: {}\n", self.packet_count));
        report.push_str(&format!("Unique flows encountered: {}\n", self.flow_tracker.len()));
        report.push_str(&format!("Snapshots taken: {}\n\n", self.stats_history.len()));

        // Final state analysis
        report.push_str(&format!("=== Final Memory State ===\n"));
        report.push_str(&format!("Active Items: {} ({}KB)\n", 
            final_stats.active_items_count,
            final_stats.active_items_memory / 1024));
        report.push_str(&format!("Active Nodes: {} ({}KB)\n", 
            final_stats.active_nodes_count,
            final_stats.active_nodes_memory / 1024));
        report.push_str(&format!("Non-target entries: {}KB\n", 
            final_stats.non_target_memory / 1024));
        report.push_str(&format!("Hash counter allocated: {}KB\n", 
            final_stats.hash_counter_allocated_memory / 1024));
        report.push_str(&format!("Hash counter efficiency: {:.2}%\n", 
            final_stats.hash_counter_efficiency * 100.0));
        report.push_str(&format!("Average memory per flow: {:.2} bytes\n", 
            final_stats.avg_memory_per_flow));
        report.push_str(&format!("Peak total memory: {}KB\n\n", 
            final_stats.peak_total_memory / 1024));

        // Historical analysis
        if self.stats_history.len() > 1 {
            report.push_str(&format!("=== Memory Growth Analysis ===\n"));
            
            let first_stats = &self.stats_history[0];
            let growth_items = final_stats.active_items_count as i32 - first_stats.active_items_count as i32;
            let growth_nodes = final_stats.active_nodes_count as i32 - first_stats.active_nodes_count as i32;
            let growth_memory = final_stats.total_active_memory() as i32 - first_stats.total_active_memory() as i32;
            
            report.push_str(&format!("Item count change: {}\n", growth_items));
            report.push_str(&format!("Node count change: {}\n", growth_nodes));
            report.push_str(&format!("Total memory change: {}KB\n", growth_memory / 1024));
            
            // Calculate average memory per new flow
            let new_flows = final_stats.unique_flows_processed - first_stats.unique_flows_processed;
            if new_flows > 0 {
                report.push_str(&format!("Average memory per new flow: {:.2} bytes\n", 
                    growth_memory as f64 / new_flows as f64));
            }
        }

        // Efficiency analysis
        report.push_str(&format!("\n=== Efficiency Metrics ===\n"));
        let total_allocated = final_stats.hash_counter_allocated_memory + 
                             (self.sampler.k * mem::size_of::<Node>()); // Estimated max EFP capacity
        let utilization = final_stats.total_active_memory() as f64 / total_allocated as f64;
        report.push_str(&format!("Overall memory utilization: {:.2}%\n", utilization * 100.0));
        
        let avg_items_per_flow = final_stats.active_items_count as f64 / self.flow_tracker.len() as f64;
        let avg_nodes_per_flow = final_stats.active_nodes_count as f64 / self.flow_tracker.len() as f64;
        report.push_str(&format!("Average Items per flow: {:.2}\n", avg_items_per_flow));
        report.push_str(&format!("Average Nodes per flow: {:.2}\n", avg_nodes_per_flow));

        report
    }

    /// Export detailed statistics for further analysis
    pub fn export_stats_csv(&self) -> String {
        let mut csv = String::new();
        csv.push_str("snapshot,active_items,active_items_kb,active_nodes,active_nodes_kb,non_target_kb,total_active_kb,unique_flows,avg_memory_per_flow,efficiency_percent\n");
        
        for (i, stats) in self.stats_history.iter().enumerate() {
            csv.push_str(&format!("{},{},{},{},{},{},{},{},{:.2},{:.2}\n",
                i,
                stats.active_items_count,
                stats.active_items_memory / 1024,
                stats.active_nodes_count,
                stats.active_nodes_memory / 1024,
                stats.non_target_memory / 1024,
                stats.total_active_memory() / 1024,
                stats.unique_flows_processed,
                stats.avg_memory_per_flow,
                stats.hash_counter_efficiency * 100.0
            ));
        }
        
        csv
    }
}

/// Modified main function to run the memory benchmark
fn run_memory_benchmark(
    default_delta: u16, elephant_length: u32, mouse_length: u32,
    tor_src_dataset: &str, tor_ratio: &str
) {
    // Benchmark parameters
    let w = 2usize.pow(16);  // 65536
    let h = 3;
    let k = 2usize.pow(10);  // 1024
    let p_1 = 0.99;
    let p_2 = 0.96;
    let snapshot_interval = 10000; // Take snapshot every 10K packets

    let mut benchmark = MemoryBenchmark::new(
        default_delta, elephant_length, mouse_length,
        w, h, k, p_1, p_2, snapshot_interval
    );

    let start = Instant::now();

    // Process PCAP file (similar to your original code)
    let base_dir = std::path::Path::new("/");
    let pcap_file = base_dir.join(format!(
        "_data/LOWPTOR/{}/lowptor-{}-{}.pcap",
        tor_src_dataset, tor_src_dataset, tor_ratio
    ));

    println!("{} Processing: {:?}", Utc::now().with_timezone(&BEIJING_OFFSET), pcap_file);
    
    let mut pcap_obj = Capture::from_file(&pcap_file).expect("Failed to open PCAP file");

    // Process packets with memory tracking
    while let Ok(pkt) = pcap_obj.next_packet() {
        let timestamp = pkt.header.ts;
        let unix_millis = (timestamp.tv_sec as u64 * 1000) + (timestamp.tv_usec as u64 / 1000);

        let ethernet_pkt = EthernetPacket::new(pkt.data).unwrap();
        match ethernet_pkt.get_ethertype() {
            EtherTypes::Ipv4 => {
                if let Some(ip_pkt) = Ipv4Packet::new(ethernet_pkt.payload()) {
                    if let IpNextHeaderProtocols::Tcp = ip_pkt.get_next_level_protocol() {
                        if let Some(tcp_pkt) = TcpPacket::new(ip_pkt.payload()) {
                            benchmark.process_packet(
                                &ip_pkt.get_source().to_string(),
                                &ip_pkt.get_destination().to_string(),
                                &tcp_pkt.get_source().to_string(),
                                &tcp_pkt.get_destination().to_string(),
                                unix_millis,
                            );
                        }
                    }
                }
            }
            EtherTypes::Ipv6 => {
                if let Some(ipv6_pkt) = Ipv6Packet::new(ethernet_pkt.payload()) {
                    if let IpNextHeaderProtocols::Tcp = ipv6_pkt.get_next_header() {
                        if let Some(tcp_pkt) = TcpPacket::new(ipv6_pkt.payload()) {
                            benchmark.process_packet(
                                &ipv6_pkt.get_source().to_string(),
                                &ipv6_pkt.get_destination().to_string(),
                                &tcp_pkt.get_source().to_string(),
                                &tcp_pkt.get_destination().to_string(),
                                unix_millis,
                            );
                        }
                    }
                }
            }
            _ => continue,
        }
    }

    println!("\n{} Benchmark completed in {:.2}s", 
        Utc::now().with_timezone(&BEIJING_OFFSET),
        start.elapsed().as_secs_f32()
    );

    // Generate and display comprehensive report
    let report = benchmark.generate_report();
    println!("{}", report);

    // Export CSV for detailed analysis
    let csv_data = benchmark.export_stats_csv();
    println!("\n=== CSV Export for Analysis ===");
    println!("{}", csv_data);

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_benchmark_basic() {
        let mut benchmark = MemoryBenchmark::new(
            16000, 100, 10, 100, 3, 50, 0.99, 0.96, 5
        );

        // Simulate some packet processing
        for i in 0..20 {
            benchmark.process_packet(
                "192.168.1.1", "192.168.1.2", 
                &format!("{}", 8000 + i), "80", 
                1000000 + i as u64
            );
        }

        let stats = benchmark.calculate_memory_stats();
        assert!(stats.unique_flows_processed > 0);
        assert!(stats.total_active_memory() > 0);

        let report = benchmark.generate_report();
        assert!(report.contains("Memory Usage Benchmark Report"));
    }
}
