use std::collections::HashMap;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::net::IpAddr;

use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::ipv6::Ipv6Packet;
use pnet::packet::tcp::TcpPacket;
use pnet::packet::Packet;

use crate::util::pkt_type::PacketDirection;
use crate::topk::small_flow_sampling::Sampler;


pub fn get_flow_key(
    src_ip: &str,
    dst_ip: &str,
    src_port: u16,
    dst_port: u16,
    direction: PacketDirection
) -> (String, String, u16, u16) {
    match direction {
        PacketDirection::Upstream => (
            src_ip.to_string(),
            dst_ip.to_string(),
            src_port,
            dst_port,
        ),
        PacketDirection::Downstream => (
            dst_ip.to_string(),
            src_ip.to_string(),
            dst_port,
            src_port,
        ),
    }
}

pub fn ip_from_u8(_ip: &[u8]) -> String {
	match _ip.len() {
		4 => {
			let ipv4 = Ipv4Addr::new(_ip[0], _ip[1], _ip[2], _ip[3]);

			ipv4.to_string()
		},
		16 => {
			let mut segments: [u16; 8] = [0; 8];
			for i in 0..8 {
				segments[i] = ((_ip[i * 2] as u16) << 8) | (_ip[i * 2 + 1] as u16);
			}
			let ipv6 = Ipv6Addr::new(
				segments[0], segments[1], segments[2], segments[3],
				segments[4], segments[5], segments[6], segments[7],
			);

			ipv6.to_string()
		},
		_ => "Invalid IP length".to_string(),
	}
}

pub fn ip_to_u8(_ip: &str)-> Option<Vec<u8>> {
    match _ip.parse::<IpAddr>() {
        Ok(ip) => match ip {
            IpAddr::V4(v4) => Some(v4.octets().to_vec()),
            IpAddr::V6(v6) => Some(v6.octets().to_vec()),
        },
        Err(_) => None,
    }
}

pub fn port_from_u8(high_byte: u8, low_byte: u8) -> u16 {
    // Left shift the high byte and perform a bitwise `OR` with the low byte
    ((high_byte as u16) << 8) | (low_byte as u16)
}

pub fn port_to_u8(port: u16) -> Vec<u8> {
    // Right shift by 8 bits and perform a bitwise `AND` with 0xFF
    let high_byte = (port >> 8) as u8;
    // Perform a bitwise `AND` with 0xFF
    let low_byte = (port & 0xFF) as u8;

    vec![high_byte, low_byte]
}

pub fn update_flow_count(
    target: &mut HashMap<String, u32>,
    src_ip: &str,
    dst_ip: &str,
    src_port: &str,
    dst_port: &str,
) {
    let flow_key_upstream = format!(
        "{}, {}, {}, {}", src_ip, dst_ip, src_port, dst_port
    );
    let flow_key_downstream = format!(
        "{}, {}, {}, {}", dst_ip, src_ip, dst_port, src_port
    );

    if let Some(cur_count) = target.get(&flow_key_upstream) {
        target.insert(flow_key_upstream, cur_count + 1);
    } else if let Some(cur_count) = target.get(&flow_key_downstream) {
        target.insert(flow_key_downstream, cur_count + 1);
    } else {
        target.insert(flow_key_upstream, 1);
    }
}


#[cfg(test)]
mod tests {
    use crate::util::helpers::{port_to_u8, port_from_u8};

    #[test]
    fn test_port_conversion() {
        let port: u16 = 8080;

        let bytes = port_to_u8(port);
		// `31` is the high byte and `64` is the low byte
        assert_eq!(bytes, vec![31, 144]);

        let reconstructed_port = port_from_u8(bytes[0], bytes[1]);
        assert_eq!(reconstructed_port, port);
    }
}
